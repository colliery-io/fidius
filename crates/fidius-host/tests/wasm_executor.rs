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

//! Round-trip tests for the WASM component backend (FIDIUS-T-0102). Builds the
//! reference greeter component (tests/wasm-fixtures/greeter) via cargo-component
//! and drives it through `WasmComponentExecutor` directly. Runs only with
//! `--features wasm` and requires the component toolchain (FIDIUS-T-0094).

#![cfg(feature = "wasm")]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use fidius_core::descriptor::BufferStrategyKind;
use fidius_core::wasm_descriptor::{WasmInterfaceDescriptor, WasmMethodDesc};
use fidius_core::{from_value, to_value};
use fidius_host::executor::{PluginExecutor, ValueExecutor, WasmComponentExecutor, WasmMethod};
use fidius_host::{CallError, LoadError, PluginHost, PluginInfo, PluginRuntimeKind};

const IFACE: &str = "fidius:greeter/greeter@1.0.0";
const EXPECTED_HASH: u64 = 0x0102_0304_0506_0708;

/// Build the greeter component once (process-wide cache) and return its bytes.
fn greeter_component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/wasm-fixtures/greeter");
        let status = Command::new("cargo")
            .args(["component", "build", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("run `cargo component build` (is cargo-component installed? see T-0094)");
        assert!(status.success(), "cargo component build failed");
        let art = fixture.join("target/wasm32-wasip1/release/greeter_guest.wasm");
        std::fs::read(&art).unwrap_or_else(|e| panic!("read {}: {e}", art.display()))
    })
}

fn executor_with(capabilities: Vec<String>) -> WasmComponentExecutor {
    let info = PluginInfo {
        name: "greeter".to_string(),
        interface_name: "greeter".to_string(),
        interface_hash: EXPECTED_HASH,
        interface_version: 1,
        capabilities: 0,
        buffer_strategy: BufferStrategyKind::PluginAllocated,
        runtime: PluginRuntimeKind::Wasm,
    };
    let methods = vec![
        WasmMethod {
            name: "greet".to_string(),
            wire_raw: false,
        },
        WasmMethod {
            name: "add".to_string(),
            wire_raw: false,
        },
        WasmMethod {
            name: "echo-bytes".to_string(),
            wire_raw: true,
        },
        WasmMethod {
            name: "probe-env".to_string(),
            wire_raw: false,
        },
    ];
    WasmComponentExecutor::from_component_bytes(
        greeter_component(),
        IFACE.to_string(),
        methods,
        capabilities,
        info,
    )
    .expect("build executor")
}

fn executor() -> WasmComponentExecutor {
    executor_with(vec![])
}

#[test]
fn interface_hash_matches() {
    assert_eq!(executor().interface_hash().unwrap(), EXPECTED_HASH);
}

#[test]
fn typed_call_greet() {
    let exec = executor();
    let out = exec
        .call(0, to_value(&("World".to_string(),)).unwrap())
        .unwrap();
    let s: String = from_value(out).unwrap();
    assert_eq!(s, "Hello, World!");
}

#[test]
fn typed_call_add_ok_and_err() {
    let exec = executor();
    // Ok arm: result<s64, _> unwrapped to the inner value.
    let out = exec.call(1, to_value(&(2i64, 3i64)).unwrap()).unwrap();
    let n: i64 = from_value(out).unwrap();
    assert_eq!(n, 5);

    // Err arm (overflow) → CallError::Plugin with the guest's code/message.
    let err = exec
        .call(1, to_value(&(i64::MAX, 1i64)).unwrap())
        .unwrap_err();
    match err {
        CallError::Plugin(e) => assert_eq!(e.code, "overflow"),
        other => panic!("expected CallError::Plugin, got {other:?}"),
    }
}

#[test]
fn raw_call_echo_bytes_reverses() {
    let exec = executor();
    let out = exec.call_raw(2, b"abcdef").unwrap();
    assert_eq!(out, b"fedcba");
}

#[test]
fn method_count_and_info() {
    let exec = executor();
    assert_eq!(exec.method_count(), 4);
    assert_eq!(exec.info().runtime, PluginRuntimeKind::Wasm);
}

// ── Load through PluginHost (FIDIUS-T-0103) ────────────────────────────────

static METHOD_DESCS: [WasmMethodDesc; 4] = [
    WasmMethodDesc {
        name: "greet",
        wire_raw: false,
    },
    WasmMethodDesc {
        name: "add",
        wire_raw: false,
    },
    WasmMethodDesc {
        name: "echo-bytes",
        wire_raw: true,
    },
    WasmMethodDesc {
        name: "probe-env",
        wire_raw: false,
    },
];
static GREETER_DESC: WasmInterfaceDescriptor = WasmInterfaceDescriptor {
    interface_name: "greeter",
    interface_export: IFACE,
    interface_hash: EXPECTED_HASH,
    methods: &METHOD_DESCS,
};

/// Stage a `runtime = "wasm"` package directory containing the built component,
/// with the given `[wasm].capabilities` allow-list.
fn stage_wasm_package(root: &std::path::Path, capabilities: &[&str]) {
    let dir = root.join("greeter-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    let caps = if capabilities.is_empty() {
        String::new()
    } else {
        let list = capabilities
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!("capabilities = [{list}]\n")
    };
    std::fs::write(
        dir.join("package.toml"),
        format!(
            r#"
[package]
name = "greeter-pkg"
version = "0.1.0"
interface = "greeter"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "greeter_guest.wasm"
{caps}"#
        ),
    )
    .unwrap();
    std::fs::write(dir.join("greeter_guest.wasm"), greeter_component()).unwrap();
}

