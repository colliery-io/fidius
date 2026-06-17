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

// ── Pack / precompile (FIDIUS-T-0107) ───────────────────────────────────────

/// Record `precompiled = "<name>"` under `[wasm]` in a staged package.toml.
fn set_precompiled(pkg_dir: &std::path::Path, cwasm: &str) {
    let p = pkg_dir.join("package.toml");
    let content = std::fs::read_to_string(&p).unwrap();
    let pos = content.find("[wasm]").unwrap();
    let after = pos + "[wasm]".len();
    let line_end = content[after..].find('\n').map(|i| after + i + 1).unwrap();
    let mut out = content[..line_end].to_string();
    out.push_str(&format!("precompiled = \"{cwasm}\"\n"));
    out.push_str(&content[line_end..]);
    std::fs::write(&p, out).unwrap();
}

#[test]
fn precompiled_cwasm_loads_via_aot_and_calls() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_wasm_package(tmp.path(), &[]);
    let dir = tmp.path().join("greeter-pkg");

    let cwasm = fidius_host::executor::precompile_component(greeter_component())
        .expect("precompile greeter");
    std::fs::write(dir.join("greeter_guest.cwasm"), &cwasm).unwrap();
    set_precompiled(&dir, "greeter_guest.cwasm");

    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm("greeter-pkg", &GREETER_DESC)
        .expect("AOT load");
    let s: String = handle.call_method(0, &("Zed".to_string(),)).unwrap();
    assert_eq!(s, "Hello, Zed!");
}

#[test]
fn stale_cwasm_falls_back_to_jit() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_wasm_package(tmp.path(), &[]);
    let dir = tmp.path().join("greeter-pkg");

    // A garbage .cwasm (wrong engine/version) must be ignored, not fatal.
    std::fs::write(dir.join("greeter_guest.cwasm"), b"not a real cwasm header").unwrap();
    set_precompiled(&dir, "greeter_guest.cwasm");

    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm("greeter-pkg", &GREETER_DESC)
        .expect("stale .cwasm should fall back to JIT, not fail");
    let s: String = handle.call_method(0, &("Q".to_string(),)).unwrap();
    assert_eq!(s, "Hello, Q!");
}

#[test]
fn pack_unpack_load_roundtrip() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_wasm_package(tmp.path(), &[]);
    let dir = tmp.path().join("greeter-pkg");

    let fid = tmp.path().join("greeter.fid");
    fidius_core::package::pack_package(&dir, Some(&fid)).expect("pack");

    let unpacked = tmp.path().join("unpacked");
    std::fs::create_dir_all(&unpacked).unwrap();
    let pkg_dir = fidius_core::package::unpack_package(&fid, &unpacked).expect("unpack");
    assert!(pkg_dir.join("package.toml").exists());
    assert!(pkg_dir.join("greeter_guest.wasm").exists());

    let host = PluginHost::builder()
        .search_path(&unpacked)
        .build()
        .unwrap();
    let handle = host
        .load_wasm("greeter-pkg", &GREETER_DESC)
        .expect("load from unpacked .fid");
    let s: String = handle.call_method(0, &("Pax".to_string(),)).unwrap();
    assert_eq!(s, "Hello, Pax!");
}

// ── Signature policy for wasm packages (FIDIUS-T-0108) ──────────────────────

/// Sign a staged package dir over its `package_digest` (the same scheme
/// `fidius package sign` uses) and return the verifying key.
fn sign_pkg(pkg_dir: &std::path::Path) -> ed25519_dalek::VerifyingKey {
    use ed25519_dalek::Signer;
    let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
    let digest = fidius_core::package::package_digest(pkg_dir).unwrap();
    let sig = sk.sign(&digest);
    std::fs::write(pkg_dir.join("package.sig"), sig.to_bytes()).unwrap();
    sk.verifying_key()
}

#[test]
fn signed_wasm_package_loads_when_signature_required() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_wasm_package(tmp.path(), &[]);
    let vk = sign_pkg(&tmp.path().join("greeter-pkg"));

    let host = PluginHost::builder()
        .search_path(tmp.path())
        .require_signature(true)
        .trusted_keys(&[vk])
        .build()
        .unwrap();
    let handle = host
        .load_wasm("greeter-pkg", &GREETER_DESC)
        .expect("signed package should load");
    let s: String = handle.call_method(0, &("Sig".to_string(),)).unwrap();
    assert_eq!(s, "Hello, Sig!");
}

