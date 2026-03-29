---
id: r-07-human-readable-error-messages
level: task
title: "R-07: Human-readable error messages for wire/strategy mismatches"
short_code: "FIDIUS-T-0046"
created_at: 2026-03-29T17:19:42.163662+00:00
updated_at: 2026-03-29T17:27:02.558644+00:00
parent: FIDIUS-I-0008
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0008
---

# R-07: Human-readable error messages for wire/strategy mismatches

**Addresses**: API-02, LEG-06, LEG-12, OPS-03 | **Effort**: 2-3 hours

## Objective

Make wire format and buffer strategy mismatch errors actionable by storing enum values instead of raw u8 discriminants, adding Display impls, and including build profile hints so developers can immediately diagnose debug/release mismatches.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `LoadError::WireFormatMismatch` stores `WireFormat` enums (not `u8`) for `got` and `expected` fields
- [ ] `LoadError::BufferStrategyMismatch` stores `BufferStrategyKind` enums (not `u8`) for `got` and `expected` fields
- [ ] `Display` impl for `WireFormat` includes build profile hint (e.g., `"Json (debug build)"`, `"Bincode (release build)"`)
- [ ] `Display` impl for `BufferStrategyKind` provides human-readable names
- [ ] Error messages include hint: `"Ensure both plugin and host are compiled with the same build profile."`
- [ ] Loader passes enum values instead of `as u8` casts
- [ ] Existing tests pass; new test confirms mismatch error message contains profile hint

## Implementation Notes

1. In `fidius-core/src/descriptor.rs`, add `Display` impls for `WireFormat` and `BufferStrategyKind` enums with build profile annotations.
2. In `fidius-host/src/error.rs`, change `WireFormatMismatch { got: u8, expected: u8 }` to `WireFormatMismatch { got: WireFormat, expected: WireFormat }` and likewise for `BufferStrategyMismatch`.
3. In `fidius-host/src/loader.rs`, update the mismatch checks to pass the enum values directly from the descriptor accessors (which return `Result` after R-05).

### Dependencies

- R-05 (FIDIUS-T-0043): Descriptor accessors must return `Result` with enum types before this task can consume them cleanly.

### Files

- `fidius-core/src/descriptor.rs` -- Display impls
- `fidius-host/src/error.rs` -- enum-typed error variants
- `fidius-host/src/loader.rs` -- pass enums instead of u8

## Status Updates

*To be added during implementation*