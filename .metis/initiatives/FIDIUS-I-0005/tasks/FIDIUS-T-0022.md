---
id: signing-commands-keygen-sign-verify
level: task
title: "Signing commands — keygen, sign, verify"
short_code: "FIDIUS-T-0022"
created_at: 2026-03-29T11:35:18.498093+00:00
updated_at: 2026-03-29T11:51:40.041751+00:00
parent: FIDIUS-I-0005
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0005
---

# Signing commands — keygen, sign, verify

## Parent Initiative

[[FIDIUS-I-0005]]

## Objective

Implement the three signing subcommands: `keygen` generates an Ed25519 keypair, `sign` produces a detached `.sig` file for a dylib, `verify` checks a signature.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `fides keygen --out <name>` writes `<name>.secret` (32 bytes) and `<name>.public` (32 bytes)
- [ ] `fides sign --key <secret_file> <dylib>` reads dylib, signs with secret key, writes `<dylib>.sig` (64 bytes)
- [ ] `fides verify --key <public_file> <dylib>` reads dylib + `.sig`, verifies, exits 0 on success, exits 1 on failure with message
- [ ] Round-trip works: keygen → sign → verify succeeds
- [ ] Tamper detection: sign → modify dylib → verify fails
- [ ] Missing .sig file → clear error message

## Implementation Notes

### Technical Approach

Add handler functions in `fidius-cli/src/main.rs` (or split into `fidius-cli/src/commands/`).

Uses `ed25519_dalek::{SigningKey, VerifyingKey, Signer, Verifier}` and `rand::rngs::OsRng` for key generation.

Key files are raw bytes (32 bytes for secret, 32 bytes for public, 64 bytes for signature). Simple and `xxd`-inspectable.

### Dependencies
- FIDIUS-T-0021 (clap structure must exist)

## Status Updates

- **2026-03-29**: Implemented in T-0021 (all commands in one pass). Tested: keygen → sign → verify round-trip on test-plugin-smoke cdylib — all working. Keys are raw 32-byte files, sigs are raw 64-byte files.