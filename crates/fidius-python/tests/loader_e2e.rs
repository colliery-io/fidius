// Copyright 2026 Colliery, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! End-to-end loader test: build a small Python plugin in a temp dir,
//! prepend the in-tree `python/fidius` SDK to its vendor/, load it via
//! `load_python_plugin`, and exercise typed + raw + error paths.
//!
//! No PluginHost integration here — that lands in T-0090. This test pokes
//! the loader API directly so we can verify the dispatcher's behaviour
//! independently.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use fidius_core::python_descriptor::{PythonInterfaceDescriptor, PythonMethodDesc};
use fidius_python::{load_python_plugin, PythonCallError};

const HASH: u64 = 0xDEADBEEF_CAFEF00D;
const GREETER_METHODS: [PythonMethodDesc; 3] = [
    PythonMethodDesc {
        name: "greet",
        wire_raw: false,
    },
    PythonMethodDesc {
        name: "double",
        wire_raw: false,
    },
    PythonMethodDesc {
        name: "reverse_bytes",
        wire_raw: true,
    },
];

const ERROR_METHODS: [PythonMethodDesc; 1] = [PythonMethodDesc {
    name: "boom",
    wire_raw: false,
}];

/// Make a `'static` interface descriptor with a unique name so each test
/// gets its own slot in Python's `sys.modules` cache. Tests share the
/// embedded interpreter, so a fixed module name across tests would mean
/// the second-and-later loads silently reuse the first test's module
/// (the shared-`sys.modules` constraint we ship as a documented feature).
fn fresh_descriptor(
    methods: &'static [PythonMethodDesc],
) -> (&'static PythonInterfaceDescriptor, String) {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let name = format!("greeter_t{id}");
    let leaked_name: &'static str = Box::leak(name.clone().into_boxed_str());
    let desc = Box::leak(Box::new(PythonInterfaceDescriptor {
        interface_name: leaked_name,
        interface_hash: HASH,
        methods,
    }));
    (desc, name)
}

/// Stand up a minimal Python plugin package on disk:
/// - manifest declaring runtime=python with the supplied entry_module
/// - <entry_module>.py implementing the three test methods (or `boom` for
///   the error scenario)
/// - vendored fidius SDK so plugin code can `from fidius import method, PluginError`
fn make_plugin(
    tmp: &tempfile::TempDir,
    entry_module: &str,
    declared_hash: u64,
    methods_source: &str,
) -> PathBuf {
    let dir = tmp.path().to_path_buf();

    std::fs::write(
        dir.join("package.toml"),
        format!(
            r#"
[package]
name = "{entry_module}-py"
version = "0.1.0"
interface = "greeter"
interface_version = 1
runtime = "python"

[metadata]
category = "test"

[python]
entry_module = "{entry_module}"
"#
        ),
    )
    .unwrap();

    // Vendor the in-tree fidius SDK so the plugin can import it.
    let sdk_src = repo_root().join("python/fidius");
    let vendor = dir.join("vendor");
    std::fs::create_dir_all(&vendor).unwrap();
    copy_dir(&sdk_src, &vendor.join("fidius"));

    std::fs::write(
        dir.join(format!("{entry_module}.py")),
        format!(
            r#"
from fidius import method, PluginError

__interface_hash__ = {hash}

{methods_source}
"#,
            hash = declared_hash,
            methods_source = methods_source,
        ),
    )
    .unwrap();

    dir
}

const GREETER_METHODS_SRC: &str = r#"
@method
def greet(name):
    return f"Hello, {name}!"

@method
def double(payload):
    return {"name": payload["name"], "twice": payload["count"] * 2}

@method
def reverse_bytes(data):
    return bytes(reversed(data))
"#;

const ERROR_METHODS_SRC: &str = r#"
@method
def boom(arg):
    raise PluginError("BAD_INPUT", "deliberate failure", details={"got": arg})
"#;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn copy_dir(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

