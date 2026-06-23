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

//! Capability-gated `wasi:sockets` TCP egress E2E (FIDIUS-I-0033). A real WASM
//! component (`tests/wasm-fixtures/tcp-echo`) makes an outbound TCP connection via
//! `std::net::TcpStream` (which is `wasi:sockets` on `wasm32-wasip2`) and echoes
//! bytes through a loopback server. The host enforces the **two-key** gate and
//! routes every connect through the embedder's `EgressPolicy::authorize_tcp`:
//!   - allowed  → the guest connects and the bytes round-trip;
//!   - denied   → the policy refuses before connect; the guest gets an empty result;
//!   - no policy / no capability → no socket check is installed → the deny-all
//!     `WasiCtx` refuses the connect (fails closed at connect time — note this is
//!     runtime, not load time, since `wasi:sockets` is always linked, unlike the
//!     absent-import fail-closed of `wasi:http`).
//!
//! The reference allow/deny policies here are exactly what the docs say an
//! embedder writes for a DB connector — fidius ships none of this.

#![cfg(feature = "wasm")]

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, OnceLock};

use fidius_core::descriptor::BufferStrategyKind;
use fidius_core::wasm_descriptor::{WasmInterfaceDescriptor, WasmMethodDesc};
use fidius_host::executor::{EgressDenied, EgressPolicy, WasmComponentExecutor, WasmMethod};
use fidius_host::{CallError, PluginHandle, PluginHost, PluginInfo, PluginRuntimeKind};

const IFACE: &str = "fidius:tcp-echo/tcp-echo@1.0.0";

/// Build the tcp-echo component once (mirrors `macro_egress_e2e`): CI installs the
/// `wasm32-wasip2` target, so this runs the egress path for real there. If the
/// target/cargo isn't available (a dev without the wasm target), the build fails
/// and we return `None` so the tests skip rather than hard-fail.
fn tcp_echo_component() -> Option<Vec<u8>> {
    static BYTES: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    BYTES
        .get_or_init(|| {
            let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/wasm-fixtures/tcp-echo");
            let built = Command::new("cargo")
                .args(["build", "--target", "wasm32-wasip2", "--release"])
                .current_dir(&fixture)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !built {
                return None;
            }
            let art = fixture.join("target/wasm32-wasip2/release/tcp_echo_guest.wasm");
            std::fs::read(&art).ok()
        })
        .clone()
}

/// One-shot mock TCP echo server on an ephemeral loopback port: accepts one
/// connection, reads `n` bytes, writes them back. Returns `host:port` + handle.
fn mock_tcp_echo_once(n: usize) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let h = std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = vec![0u8; n];
            if stream.read_exact(&mut buf).is_ok() {
                let _ = stream.write_all(&buf);
                let _ = stream.flush();
            }
        }
    });
    (addr.to_string(), h)
}

/// Reference embedder policy: allow TCP to loopback (the test's grant), deny HTTP.
struct AllowLoopbackTcp;
impl EgressPolicy for AllowLoopbackTcp {
    fn authorize(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
        Err(EgressDenied::new("http denied by tcp-only test policy"))
    }
    fn authorize_tcp(&self, addr: &SocketAddr) -> Result<(), EgressDenied> {
        if addr.ip().is_loopback() {
            Ok(())
        } else {
            Err(EgressDenied::new("only loopback allow-listed in test"))
        }
    }
}

/// Reference embedder policy: deny all TCP (relies on the default-deny `authorize_tcp`).
struct DenyAllTcp;
impl EgressPolicy for DenyAllTcp {
    fn authorize(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
        Err(EgressDenied::new("denied"))
    }
    // authorize_tcp uses the trait default: deny.
}

fn load(
    caps: Vec<String>,
    egress: Option<Arc<dyn EgressPolicy>>,
) -> Result<PluginHandle, CallError> {
    let bytes = tcp_echo_component().expect("tcp_echo_guest.wasm present");
    let info = PluginInfo {
        name: "tcp-echo".into(),
        interface_name: "tcp-echo".into(),
        interface_hash: 0,
        interface_version: 1,
        capabilities: 0,
        buffer_strategy: BufferStrategyKind::PluginAllocated,
        runtime: PluginRuntimeKind::Wasm,
    };
    let methods = vec![WasmMethod {
        name: "connect-and-echo".into(),
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
fn tcp_egress_allowed_echoes() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP tcp_egress_allowed_echoes: tcp_echo_guest.wasm not built");
        return;
    }
    let payload = b"postgres-startup".to_vec();
    let (addr, server) = mock_tcp_echo_once(payload.len());
    let handle = load(vec!["tcp".into()], Some(Arc::new(AllowLoopbackTcp))).expect("load");
    let echoed: Vec<u8> = handle
        .call_method(0, &(addr, payload.clone()))
        .expect("echo");
    server.join().unwrap();
    assert_eq!(
        echoed, payload,
        "the bytes must round-trip through the sandbox"
    );
}

#[test]
fn tcp_egress_denied_by_policy() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP tcp_egress_denied_by_policy");
        return;
    }
    // The policy refuses the connect, so no server is needed.
    let handle = load(vec!["tcp".into()], Some(Arc::new(DenyAllTcp))).expect("load");
    let echoed: Vec<u8> = handle
        .call_method(0, &("127.0.0.1:1".to_string(), b"x".to_vec()))
        .expect("call");
    assert!(echoed.is_empty(), "denied egress must yield no bytes");
}

