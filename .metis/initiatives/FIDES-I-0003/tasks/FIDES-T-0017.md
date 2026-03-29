---
id: signature-verification
level: task
title: "Signature verification"
short_code: "FIDES-T-0017"
created_at: 2026-03-29T01:28:34.107304+00:00
updated_at: 2026-03-29T11:24:34.792275+00:00
parent: FIDES-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDES-I-0003
---

# Signature verification

## Parent Initiative

[[FIDES-I-0003]]

## Objective

Implement Ed25519 signature verification for plugin dylibs. The host reads a `.sig` file alongside the dylib, verifies it against configured trusted public keys, and rejects unsigned plugins when `require_signature` is set.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `verify_signature(dylib_path, sig_path, trusted_keys) -> Result<(), LoadError>` function
- [ ] Reads dylib bytes, reads `.sig` file, verifies Ed25519 signature via `ed25519-dalek`
- [ ] Missing `.sig` file → `LoadError::SignatureRequired` (when required)
- [ ] Invalid signature → `LoadError::SignatureInvalid`
- [ ] Signature verified against any of the trusted keys — first match wins
- [ ] Integrated into the load sequence: called between descriptor validation and handle construction
- [ ] Unit tests: generate keypair, sign a file, verify succeeds; tamper with file, verify fails; wrong key, verify fails

## Implementation Notes

### Technical Approach

File: `fides-host/src/signing.rs`

Use `ed25519_dalek::{SigningKey, VerifyingKey, Signature}`. The `.sig` file contains the raw 64-byte signature. The dylib bytes are the message.

```rust
let dylib_bytes = std::fs::read(dylib_path)?;
let sig_bytes = std::fs::read(sig_path)?;
let signature = Signature::from_bytes(&sig_bytes)?;
for key in trusted_keys {
    if key.verify(&dylib_bytes, &signature).is_ok() {
        return Ok(());
    }
}
Err(LoadError::SignatureInvalid)
```

### Dependencies
- FIDES-T-0013 (LoadError types)

## Status Updates

- **2026-03-29**: Implemented in `fides-host/src/signing.rs`. `verify_signature()` reads dylib + .sig, verifies against trusted keys. 4 unit tests pass: valid sig succeeds, tampered file fails, wrong key fails, missing sig returns SignatureRequired.