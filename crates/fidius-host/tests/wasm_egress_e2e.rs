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

//! Capability-gated `wasi:http` egress E2E (FIDIUS-I-0027 / E2). A real WASM
//! component (`tests/wasm-fixtures/fetcher`) imports `wasi:http/outgoing-handler`
//! and makes an outbound GET. The host enforces the **two-key** gate and routes
//! every request through the embedder's `EgressPolicy`:
//!   - allowed  → the guest fetches the mock server's body;
//!   - denied   → the policy refuses before dispatch; the guest gets an error;
//!   - no policy / no capability → `wasi:http` isn't linked → fails closed.
//!
//! The reference allow/deny policies here are exactly what the docs say an
//! embedder writes — fidius ships none of this (mechanism, not policy).

#![cfg(feature = "wasm")]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;

use fidius_core::descriptor::BufferStrategyKind;
use fidius_core::wasm_descriptor::{WasmInterfaceDescriptor, WasmMethodDesc};
use fidius_host::executor::{EgressDenied, EgressPolicy, WasmComponentExecutor, WasmMethod};
use fidius_host::{CallError, LoadError, PluginHandle, PluginHost, PluginInfo, PluginRuntimeKind};

const IFACE: &str = "fidius:fetcher/fetcher@1.0.0";

fn fetcher_component() -> Option<Vec<u8>> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/wasm-fixtures/fetcher/fetcher_guest.wasm");
    std::fs::read(p).ok()
}

/// One-shot mock HTTP server on an ephemeral loopback port; serves a single
/// request with `body`. Returns the base URL + the server thread handle.
fn mock_http_once(body: &'static str) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{addr}/");
    let h = std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 2048];
            let _ = stream.read(&mut buf); // consume the request line + headers
            let resp = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });
    (url, h)
}

/// Reference embedder policy: allow everything (the test's loopback grant).
struct AllowAll;
impl EgressPolicy for AllowAll {
    fn authorize(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
        Ok(())
    }
}

/// Reference embedder policy: deny everything.
struct DenyAll;
impl EgressPolicy for DenyAll {
    fn authorize(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
        Err(EgressDenied::new("denied by test policy"))
    }
}

fn load(
    caps: Vec<String>,
    egress: Option<Arc<dyn EgressPolicy>>,
) -> Result<PluginHandle, CallError> {
    let bytes = fetcher_component().expect("fetcher_guest.wasm present");
    let info = PluginInfo {
        name: "fetcher".into(),
        interface_name: "fetcher".into(),
        interface_hash: 0,
        interface_version: 1,
        capabilities: 0,
        buffer_strategy: BufferStrategyKind::PluginAllocated,
        runtime: PluginRuntimeKind::Wasm,
    };
    let methods = vec![WasmMethod {
        name: "fetch".into(),
        wire_raw: false,
        streaming: false,
    }];
    WasmComponentExecutor::from_component_bytes_with_egress(
        &bytes,
        IFACE.into(),
        methods,
        caps,
        egress,
        info,
    )
    .map(PluginHandle::from_wasm)
}

#[test]
fn egress_allowed_fetches_body() {
    if fetcher_component().is_none() {
        eprintln!("SKIP egress_allowed_fetches_body: fetcher_guest.wasm not built");
        return;
    }
    let (url, server) = mock_http_once("hello from mock");
    let handle = load(vec!["http".into()], Some(Arc::new(AllowAll))).expect("load");
    let body: String = handle.call_method(0, &(url,)).expect("fetch");
    server.join().unwrap();
    assert_eq!(body, "hello from mock");
}

#[test]
fn egress_denied_by_policy() {
    if fetcher_component().is_none() {
        eprintln!("SKIP egress_denied_by_policy");
        return;
    }
    // The policy refuses before dispatch, so no server is needed.
    let handle = load(vec!["http".into()], Some(Arc::new(DenyAll))).expect("load");
    let body: String = handle
        .call_method(0, &("http://127.0.0.1:1/".to_string(),))
        .expect("fetch");
    assert!(
        body.starts_with("ERROR:"),
        "expected a denied egress error, got: {body}"
    );
}

#[test]
fn no_policy_fails_closed() {
    if fetcher_component().is_none() {
        eprintln!("SKIP no_policy_fails_closed");
        return;
    }
    // `http` capability declared but NO egress policy → `wasi:http` isn't linked
    // → the component (which imports it) fails to instantiate at load.
    let res = load(vec!["http".into()], None);
    assert!(
        matches!(res, Err(CallError::Backend { .. })),
        "must fail closed with a Backend error"
    );
}

