---
id: r-18-add-test-coverage-for-error
level: task
title: "R-18: Add test coverage for error paths"
short_code: "FIDIUS-T-0054"
created_at: 2026-03-29T17:19:51.240001+00:00
updated_at: 2026-03-29T17:35:29.540265+00:00
parent: FIDIUS-I-0008
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0008
---

# R-18: Add test coverage for error paths

**Addresses**: COR-16, COR-15, COR-19 | **Effort**: 2-3 days

## Objective

Add test coverage for all untested error paths through `call_method` and related infrastructure, including Result-returning methods, plugin panics, out-of-bounds vtable access, unimplemented optional methods, and hash regression vectors. Eliminate signing test flakiness with per-test temp dirs.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Test fixture plugin with a method returning `Result<T, PluginError>` -- both Ok and Err paths tested
- [ ] Test fixture plugin with a method that panics -- verify `CallError::Panic` is returned (with message after R-14)
- [ ] Test for out-of-bounds vtable index returning `CallError::MethodNotFound` (requires R-02)
- [ ] Test for calling an unimplemented optional method returning `CallError::NotImplemented` (requires R-02)
- [ ] `hash_known_vectors` test uses hardcoded expected hash values instead of comparing to self
- [ ] All signing tests use per-test temporary directories (no shared state, no flakiness)
- [ ] All new tests pass in CI

## Implementation Notes

1. In `fidius-host/tests/`, add integration tests:
   - Create a test fixture plugin crate (or extend existing one) with a Result-returning method and a panicking method.
   - Test `STATUS_PLUGIN_ERROR` path: call the error-returning method, assert `CallError::PluginError` is returned.
   - Test `STATUS_PANIC` path: call the panicking method, assert `CallError::Panic` is returned.
   - After R-02: test out-of-bounds index and null function pointer (unimplemented optional method).
2. In `fidius-macro/tests/`, add macro expansion tests verifying the generated shim handles panics and errors correctly.
3. For hash tests: hardcode exact expected values so regressions are caught.
4. For signing tests: use `tempfile::tempdir()` for each test to create isolated working directories.

### Dependencies

- R-02 (FIDIUS-T-0040): Required for bounds-checking and NotImplemented tests.
- R-14 (FIDIUS-T-0052): Required for panic message verification tests.
- Tests for features not yet implemented should be added as those tasks are completed.

### Files

- `fidius-host/tests/` -- integration tests for error paths
- `fidius-macro/tests/` -- macro expansion tests

## Status Updates

*To be added during implementation*