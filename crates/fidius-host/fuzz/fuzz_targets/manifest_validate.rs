// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// FIDIUS-I-0033 Phase 2: fuzz the package-manifest parse + validation path.
// `PluginDescriptor` itself is a `#[repr(C)]` FFI struct (not byte-parseable), so
// the analogous untrusted-parse surface is the package manifest: arbitrary TOML →
// `PackageManifest` → `validate_runtime`. Neither parsing nor validation may panic.
#![no_main]

use fidius_core::package::PackageManifest;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(manifest) = toml::from_str::<PackageManifest<toml::Value>>(s) {
            let _ = manifest.validate_runtime();
        }
    }
});