#[test]
fn tcp_no_capability_fails_closed() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP tcp_no_capability_fails_closed");
        return;
    }
    // Policy supplied but the package didn't declare `tcp` → no socket check is
    // installed → the deny-all WasiCtx refuses the connect.
    let handle = load(vec![], Some(Arc::new(AllowLoopbackTcp))).expect("load");
    let echoed: Vec<u8> = handle
        .call_method(0, &("127.0.0.1:1".to_string(), b"x".to_vec()))
        .expect("call");
    assert!(echoed.is_empty(), "no capability must fail closed");
}

#[test]
fn tcp_no_policy_fails_closed() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP tcp_no_policy_fails_closed");
        return;
    }
    // `tcp` declared but NO policy → no socket check installed → connect denied.
    // (Unlike wasi:http this still *loads*; wasi:sockets is always linked, so the
    // fail-closed is at connect time, not instantiate time.)
    let handle = load(vec!["tcp".into()], None).expect("load");
    let echoed: Vec<u8> = handle
        .call_method(0, &("127.0.0.1:1".to_string(), b"x".to_vec()))
        .expect("call");
    assert!(echoed.is_empty(), "no policy must fail closed");
}

// ── Ergonomic path: egress through `PluginHost::load_wasm` (not the low-level
// executor constructor). The tcp-echo fixture carries `fidius-interface-hash`, so
// it loads as a real package.

static TCP_METHODS: [WasmMethodDesc; 1] = [WasmMethodDesc {
    name: "connect-and-echo",
    wire_raw: false,
    streaming: false,
}];
static TCP_ECHO: WasmInterfaceDescriptor = WasmInterfaceDescriptor {
    interface_name: "tcp-echo",
    interface_export: IFACE,
    interface_hash: 0x7CCB_0033_0000_0001,
    methods: &TCP_METHODS,
};

/// Stage the tcp-echo fixture as a loadable wasm package declaring the `tcp` capability.
fn stage_tcp_pkg(root: &std::path::Path) {
    let dir = root.join("tcp-echo-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("tcp_echo_guest.wasm"),
        tcp_echo_component().unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("package.toml"),
        "[package]\nname = \"tcp-echo-pkg\"\nversion = \"0.1.0\"\ninterface = \"tcp-echo\"\n\
         interface_version = 1\nruntime = \"wasm\"\n\n[metadata]\ncategory = \"test\"\n\n\
         [wasm]\ncomponent = \"tcp_echo_guest.wasm\"\ncapabilities = [\"tcp\"]\n",
    )
    .unwrap();
}

#[test]
fn tcp_egress_via_builder_default_policy() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP tcp_egress_via_builder_default_policy");
        return;
    }
    let payload = b"hello via builder".to_vec();
    let (addr, server) = mock_tcp_echo_once(payload.len());
    let tmp = tempfile::TempDir::new().unwrap();
    stage_tcp_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .egress(AllowLoopbackTcp)
        .build()
        .unwrap();
    let handle = host
        .load_wasm("tcp-echo-pkg", &TCP_ECHO)
        .expect("load_wasm");
    let echoed: Vec<u8> = handle
        .call_method(0, &(addr, payload.clone()))
        .expect("echo");
    server.join().unwrap();
    assert_eq!(echoed, payload);
}

#[test]
fn tcp_egress_via_per_plugin_policy() {
    if tcp_echo_component().is_none() {
        eprintln!("SKIP tcp_egress_via_per_plugin_policy");
        return;
    }
    let payload = b"hello per-plugin".to_vec();
    let (addr, server) = mock_tcp_echo_once(payload.len());
    let tmp = tempfile::TempDir::new().unwrap();
    stage_tcp_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm_with_egress("tcp-echo-pkg", &TCP_ECHO, AllowLoopbackTcp)
        .expect("load_wasm_with_egress");
    let echoed: Vec<u8> = handle
        .call_method(0, &(addr, payload.clone()))
        .expect("echo");
    server.join().unwrap();
    assert_eq!(echoed, payload);
}
