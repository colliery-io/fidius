// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// FIDIUS-I-0033 Phase 5: facade tests for the `wasm` egress surface. Exercises the
// re-exported `EgressPolicy`/`EgressDenied` (incl. the default-deny `authorize_tcp`
// / `authorize_udp` from FIDIUS-I-0033) through the facade — naming `http::Parts`
// via the re-exported `http_types`, so the test needs no direct `http` dep.
#![cfg(feature = "wasm")]

use fidius::{http_types, EgressDenied, EgressPolicy};
use std::net::SocketAddr;

/// A policy that implements only the required `authorize` (HTTP) and leaves the TCP
/// and UDP hooks at their trait defaults.
struct HttpOnly;
impl EgressPolicy for HttpOnly {
    fn authorize(&self, _parts: &mut http_types::request::Parts) -> Result<(), EgressDenied> {
        Ok(())
    }
}

#[test]
fn egress_tcp_and_udp_default_to_deny_through_facade() {
    let addr: SocketAddr = "203.0.113.10:5432".parse().unwrap();
    // The two-key gate's fail-closed default: a policy that doesn't override the
    // raw-socket hooks must NOT grant TCP or UDP.
    assert!(
        HttpOnly.authorize_tcp(&addr).is_err(),
        "default authorize_tcp must deny"
    );
    assert!(
        HttpOnly.authorize_udp(&addr).is_err(),
        "default authorize_udp must deny"
    );
}

#[test]
fn egress_denied_constructs_through_facade() {
    let denied = EgressDenied::new("not allowed");
    assert!(format!("{denied:?}").contains("not allowed"));
}

// The host-function channel's wasm bind path must resolve through the facade
// when a downstream enables `host` + `wasm` (this test crate does): the
// generated `<Trait>Binding::bind_wasm` names `fidius::PluginHandle::
// bind_wasm_host_table`, which only exists behind these features.
#[fidius::host_interface(version = 1, crate = "fidius")]
trait WasmEchoHost: Send + Sync {
    fn echo(&self, s: String) -> String;
}

#[test]
#[allow(non_upper_case_globals)]
fn host_interface_wasm_bind_surface_resolves_through_facade() {
    // Name the generated wasm bind entry point (compile guard) without
    // loading a component: a fn pointer to it must typecheck.
    let _bind: fn(
        &fidius::PluginHandle,
        std::sync::Arc<dyn WasmEchoHost>,
    ) -> Result<(), fidius::LoadError> = WasmEchoHostBinding::bind_wasm;
    assert_ne!(
        __fidius_host_WasmEchoHost::WasmEchoHost_HOST_INTERFACE_HASH,
        0
    );
}
