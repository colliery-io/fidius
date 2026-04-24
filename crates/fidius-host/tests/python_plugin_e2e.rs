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

//! Capstone integration test: a real Python plugin package
//! (`tests/test-plugin-py-greeter`) implementing the `BytePipe` trait
//! defined in Rust (`tests/test-plugin-smoke/src/lib.rs`), loaded and
//! exercised through the standard `PluginHost` API.
//!
//! What this test proves end-to-end:
//!
//! 1. A Python author shipping `.py` + `package.toml` + `vendor/` (no Rust
//!    toolchain involved) produces a plugin the host loads transparently.
//! 2. `PluginHost::discover()` surfaces the python package alongside any
//!    cdylib plugins, with `PluginInfo.runtime = Python`.
//! 3. The hash compiled into `__interface_hash__` (generated via
//!    `fidius python-stub`) matches the runtime constant the macro
//!    produces — so the contract loop is closed end-to-end.
//! 4. Both the `#[wire(raw)]` `reverse` method and the typed `name`
//!    method round-trip correctly.
//! 5. A tampered `__interface_hash__` is rejected at load with a clear error.

#![cfg(feature = "python")]

use std::path::PathBuf;

use fidius_core::python_descriptor::PythonInterfaceDescriptor;
use fidius_host::{PluginHost, PluginRuntimeKind};

/// Directory structure mirrors what a deployer would have:
///
/// ```
/// plugins/
/// └── py-byte-pipe/
///     ├── package.toml
///     ├── byte_pipe.py
///     └── vendor/
///         └── fidius/
///             └── ...
/// ```
fn stage_plugin(tmp: &tempfile::TempDir) -> PathBuf {
    let plugins_root = tmp.path().to_path_buf();
    let dest = plugins_root.join("py-byte-pipe");
    let src = repo_root().join("tests/test-plugin-py-greeter");

    copy_dir(&src, &dest);

    // Vendor the in-tree fidius SDK into the staged copy. In production
    // the deployer would `pip install fidius --target vendor/` once and
    // commit the result; here we just snapshot the source tree.
    let vendor = dest.join("vendor");
    std::fs::create_dir_all(&vendor).unwrap();
    copy_dir(&repo_root().join("python/fidius"), &vendor.join("fidius"));

    plugins_root
}

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

/// Produce the BytePipe descriptor from the Rust trait via the macro-emitted
/// companion module — the same hash the Python plugin's `__interface_hash__`
/// is derived from. Wrapping in a fn so the test gets a `'static` reference
/// without exporting it from the test plugin crate.
fn byte_pipe_descriptor() -> &'static PythonInterfaceDescriptor {
    &test_plugin_smoke::__fidius_BytePipe::BytePipe_PYTHON_DESCRIPTOR
}

#[test]
fn discover_lists_python_plugin_with_python_runtime() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plugins = stage_plugin(&tmp);

    let host = PluginHost::builder().search_path(&plugins).build().unwrap();
    let infos = host.discover().unwrap();

    let info = infos
        .iter()
        .find(|i| i.name == "py-byte-pipe")
        .expect("py-byte-pipe should appear in discovery results");
    assert!(matches!(info.runtime, PluginRuntimeKind::Python));
    assert_eq!(info.interface_name, "BytePipe");
    assert_eq!(info.interface_version, 1);
}

#[test]
fn typed_method_round_trips() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plugins = stage_plugin(&tmp);
    let host = PluginHost::builder().search_path(&plugins).build().unwrap();

    let handle = host
        .load_python("py-byte-pipe", byte_pipe_descriptor())
        .expect("load_python should succeed");

    // BytePipe.name has index 1 (reverse is 0). Typed call: zero-arg
    // method, encoded as the empty tuple (= JSON `[]`).
    let input = serde_json::to_vec(&()).unwrap();
    let out = handle.call_typed_json(1, &input).expect("name");
    let result: String = serde_json::from_slice(&out).unwrap();
    assert_eq!(result, "py-byte-pipe");
}

#[test]
fn raw_wire_method_round_trips_2mb() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plugins = stage_plugin(&tmp);
    let host = PluginHost::builder().search_path(&plugins).build().unwrap();

    let handle = host
        .load_python("py-byte-pipe", byte_pipe_descriptor())
        .expect("load_python should succeed");

    let payload: Vec<u8> = (0..(2 * 1024 * 1024u32))
        .map(|i| (i & 0xFF) as u8)
        .collect();
    let result = handle
        .call_raw(0, &payload)
        .expect("reverse_bytes should round-trip");

    assert_eq!(result.len(), payload.len());
    assert_eq!(result.first(), payload.last());
    assert_eq!(result.last(), payload.first());
}

#[test]
fn tampered_interface_hash_is_rejected_at_load() {
    // Rename the entry module to dodge the shared-`sys.modules` cache —
    // earlier tests may have already imported `byte_pipe` so a re-import
    // would silently reuse it. This is exactly the deployment constraint
    // we document for production: each fresh load needs a fresh module
    // name within one host process.
    let tmp = tempfile::TempDir::new().unwrap();
    let plugins = stage_plugin(&tmp);
    let pkg_dir = plugins.join("py-byte-pipe");

    // Rename the .py and the manifest's entry_module to a unique name.
    let original = pkg_dir.join("byte_pipe.py");
    let tampered_path = pkg_dir.join("byte_pipe_tampered.py");
    let src = std::fs::read_to_string(&original).unwrap();
    let tampered = src.replace("0xDF233D1A5936EB5C", "0xBADBADBADBADBADB");
    std::fs::write(&tampered_path, tampered).unwrap();
    std::fs::remove_file(&original).unwrap();

    let manifest_path = pkg_dir.join("package.toml");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    let manifest = manifest.replace("byte_pipe", "byte_pipe_tampered");
    std::fs::write(&manifest_path, manifest).unwrap();

    let host = PluginHost::builder().search_path(&plugins).build().unwrap();
    let err = host
        .load_python("py-byte-pipe", byte_pipe_descriptor())
        .unwrap_err();

    let msg = format!("{err}");
    assert!(
        msg.contains("interface hash mismatch") || msg.contains("python load failed"),
        "expected hash-mismatch error, got: {msg}"
    );
}
