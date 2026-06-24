// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// FIDIUS-I-0033 Phase 2: fuzz the framed streaming wire decoder. `Frame::read`
// parses length-prefixed payloads out of untrusted bytes — exactly the kind of
// bounds logic fidius owns. It must never panic; and a cleanly-decoded frame must
// re-encode to bytes that decode back to the same frame (round-trip stability).
#![no_main]

use fidius_guest::Frame;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok((frame, _consumed)) = Frame::read(data) {
        if let Ok(bytes) = frame.encode() {
            let again = Frame::decode(&bytes).expect("decode re-encoded frame");
            assert_eq!(frame, again, "frame round-trip changed the frame");
        }
    }
});