#[test]
fn no_capability_fails_closed() {
    if fetcher_component().is_none() {
        eprintln!("SKIP no_capability_fails_closed");
        return;
    }
    // Policy supplied but the package didn't declare `http` → not linked → closed.
    let res = load(vec![], Some(Arc::new(AllowAll)));
    assert!(
        matches!(res, Err(CallError::Backend { .. })),
        "must fail closed with a Backend error"
    );
}

// ── Ergonomic path: egress through `PluginHost::load_wasm` (not the low-level
// executor constructor). The fetcher fixture carries `fidius-interface-hash`, so
// it loads as a real package.

static FETCHER_METHODS: [WasmMethodDesc; 1] = [WasmMethodDesc {
    name: "fetch",
    wire_raw: false,
    streaming: false,
}];
static FETCHER: WasmInterfaceDescriptor = WasmInterfaceDescriptor {
    interface_name: "fetcher",
    interface_export: IFACE,
    interface_hash: 0xFE7C_4E20_0000_0001,
    methods: &FETCHER_METHODS,
};

/// Stage the fetcher as a loadable wasm package declaring the `http` capability.
fn stage_fetcher_pkg(root: &std::path::Path) {
    let dir = root.join("fetcher-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("fetcher_guest.wasm"), fetcher_component().unwrap()).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        "[package]\nname = \"fetcher-pkg\"\nversion = \"0.1.0\"\ninterface = \"fetcher\"\n\
         interface_version = 1\nruntime = \"wasm\"\n\n[metadata]\ncategory = \"test\"\n\n\
         [wasm]\ncomponent = \"fetcher_guest.wasm\"\ncapabilities = [\"http\"]\n",
    )
    .unwrap();
}

#[test]
fn egress_via_builder_default_policy() {
    if fetcher_component().is_none() {
        eprintln!("SKIP egress_via_builder_default_policy");
        return;
    }
    let (url, server) = mock_http_once("hello via builder");
    let tmp = tempfile::TempDir::new().unwrap();
    stage_fetcher_pkg(tmp.path());
    // Host-wide egress policy via the builder — no dropping to the executor.
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .egress(AllowAll)
        .build()
        .unwrap();
    let handle = host.load_wasm("fetcher-pkg", &FETCHER).expect("load_wasm");
    let body: String = handle.call_method(0, &(url,)).expect("fetch");
    server.join().unwrap();
    assert_eq!(body, "hello via builder");
}

#[test]
fn egress_via_per_plugin_policy() {
    if fetcher_component().is_none() {
        eprintln!("SKIP egress_via_per_plugin_policy");
        return;
    }
    let (url, server) = mock_http_once("hello per-plugin");
    let tmp = tempfile::TempDir::new().unwrap();
    stage_fetcher_pkg(tmp.path());
    // No host-wide policy; supplied per-load (the connector-isolation primitive).
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm_with_egress("fetcher-pkg", &FETCHER, AllowAll)
        .expect("load_wasm_with_egress");
    let body: String = handle.call_method(0, &(url,)).expect("fetch");
    server.join().unwrap();
    assert_eq!(body, "hello per-plugin");
}

#[test]
fn load_wasm_without_egress_fails_closed() {
    if fetcher_component().is_none() {
        eprintln!("SKIP load_wasm_without_egress_fails_closed");
        return;
    }
    let tmp = tempfile::TempDir::new().unwrap();
    stage_fetcher_pkg(tmp.path());
    // Package declares `http` but the host supplies NO policy → wasi:http unlinked
    // → the component (which imports it) fails to load.
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let res = host.load_wasm("fetcher-pkg", &FETCHER);
    assert!(
        matches!(res, Err(LoadError::WasmLoad(_))),
        "must fail closed without an egress policy"
    );
}

#[test]
fn egress_via_builder_arc_dyn_policy() {
    if fetcher_component().is_none() {
        eprintln!("SKIP egress_via_builder_arc_dyn_policy");
        return;
    }
    let (url, server) = mock_http_once("hello via arc");
    let tmp = tempfile::TempDir::new().unwrap();
    stage_fetcher_pkg(tmp.path());
    // An already-erased Arc<dyn EgressPolicy> (weir's case) — `.egress_policy(arc)`.
    let policy: Arc<dyn EgressPolicy> = Arc::new(AllowAll);
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .egress_policy(policy)
        .build()
        .unwrap();
    let handle = host.load_wasm("fetcher-pkg", &FETCHER).expect("load_wasm");
    let body: String = handle.call_method(0, &(url,)).expect("fetch");
    server.join().unwrap();
    assert_eq!(body, "hello via arc");
}
