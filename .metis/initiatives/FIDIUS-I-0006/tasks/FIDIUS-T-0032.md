---
id: cli-package-signing-sign-and-verify
level: task
title: "CLI package signing — sign and verify"
short_code: "FIDIUS-T-0032"
created_at: 2026-03-29T14:00:05.861574+00:00
updated_at: 2026-03-29T14:39:36.994875+00:00
parent: FIDIUS-I-0006
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0006
---

# CLI package signing — sign and verify

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0006]]

## Objective

Implement `fidius package sign` and `fidius package verify` CLI subcommands. Signs `package.toml` (not the source tree), reusing the existing Ed25519 signing infrastructure. The signature establishes trust in the manifest metadata.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `fidius package sign --key <secret> <dir>` — signs `<dir>/package.toml`, writes `<dir>/package.toml.sig`
- [ ] `fidius package verify --key <public> <dir>` — verifies `package.toml` against `package.toml.sig`, exits 0/1
- [ ] Reuses existing `ed25519_dalek` signing code from `fidius-cli/src/commands.rs`
- [ ] Round-trip: sign → verify succeeds
- [ ] Tamper: sign → modify `package.toml` → verify fails
- [ ] Missing `.sig` → clear error on verify

## Implementation Notes

### Technical Approach

Add `Sign` and `Verify` variants to the `PackageCommands` enum. Implementation is nearly identical to the existing `fidius sign` / `fidius verify` commands but operates on `package.toml` instead of a dylib.

Refactor the signing logic from `commands.rs` into a shared function if not already, so both dylib signing and manifest signing use the same code path.

### Dependencies
- FIDIUS-T-0030 (package types)
- FIDIUS-T-0031 (package subcommand group must exist)

## Status Updates

- **2026-03-29**: Implemented in T-0031 (all package commands done in one pass). `package_sign` and `package_verify` delegate to existing `sign`/`verify` functions with `package.toml` as the target path. Reuses same Ed25519 code path.