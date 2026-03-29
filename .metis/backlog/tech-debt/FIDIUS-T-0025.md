---
id: handle-result-t-pluginerror
level: task
title: "Handle Result<T, PluginError> distinctly in FFI shims"
short_code: "FIDIUS-T-0025"
created_at: 2026-03-29T12:20:38.034784+00:00
updated_at: 2026-03-29T12:35:20.925260+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Handle Result<T, PluginError> distinctly in FFI shims

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective

The FFI shims currently serialize the entire return value of the method (including `Result` wrappers). Per the spec, when a method returns `Result<T, PluginError>`, the shim should unwrap it: serialize `T` on `Ok` with `STATUS_OK`, or serialize the `PluginError` on `Err` with `STATUS_PLUGIN_ERROR`. This enables the host's `call_method` to distinguish plugin errors from serialization errors via status codes.

## Technical Debt Impact

- **Current Problems**: The host gets back a serialized `Result<T, PluginError>` as the output, bypassing the status code mechanism. `CallError::Plugin` is never returned — the caller has to manually unwrap the Result themselves.
- **Benefits of Fixing**: Clean separation: status code tells the host what happened, output buffer contains either `T` or `PluginError` depending on status. Matches the spec and enables `PluginHandle::call_method` to return `CallError::Plugin(err)` automatically.
- **Risk Assessment**: Medium — changes the FFI contract for Result-returning methods. Existing tests need updating.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Shim detects if the method return type is `Result<T, PluginError>` (via syn type inspection)
- [ ] On `Ok(val)`: serialize `val`, return `STATUS_OK`
- [ ] On `Err(plugin_err)`: serialize `plugin_err`, return `STATUS_PLUGIN_ERROR`
- [ ] Non-Result return types continue to work as before (serialize directly, STATUS_OK)
- [ ] `PluginHandle::call_method` returns `CallError::Plugin(err)` when status is PLUGIN_ERROR
- [ ] Test: plugin method returns `Err(PluginError::new(...))`, host gets `CallError::Plugin`

## Implementation Notes

### Technical Approach

File: `fidius-macro/src/impl_macro.rs`, `generate_shims()` function.

Inspect the method's return type in the IR. If it matches `Result<_, PluginError>`, generate a shim that pattern-matches the return:
```rust
match instance.method(args) {
    Ok(val) => { serialize(val); STATUS_OK }
    Err(err) => { serialize(err); STATUS_PLUGIN_ERROR }
}
```

The type detection can use the `MethodIR.return_type` field — check if it's a `Result` with `PluginError` as the error type.

## Status Updates

- **2026-03-29**: Fixed. Added `returns_result` to `MethodInfo` via `is_result_type()` check on return type. Shims now pattern-match `Ok(val)` → serialize val + STATUS_OK, `Err(err)` → serialize err + STATUS_PLUGIN_ERROR. Non-Result methods unchanged. All 62 tests pass.