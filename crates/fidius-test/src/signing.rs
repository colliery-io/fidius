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

//! Deterministic signing fixtures for tests that exercise Fidius signature
//! verification flows.
//!
//! These helpers are **not secure** — the signing keys are derived from a
//! single byte seed. They exist so tests can sign and verify plugin dylibs
//! without generating fresh random keys each run.

use std::path::Path;

use ed25519_dalek::{Signer, SigningKey, VerifyingKey};

/// Deterministic Ed25519 keypair derived from `seed` repeated 32 times.
///
/// Use different seeds across tests that need distinct keys (e.g., to verify
/// that a wrong-key signature is rejected).
pub fn fixture_keypair_with_seed(seed: u8) -> (SigningKey, VerifyingKey) {
    let signing = SigningKey::from_bytes(&[seed; 32]);
    let verifying = signing.verifying_key();
    (signing, verifying)
}

/// Convenience: [`fixture_keypair_with_seed(1)`](fixture_keypair_with_seed).
pub fn fixture_keypair() -> (SigningKey, VerifyingKey) {
    fixture_keypair_with_seed(1)
}

/// Sign a plugin dylib in place by writing a detached `.sig` file alongside it.
///
/// The signature file uses the same naming convention as `fidius sign` —
/// appends `.sig` to the full filename (e.g., `foo.dylib` → `foo.dylib.sig`).
pub fn sign_dylib(dylib: &Path, key: &SigningKey) -> std::io::Result<()> {
    let bytes = std::fs::read(dylib)?;
    let signature = key.sign(&bytes);
    let sig_path = dylib.with_extension(format!(
        "{}.sig",
        dylib.extension().and_then(|e| e.to_str()).unwrap_or("")
    ));
    std::fs::write(sig_path, signature.to_bytes())?;
    Ok(())
}
