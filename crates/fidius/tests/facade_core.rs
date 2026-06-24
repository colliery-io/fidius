// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// FIDIUS-I-0033 Phase 5: first-class tests for the `fidius` facade. These exercise
// the *re-exported* public surface the way a downstream consumer does — not just
// `use` compile-guards — so a broken or renamed re-export fails a test, not just a
// doctest. This file covers the always-available (no-feature) core surface.
//
// The `plugin_interface` macro generates a `{Trait}_VTable` struct and a trait only
// referenced through generated companion items — silence the codegen-shape lints.
#![allow(non_camel_case_types, dead_code)]

use fidius::{from_value, to_value, wire, PluginError, Value};

#[test]
fn wire_roundtrip_through_facade() {
    let payload: Vec<(String, i64)> = vec![("a".into(), 1), ("b".into(), -2)];
    let bytes = wire::serialize(&payload).expect("serialize via facade");
    let back: Vec<(String, i64)> = wire::deserialize(&bytes).expect("deserialize via facade");
    assert_eq!(payload, back);
}

#[test]
fn value_bridge_through_facade() {
    let original = vec![("k".to_string(), 7u32)];
    let v: Value = to_value(&original).expect("to_value via facade");
    let back: Vec<(String, u32)> = from_value(v).expect("from_value via facade");
    assert_eq!(original, back);
}

#[test]
fn plugin_error_is_reexported_and_constructs() {
    let err = PluginError::new("ERR_CODE", "boom");
    assert!(format!("{err}").contains("boom"));
}

#[test]
fn hashing_is_reexported() {
    // FNV-1a + interface hashing are part of the public contract.
    let a = fidius::hash::fnv1a(b"fidius");
    let b = fidius::hash::fnv1a(b"fidius");
    let c = fidius::hash::fnv1a(b"fidius!");
    assert_eq!(a, b, "fnv1a must be deterministic");
    assert_ne!(a, c, "different input → different hash");
}

#[test]
fn abi_constants_are_reexported() {
    // Downstream loaders compare these; they must be reachable through the facade.
    let _abi: u32 = fidius::ABI_VERSION;
    let _magic = fidius::FIDIUS_MAGIC;
}

// The macro is the headline facade export. Defining an interface through
// `fidius::plugin_interface` (with `crate = "fidius"`) must generate the interface
// hash — proving the macro's codegen resolves against the facade crate path.
#[fidius::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius")]
trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;
}

#[test]
#[allow(non_upper_case_globals)]
fn plugin_interface_macro_generates_hash_through_facade() {
    assert_ne!(
        __fidius_Greeter::Greeter_INTERFACE_HASH,
        0,
        "macro must generate a non-zero hash"
    );
}
