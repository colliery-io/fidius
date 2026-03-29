---
id: r-01-fix-free-buffer-capacity
level: task
title: "R-01: Fix free_buffer capacity mismatch"
short_code: "FIDIUS-T-0037"
created_at: 2026-03-29T16:29:45.214822+00:00
updated_at: 2026-03-29T16:38:11.813070+00:00
parent: FIDIUS-I-0007
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0007
---

# R-01: Fix free_buffer capacity mismatch

**Addresses**: COR-03, SEC-08

## Parent Initiative

[[FIDIUS-I-0007]]

## Objective

Add `output_bytes.shrink_to_fit()` before `std::mem::forget(output_bytes)` in generated FFI shim code to fix the `free_buffer` capacity mismatch. Every plugin method call currently triggers undefined behavior because `free_buffer` reconstructs a `Vec` with `capacity == len`, but the original `Vec` may have excess capacity, causing incorrect deallocation size and potential heap corruption.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `output_bytes.shrink_to_fit()` is added immediately before `let len = output_bytes.len();` in the generated shim code
- [ ] Fix is applied to both the normal return path and the `returns_result` Ok path
- [ ] A test verifies `free_buffer` completes without sanitizer errors

## Implementation Notes

### Technical Approach

1. In `fidius-macro/src/impl_macro.rs`, in the `generate_shims` function, add `output_bytes.shrink_to_fit();` to the generated code immediately before the line that reads `let len = output_bytes.len();`.
2. Apply the same fix to both the normal return path and the `returns_result` Ok path.
3. Add a test that verifies `free_buffer` is called correctly (e.g., by checking that the method completes without sanitizer errors).

### Dependencies

None. Self-contained one-line fix in the code generator.

## Status Updates

- **2026-03-29**: Added `output_bytes.shrink_to_fit()` before `len()` in generated shim. Single location covers both normal and Result paths. Tests pass.