#[test]
fn load_wasm_through_host_and_call() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_wasm_package(tmp.path(), &[]);
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();

    let handle = host
        .load_wasm("greeter-pkg", &GREETER_DESC)
        .expect("load_wasm");

    // Typed call through the unified PluginHandle.
    let s: String = handle.call_method(0, &("Ada".to_string(),)).unwrap();
    assert_eq!(s, "Hello, Ada!");
    // Raw call through the unified PluginHandle.
    let rev = handle.call_method_raw(2, b"xyz").unwrap();
    assert_eq!(rev, b"zyx");
    assert_eq!(handle.info().runtime, PluginRuntimeKind::Wasm);
}

#[test]
fn load_wasm_rejects_interface_hash_mismatch() {
    static BAD_DESC: WasmInterfaceDescriptor = WasmInterfaceDescriptor {
        interface_name: "greeter",
        interface_export: IFACE,
        interface_hash: 0xDEAD_BEEF_DEAD_BEEF, // wrong on purpose
        methods: &METHOD_DESCS,
    };
    let tmp = tempfile::TempDir::new().unwrap();
    stage_wasm_package(tmp.path(), &[]);
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();

    let err = match host.load_wasm("greeter-pkg", &BAD_DESC) {
        Ok(_) => panic!("expected interface-hash mismatch to reject the load"),
        Err(e) => e,
    };
    assert!(
        matches!(err, LoadError::InterfaceHashMismatch { .. }),
        "expected InterfaceHashMismatch, got {err:?}"
    );
}

#[test]
fn discover_surfaces_wasm_package() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_wasm_package(tmp.path(), &[]);
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let infos = host.discover().unwrap();
    let w = infos
        .iter()
        .find(|i| i.name == "greeter-pkg")
        .expect("wasm package in discovery");
    assert!(w.is_wasm());
}

// ── Capability policy (FIDIUS-T-0104) ──────────────────────────────────────

const PROBE_ENV: usize = 3;

#[test]
fn env_capability_denied_by_default() {
    // Even with the env var set in the host process, a deny-all package
    // (no `[wasm].capabilities`) cannot see it.
    std::env::set_var("FIDIUS_TEST_CAP", "1");
    let tmp = tempfile::TempDir::new().unwrap();
    stage_wasm_package(tmp.path(), &[]);
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host.load_wasm("greeter-pkg", &GREETER_DESC).unwrap();
    let visible: bool = handle.call_method(PROBE_ENV, &()).unwrap();
    assert!(!visible, "env must be denied without the `env` capability");
}

#[test]
fn env_capability_granted_via_allowlist() {
    std::env::set_var("FIDIUS_TEST_CAP", "1");
    let tmp = tempfile::TempDir::new().unwrap();
    stage_wasm_package(tmp.path(), &["env"]);
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host.load_wasm("greeter-pkg", &GREETER_DESC).unwrap();
    let visible: bool = handle.call_method(PROBE_ENV, &()).unwrap();
    assert!(
        visible,
        "env must be visible once the `env` capability is granted"
    );
}

// ── Polyglot proof (FIDIUS-T-0105) ─────────────────────────────────────────

/// The Python-authored component, if it's been built (see
/// `tests/wasm-fixtures/greeter-py/build.sh`). Returns `None` when absent so
/// the test skips cleanly where `componentize-py` isn't available (e.g. CI
/// without the toolchain) rather than failing.
fn python_greeter_component() -> Option<Vec<u8>> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/wasm-fixtures/greeter-py/greeter_py.wasm");
    std::fs::read(p).ok()
}

/// A Python guest implementing the SAME `greeter` WIT is loaded and called
/// through the identical `PluginHost::load_wasm` path as the Rust guest — the
/// concrete evidence that Path B delivers language-agnostic plugins.
#[test]
fn polyglot_python_guest_behaves_identically() {
    let Some(bytes) = python_greeter_component() else {
        eprintln!(
            "SKIP polyglot_python_guest: greeter_py.wasm not built \
             (run tests/wasm-fixtures/greeter-py/build.sh)"
        );
        return;
    };

    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path().join("greeter-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        r#"
[package]
name = "greeter-pkg"
version = "0.1.0"
interface = "greeter"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "greeter_py.wasm"
capabilities = ["env"]
"#,
    )
    .unwrap();
    std::fs::write(dir.join("greeter_py.wasm"), bytes).unwrap();

    std::env::set_var("FIDIUS_TEST_CAP", "1");
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm("greeter-pkg", &GREETER_DESC)
        .expect("load python component (interface hash must match the Rust guest)");

    // Identical results to the Rust guest, through the identical host API.
    let s: String = handle.call_method(0, &("Ada".to_string(),)).unwrap();
    assert_eq!(s, "Hello, Ada!");
    let rev = handle.call_method_raw(2, b"xyz").unwrap();
    assert_eq!(rev, b"zyx");
    let env_visible: bool = handle.call_method(PROBE_ENV, &()).unwrap();
    assert!(env_visible, "env granted → visible in the Python guest too");
}

#[test]
fn unknown_capability_rejected_at_load() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_wasm_package(tmp.path(), &["filesystem"]); // never grantable
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let err = match host.load_wasm("greeter-pkg", &GREETER_DESC) {
        Ok(_) => panic!("expected unknown capability to be rejected"),
        Err(e) => e,
    };
    assert!(
        matches!(err, LoadError::WasmLoad(msg) if msg.contains("unknown wasm capability")),
        "expected a clear unknown-capability error",
    );
}