#[test]
fn unsigned_wasm_package_rejected_when_signature_required() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_wasm_package(tmp.path(), &[]); // no package.sig
    let vk = {
        use ed25519_dalek::SigningKey;
        SigningKey::from_bytes(&[7u8; 32]).verifying_key()
    };

    let host = PluginHost::builder()
        .search_path(tmp.path())
        .require_signature(true)
        .trusted_keys(&[vk])
        .build()
        .unwrap();
    match host.load_wasm("greeter-pkg", &GREETER_DESC) {
        Err(LoadError::SignatureRequired { .. }) => {}
        Err(e) => panic!("expected SignatureRequired, got {e:?}"),
        Ok(_) => panic!("expected SignatureRequired, got a handle"),
    }
}

#[test]
fn tampered_wasm_package_fails_verification() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_wasm_package(tmp.path(), &[]);
    let dir = tmp.path().join("greeter-pkg");
    let vk = sign_pkg(&dir);

    // Tamper the component after signing → package_digest changes.
    let comp = dir.join("greeter_guest.wasm");
    let mut bytes = std::fs::read(&comp).unwrap();
    bytes.push(0);
    std::fs::write(&comp, &bytes).unwrap();

    let host = PluginHost::builder()
        .search_path(tmp.path())
        .require_signature(true)
        .trusted_keys(&[vk])
        .build()
        .unwrap();
    match host.load_wasm("greeter-pkg", &GREETER_DESC) {
        Err(LoadError::SignatureInvalid { .. }) => {}
        Err(e) => panic!("expected SignatureInvalid after tampering, got {e:?}"),
        Ok(_) => panic!("expected SignatureInvalid after tampering, got a handle"),
    }
}

// ── Polyglot: JavaScript guest (FIDIUS-I-0025) ──────────────────────────────

fn js_greeter_component() -> Option<Vec<u8>> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/wasm-fixtures/greeter-js/greeter_js.wasm");
    std::fs::read(p).ok()
}

/// A JavaScript guest (jco/ComponentizeJS) implementing the SAME `greeter` WIT
/// loads and is called through the identical `PluginHost::load_wasm` path as the
/// Rust and Python guests — the third language, same host, same descriptor.
#[test]
fn polyglot_js_guest_behaves_identically() {
    let Some(bytes) = js_greeter_component() else {
        eprintln!(
            "SKIP polyglot_js_guest: greeter_js.wasm not built \
             (run tests/wasm-fixtures/greeter-js/build.sh)"
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
component = "greeter_js.wasm"
"#,
    )
    .unwrap();
    std::fs::write(dir.join("greeter_js.wasm"), bytes).unwrap();

    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm("greeter-pkg", &GREETER_DESC)
        .expect("load JS component (interface hash must match the Rust/Python guests)");

    // Identical results to the Rust + Python guests, through the identical API.
    let s: String = handle.call_method(0, &("Ada".to_string(),)).unwrap();
    assert_eq!(s, "Hello, Ada!");
    let sum: i64 = handle.call_method(1, &(2i64, 3i64)).unwrap();
    assert_eq!(sum, 5);
    let rev = handle.call_method_raw(2, b"xyz").unwrap();
    assert_eq!(rev, b"zyx");
}

// ── Polyglot: Go guest (TinyGo, FIDIUS-I-0025) ──────────────────────────────

fn go_greeter_component() -> Option<Vec<u8>> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/wasm-fixtures/greeter-go/greeter_go.wasm");
    std::fs::read(p).ok()
}

/// A Go guest (TinyGo + wit-bindgen-go) implementing the SAME `greeter` WIT loads
/// and is called through the identical host path as the Rust, Python, and
/// JavaScript guests — a fourth language, same host, same descriptor.
#[test]
fn polyglot_go_guest_behaves_identically() {
    let Some(bytes) = go_greeter_component() else {
        eprintln!(
            "SKIP polyglot_go_guest: greeter_go.wasm not built \
             (run tests/wasm-fixtures/greeter-go/build.sh)"
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
component = "greeter_go.wasm"
"#,
    )
    .unwrap();
    std::fs::write(dir.join("greeter_go.wasm"), bytes).unwrap();

    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm("greeter-pkg", &GREETER_DESC)
        .expect("load Go component (interface hash must match the other guests)");

    let s: String = handle.call_method(0, &("Ada".to_string(),)).unwrap();
    assert_eq!(s, "Hello, Ada!");
    let sum: i64 = handle.call_method(1, &(2i64, 3i64)).unwrap();
    assert_eq!(sum, 5);
    let rev = handle.call_method_raw(2, b"xyz").unwrap();
    assert_eq!(rev, b"zyx");
}
