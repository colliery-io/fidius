// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// Reference Rust guest for the fidius WASM backend (FIDIUS-I-0021 Phase 2).
// Implements the `greeter` interface from wit/world.wit. Built with
// `cargo component build`; the host loads the resulting component and dispatches
// into it (FIDIUS-T-0102 executor, FIDIUS-T-0105 integration tests).

#[allow(warnings)]
mod bindings;

use bindings::exports::fidius::greeter::greeter::{Guest, PluginError};

/// Must match what the host expects for this interface. The host calls
/// `fidius-interface-hash` at load and rejects a mismatch.
const INTERFACE_HASH: u64 = 0x0102_0304_0506_0708;

struct Component;

impl Guest for Component {
    fn greet(name: String) -> String {
        format!("Hello, {name}!")
    }

    fn add(a: i64, b: i64) -> Result<i64, PluginError> {
        a.checked_add(b).ok_or(PluginError {
            code: "overflow".to_string(),
            message: format!("{a} + {b} overflows i64"),
            details: None,
        })
    }

    fn echo_bytes(data: Vec<u8>) -> Vec<u8> {
        // Reverse, to prove the bytes actually round-trip through the guest.
        let mut out = data;
        out.reverse();
        out
    }

    fn fidius_interface_hash() -> u64 {
        INTERFACE_HASH
    }

    fn probe_env() -> bool {
        // Visible only when the host grants the `env` capability.
        std::env::var("FIDIUS_TEST_CAP").is_ok()
    }
}

bindings::export!(Component with_types_in bindings);
