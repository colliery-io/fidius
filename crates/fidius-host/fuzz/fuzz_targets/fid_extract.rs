// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// FIDIUS-I-0033 Phase 2: fuzz the `.fid` archive extraction path (the safe
// extraction hardened in FIDIUS-T-0084). Arbitrary bytes are written to a `.fid`
// file and run through `unpack_fid`; path traversal, symlinks, decompression
// bombs, and entry-count blowups must all be rejected without panicking — and
// without writing outside the destination directory.
#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Write;

fuzz_target!(|data: &[u8]| {
    let dir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(_) => return,
    };
    let archive = dir.path().join("input.fid");
    let dest = dir.path().join("out");
    {
        let mut f = match std::fs::File::create(&archive) {
            Ok(f) => f,
            Err(_) => return,
        };
        if f.write_all(data).is_err() {
            return;
        }
    }
    // Must never panic; a malicious archive returns Err, never escapes `dest`.
    let _ = fidius_host::package::unpack_fid(&archive, &dest);
});
