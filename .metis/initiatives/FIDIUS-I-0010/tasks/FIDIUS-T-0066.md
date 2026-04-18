---
id: tests-and-e2e-validation-for-multi
level: task
title: "Tests and E2E validation for multi-arg tuple packing"
short_code: "FIDIUS-T-0066"
created_at: 2026-04-01T02:15:28.637102+00:00
updated_at: 2026-04-01T02:29:10.617838+00:00
parent: FIDIUS-I-0010
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0010
---

# Tests and E2E validation for multi-arg tuple packing

## Parent Initiative

[[FIDIUS-I-0010]]

## Objective

Add dedicated compile tests for multi-arg and zero-arg methods, and ensure all existing E2E tests pass with the new tuple wire format.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New compile test with trait containing zero-arg, one-arg, two-arg, and three-arg methods
- [ ] Shim callable through vtable for each arg count variant
- [ ] Existing `impl_basic`, `crate_path`, `multi_plugin`, `async_plugin` tests pass
- [ ] Smoke test (`test-plugin-smoke`) passes
- [ ] Full pipeline test passes
- [ ] `angreal test` all green

## Implementation Notes

### Files to create/modify
- `fidius-macro/tests/multi_arg.rs` — new test with 0/1/2/3 arg methods, callable via vtable
- Existing tests may need serialization updates if they call vtable shims directly

### Dependencies
- Blocked by FIDIUS-T-0065

## Status Updates

- 2026-03-31: Created `multi_arg.rs` with 5 tests covering 0/1/2/3 arg methods via vtable. Added `add_direct(a, b)` (multi-arg) and `version()` (zero-arg) to `test-plugin-smoke` Calculator trait. Added 2 new host-side E2E tests in `integration.rs`: `call_multi_arg_add_direct` and `call_zero_arg_version` that go through `PluginHost` → `PluginHandle` → `call_method`. Fixed vtable index for `multiply` (now index 3). Full pipeline test covers custom `.testpkg` extension. All tests pass (integration went from 8 to 10).