fn load_greeter() -> (tempfile::TempDir, fidius_python::PythonPluginHandle) {
    let tmp = tempfile::TempDir::new().unwrap();
    let (desc, mod_name) = fresh_descriptor(&GREETER_METHODS);
    let dir = make_plugin(&tmp, &mod_name, HASH, GREETER_METHODS_SRC);
    let handle = load_python_plugin(&dir, desc).expect("load");
    (tmp, handle)
}

#[test]
fn typed_call_round_trip_string() {
    let (_tmp, handle) = load_greeter();
    let input = serde_json::to_vec(&("World".to_string(),)).unwrap();
    let out = handle.call_typed_json(0, &input).expect("greet");
    let result: String = serde_json::from_slice(&out).unwrap();
    assert_eq!(result, "Hello, World!");
}

#[test]
fn typed_call_with_struct_args() {
    let (_tmp, handle) = load_greeter();

    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct DoubleIn {
        name: String,
        count: i64,
    }
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct DoubleOut {
        name: String,
        twice: i64,
    }

    let input = serde_json::to_vec(&(DoubleIn {
        name: "alpha".to_string(),
        count: 5,
    },))
    .unwrap();
    let out = handle.call_typed_json(1, &input).expect("double");
    let parsed: DoubleOut = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        parsed,
        DoubleOut {
            name: "alpha".to_string(),
            twice: 10,
        }
    );
}

#[test]
fn raw_call_round_trip_2mb() {
    let (_tmp, handle) = load_greeter();

    let payload: Vec<u8> = (0..(2 * 1024 * 1024u32))
        .map(|i| (i & 0xFF) as u8)
        .collect();
    let result = handle.call_raw(2, &payload).expect("reverse_bytes");
    assert_eq!(result.len(), payload.len());
    assert_eq!(result.first(), payload.last());
    assert_eq!(result.last(), payload.first());
}

#[test]
fn plugin_error_round_trips_with_code_and_details() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (desc, mod_name) = fresh_descriptor(&ERROR_METHODS);
    let dir = make_plugin(&tmp, &mod_name, HASH, ERROR_METHODS_SRC);
    let handle = load_python_plugin(&dir, desc).expect("load");

    let input = serde_json::to_vec(&(42i64,)).unwrap();
    let err = handle.call_typed_json(0, &input).unwrap_err();
    match err {
        PythonCallError::Plugin(pe) => {
            // Today fidius.PluginError raises map under the generic Python
            // error path: code = "PluginError" (the class name) and the
            // details JSON object preserves a traceback. Flattening
            // `pe.code`/`pe.details` from the raised PluginError into the
            // host-side PluginError fields is a follow-on enhancement to
            // pyerr_to_plugin_error (T-0086 hand-off).
            assert_eq!(pe.code, "PluginError");
            assert!(pe.message.contains("deliberate failure"));
            let details = pe.details.expect("details");
            assert!(
                details.contains("traceback"),
                "expected traceback in details, got: {details}"
            );
        }
        other => panic!("expected Plugin error, got {other:?}"),
    }
}

#[test]
fn interface_hash_mismatch_is_rejected() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (desc, mod_name) = fresh_descriptor(&GREETER_METHODS);
    let dir = make_plugin(&tmp, &mod_name, 0xBAD_BAD_BAD_BAD, GREETER_METHODS_SRC);
    let err = load_python_plugin(&dir, desc).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("interface hash mismatch"),
        "expected hash mismatch, got: {msg}"
    );
}

#[test]
fn wire_mode_mismatch_typed_called_as_raw_errors() {
    let (_tmp, handle) = load_greeter();
    // greet (index 0) is typed; calling it via call_raw must error.
    let err = handle.call_raw(0, b"oops").unwrap_err();
    assert!(matches!(err, PythonCallError::WireModeMismatch { .. }));
}

#[test]
fn out_of_range_method_index_errors() {
    let (_tmp, handle) = load_greeter();
    let err = handle.call_typed_json(99, b"[]").unwrap_err();
    assert!(matches!(err, PythonCallError::InvalidMethodIndex { .. }));
}
