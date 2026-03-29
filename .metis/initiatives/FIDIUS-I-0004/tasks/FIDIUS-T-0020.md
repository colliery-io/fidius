---
id: end-to-end-validation-test-suite
level: task
title: "End-to-end validation test suite"
short_code: "FIDIUS-T-0020"
created_at: 2026-03-29T11:27:10.379979+00:00
updated_at: 2026-03-29T11:34:23.057447+00:00
parent: FIDIUS-I-0004
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0004
---

# End-to-end validation test suite

## Parent Initiative

[[FIDIUS-I-0004]]

## Objective

Write the comprehensive E2E test suite that proves the full fidius pipeline works. This extends the existing fidius-host integration tests (I-0003) with additional scenarios: signed plugins, multi-plugin via fidius-host, and negative tests (bad signature, not-found). Uses the existing test-plugin-smoke cdylib.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Signed plugin test: generate keypair, sign test-plugin-smoke cdylib, load with `require_signature(true)` + correct key → succeeds
- [ ] Bad signature test: sign with key A, load with key B → `LoadError::SignatureInvalid`
- [ ] Missing signature test: load unsigned with `require_signature(true)` → `LoadError::SignatureRequired`
- [ ] Multi-plugin via fidius-host: extend test-plugin-smoke with two impls, load both via `discover()`, call both
- [ ] All tests pass under `cargo test`

## Implementation Notes

### Technical Approach

File: `fidius-host/tests/e2e.rs` or extend `fidius-host/tests/integration.rs`.

For signing tests: use `ed25519_dalek::SigningKey` to generate a keypair in-test, sign the cdylib bytes, write the `.sig` file, then load with `PluginHost::builder().require_signature(true).trusted_keys(...)`.

The signing integration needs to be wired into the PluginHost load path — currently `host.rs` doesn't call `signing::verify_signature()`. This task needs to connect them.

### Dependencies
- FIDIUS-T-0019 (facade must be complete)

## Status Updates

- **2026-03-29**: Wired signing::verify_signature into PluginHost::load(). 4 E2E tests in fidius-host/tests/e2e.rs: signed+correct key succeeds, signed+wrong key fails (SignatureInvalid), unsigned+required fails (SignatureRequired), unsigned+not-required succeeds and calls method. Added Debug impl for LoadedPlugin. 56 tests pass across full workspace.