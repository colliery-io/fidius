---
id: tuple-pack-shim-codegen-in-plugin
level: task
title: "Tuple-pack shim codegen in plugin_impl for 0, 1, and N args"
short_code: "FIDIUS-T-0065"
created_at: 2026-04-01T02:15:27.633477+00:00
updated_at: 2026-04-01T02:25:18.320537+00:00
parent: FIDIUS-I-0010
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0010
---

# Tuple-pack shim codegen in plugin_impl for 0, 1, and N args

## Parent Initiative

[[FIDIUS-I-0010]]

## Objective

Change the shim codegen in `impl_macro.rs` so that all methods use uniform tuple encoding for arguments. Zero args = `()`, one arg = `(T,)`, N args = `(A, B, ...)`. This is a breaking wire format change.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Zero-arg methods: shim deserializes `()`, calls method with no args
- [ ] Single-arg methods: shim deserializes `(T,)`, calls method with one arg
- [ ] Multi-arg methods: shim deserializes `(A, B)`, calls method with individual args
- [ ] All existing tests updated for new wire format (callers must now serialize tuples)
- [ ] `test-plugin-smoke` updated to use tuple encoding
- [ ] Full pipeline E2E test passes with tuple encoding
- [ ] Compiles and all tests pass

## Implementation Notes

### Files to modify
- `fidius-macro/src/impl_macro.rs` — change `generate_shims` to build tuple deserialization and multi-arg method calls
- `fidius-macro/tests/impl_basic.rs` — update test to serialize `(String,)` instead of bare `String`
- `fidius-macro/tests/crate_path.rs` — same
- `fidius-host/src/handle.rs` — `call_method` callers need to pass tuples (or document the convention)
- `tests/test-plugin-smoke/src/lib.rs` — update if needed
- `fidius-cli/tests/full_pipeline.rs` — update `call_method` call

### Key codegen change
In `generate_shims`, replace:
```rust
let args = #crate_path::wire::deserialize(in_slice)?;
let output = instance.method(args);
```
With:
```rust
let (#(#arg_names,)*) = #crate_path::wire::deserialize::<(#(#arg_types,)*)>(in_slice)?;
let output = instance.method(#(#arg_names),*);
```

The IR already has `arg_types` and `arg_names` — this is purely a codegen change.

## Status Updates

- 2026-03-31: Added `arg_types` and `arg_names` to `MethodInfo`, extracted from impl block. Changed `generate_shims` to deserialize `(#(#arg_types,)*)` tuples and call methods with individual args. Updated all callers to serialize tuples: `impl_basic`, `crate_path`, `async_plugin`, `multi_plugin`, `smoke_cdylib`, `integration`, `package_e2e`, `e2e`, `full_pipeline`. Full test suite passes.