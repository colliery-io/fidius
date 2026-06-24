// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// FIDIUS-I-0033 Phase 2: fuzz the bincode wire decoder on untrusted bytes.
//
// NOTE: the neutral `Value` itself can't be bincode-decoded — its `Deserialize`
// uses `deserialize_any`, which bincode (not self-describing) rejects, so `Value`
// never crosses the bincode wire (concrete user types do). So this target decodes
// arbitrary bytes as a representative concrete payload: it must never panic, and
// any value the decoder accepts must survive a re-encode/decode unchanged. The
// exhaustive structural round-trip lives in the proptest (`fidius-guest`,
// `proptest_wire`); this is the byte-level robustness half.
#![no_main]

use fidius_guest::wire;
use libfuzzer_sys::fuzz_target;

// A record-shaped payload (string-keyed entries) — the common FFI arg/return shape.
type Payload = Vec<(String, i64)>;

fuzz_target!(|data: &[u8]| {
    if let Ok(v) = wire::deserialize::<Payload>(data) {
        let bytes = wire::serialize(&v).expect("re-serialize a decoded payload");
        let v2: Payload = wire::deserialize(&bytes).expect("re-decode round-trip");
        assert_eq!(v, v2, "wire round-trip changed the payload");
    }
});
