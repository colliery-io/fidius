---
id: r-14-preserve-panic-messages
level: task
title: "R-14: Preserve panic messages across FFI boundary"
short_code: "FIDIUS-T-0052"
created_at: 2026-03-29T17:19:49.397737+00:00
updated_at: 2026-03-29T17:32:35.857851+00:00
parent: FIDIUS-I-0008
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0008
---

# R-14: Preserve panic messages across FFI boundary

**Addresses**: OPS-09, OPS-13 | **Effort**: 3-4 hours

## Objective

Preserve panic messages across the FFI boundary so that when a plugin panics, the host receives the actual panic payload string in `CallError::Panic(String)` instead of a generic "plugin panicked during method call" message.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Generated FFI shim extracts panic payload from `catch_unwind` result using `downcast_ref::<&str>` and `downcast_ref::<String>`
- [ ] Panic message serialized into output buffer with `STATUS_PANIC` status code
- [ ] Host-side `call_method` deserializes panic message string from output buffer on `STATUS_PANIC`
- [ ] `CallError::Panic` changed from unit variant to `Panic(String)` carrying the message
- [ ] Fallback to `"unknown panic"` when payload is neither `&str` nor `String`
- [ ] Test verifying a panicking plugin method returns `CallError::Panic` with the original panic message
- [ ] `shrink_to_fit()` called on panic output buffer before `std::mem::forget` (same as R-01 pattern)

## Implementation Notes

1. In `fidius-macro/src/impl_macro.rs`, in the `generate_shims` function, update the `Err(panic_payload)` arm of `catch_unwind`:
   - Extract message via `downcast_ref::<&str>()` or `downcast_ref::<String>()`, fallback to `"unknown panic"`
   - Serialize the message string into output bytes
   - Call `shrink_to_fit()` before `std::mem::forget`
   - Write `STATUS_PANIC` to the status output, set pointer/length outputs
2. In `fidius-host/src/handle.rs`, in the `STATUS_PANIC` match arm of `call_method`, read the output buffer and deserialize the panic message string.
3. In `fidius-host/src/error.rs`, change `Panic` variant to `Panic(String)`.

### Dependencies

- R-03 (FIDIUS-T-0041): Null-pointer check on output buffer should be done first, as the panic path now produces an output buffer that goes through the same read logic.

### Files

- `fidius-macro/src/impl_macro.rs` -- shim panic payload extraction
- `fidius-host/src/handle.rs` -- deserialize panic message
- `fidius-host/src/error.rs` -- Panic(String) variant

## Status Updates

*To be added during implementation*