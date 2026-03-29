---
id: r-12-add-callerror-unknownstatus
level: task
title: "R-12: Add CallError::UnknownStatus variant"
short_code: "FIDIUS-T-0051"
created_at: 2026-03-29T17:19:48.014582+00:00
updated_at: 2026-03-29T17:31:47.113532+00:00
parent: FIDIUS-I-0008
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0008
---

# R-12: Add CallError::UnknownStatus variant

**Addresses**: API-08 | **Effort**: < 1 hour

## Objective

Add a dedicated `CallError::UnknownStatus` variant so that unknown FFI status codes are correctly classified instead of being misreported as serialization errors, enabling host code to pattern-match on them distinctly.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `CallError::UnknownStatus { code: i32 }` variant added to CallError enum
- [ ] The `format!("unknown status code: {status}")` string in `call_method` replaced with `CallError::UnknownStatus { code: status }`
- [ ] `Display` impl for `UnknownStatus` produces a clear message (e.g., "unknown FFI status code: {code}")
- [ ] Host code that matches on `CallError` can distinguish unknown status from serialization errors
- [ ] All tests pass

## Implementation Notes

1. In `fidius-host/src/error.rs`, add `UnknownStatus { code: i32 }` to the `CallError` enum.
2. In `fidius-host/src/handle.rs`, in the `call_method` match arm for the default/unknown status case, return `Err(CallError::UnknownStatus { code: status })` instead of wrapping in `CallError::Serialization`.

### Dependencies

- None.

### Files

- `fidius-host/src/error.rs` -- new variant
- `fidius-host/src/handle.rs` -- update match arm

## Status Updates

*To be added during implementation*