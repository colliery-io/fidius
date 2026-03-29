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

//! Ed25519 signature verification for plugin dylibs.

use std::path::Path;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::error::LoadError;

/// Verify a plugin dylib's signature against trusted public keys.
///
/// Reads the dylib bytes and the detached `.sig` file, then verifies
/// the Ed25519 signature against each trusted key until one matches.
///
/// # Errors
///
/// - `LoadError::SignatureRequired` if the `.sig` file doesn't exist
/// - `LoadError::SignatureInvalid` if no trusted key verifies the signature
pub fn verify_signature(
    dylib_path: &Path,
    trusted_keys: &[VerifyingKey],
) -> Result<(), LoadError> {
    let path_str = dylib_path.display().to_string();

    // Build the .sig path
    let sig_path = dylib_path.with_extension(
        format!(
            "{}.sig",
            dylib_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
        ),
    );

    // Read the sig file
    let sig_bytes = std::fs::read(&sig_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            LoadError::SignatureRequired {
                path: path_str.clone(),
            }
        } else {
            LoadError::Io(e)
        }
    })?;

    // Parse the signature (64 bytes)
    let signature = Signature::from_slice(&sig_bytes).map_err(|_| LoadError::SignatureInvalid {
        path: path_str.clone(),
    })?;

    // Read the dylib bytes
    let dylib_bytes = std::fs::read(dylib_path)?;

    // Try each trusted key
    for key in trusted_keys {
        if key.verify(&dylib_bytes, &signature).is_ok() {
            return Ok(());
        }
    }

    Err(LoadError::SignatureInvalid { path: path_str })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_file(content: &[u8]) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content).unwrap();
        f
    }

    fn sign_file(path: &Path, signing_key: &SigningKey) {
        let content = std::fs::read(path).unwrap();
        let signature = signing_key.sign(&content);
        let sig_path = path.with_extension(
            format!(
                "{}.sig",
                path.extension().and_then(|e| e.to_str()).unwrap_or("")
            ),
        );
        std::fs::write(sig_path, signature.to_bytes()).unwrap();
    }

    #[test]
    fn valid_signature_succeeds() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let verifying_key = signing_key.verifying_key();

        let file = create_test_file(b"test plugin content");
        sign_file(file.path(), &signing_key);

        let result = verify_signature(file.path(), &[verifying_key]);
        assert!(result.is_ok());
    }

    #[test]
    fn tampered_file_fails() {
        let signing_key = SigningKey::from_bytes(&[2u8; 32]);
        let verifying_key = signing_key.verifying_key();

        let file = create_test_file(b"original content");
        sign_file(file.path(), &signing_key);

        // Tamper with the file
        std::fs::write(file.path(), b"tampered content").unwrap();

        let result = verify_signature(file.path(), &[verifying_key]);
        assert!(matches!(result, Err(LoadError::SignatureInvalid { .. })));
    }

    #[test]
    fn wrong_key_fails() {
        let signing_key = SigningKey::from_bytes(&[3u8; 32]);
        let wrong_key = SigningKey::from_bytes(&[4u8; 32]).verifying_key();

        let file = create_test_file(b"test content");
        sign_file(file.path(), &signing_key);

        let result = verify_signature(file.path(), &[wrong_key]);
        assert!(matches!(result, Err(LoadError::SignatureInvalid { .. })));
    }

    #[test]
    fn missing_sig_file_returns_required() {
        let key = SigningKey::from_bytes(&[5u8; 32]).verifying_key();
        let file = create_test_file(b"no sig for this");

        let result = verify_signature(file.path(), &[key]);
        assert!(matches!(result, Err(LoadError::SignatureRequired { .. })));
    }
}
