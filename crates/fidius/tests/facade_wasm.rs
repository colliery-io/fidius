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
