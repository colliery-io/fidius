---
id: r-03-null-pointer-check-on-output
level: task
title: "R-03: Null-pointer check on output buffer"
short_code: "FIDIUS-T-0038"
created_at: 2026-03-29T16:29:46.546890+00:00
updated_at: 2026-03-29T16:39:09.600634+00:00
parent: FIDIUS-I-0007
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0007
---

# R-03: Null-pointer check on output buffer

**Addresses**: SEC-06, COR-06

## Parent Initiative

[[FIDIUS-I-0007]]

## Objective

Add a null-pointer check on `out_ptr` before creating a slice on the `STATUS_OK` path in `call_method`. A malicious or buggy plugin that returns `STATUS_OK` without setting `out_ptr` causes the host to create a slice from a null pointer, which is undefined behavior.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `out_ptr.is_null()` check is added before `std::slice::from_raw_parts(out_ptr, ...)` on the STATUS_OK path
- [ ] Returns `Err(CallError::Serialization("plugin returned null output buffer".into()))` when null
- [ ] Defensive check is documented with a comment explaining why

## Implementation Notes

### Technical Approach

1. In `fidius-host/src/handle.rs`, before `std::slice::from_raw_parts(out_ptr, ...)`, add:
   ```rust
   if out_ptr.is_null() {
       return Err(CallError::Serialization(
           "plugin returned null output buffer".into()
       ));
   }
   ```
2. Add a comment explaining the defensive check.

### Dependencies

None.

## Status Updates

- **2026-03-29**: Added null check before `from_raw_parts` on STATUS_OK path. Returns `CallError::Serialization` with descriptive message. Compiles clean.