---
id: r-05-replace-descriptor-panics
level: task
title: "R-05: Replace descriptor panics with Result returns"
short_code: "FIDIUS-T-0039"
created_at: 2026-03-29T16:29:47.440155+00:00
updated_at: 2026-03-29T16:40:57.014697+00:00
parent: FIDIUS-I-0007
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0007
---

# R-05: Replace descriptor panics with Result returns

**Addresses**: COR-13, COR-14, SEC-07, API-04

## Parent Initiative

[[FIDIUS-I-0007]]

## Objective

Replace panicking descriptor field parsers with Result returns or safe defaults. `buffer_strategy_kind()`, `wire_format_kind()`, and `has_capability()` are called on data from loaded plugins -- a single malformed dylib with an unknown `wire_format` or `buffer_strategy` byte crashes the host process. Combined with `discover()` scanning all dylibs, one corrupted file in a search path can DoS any host.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `buffer_strategy_kind()` returns `Result<BufferStrategyKind, u8>` instead of panicking
- [ ] `wire_format_kind()` returns `Result<WireFormat, u8>` instead of panicking
- [ ] `has_capability(bit)` returns `false` for `bit >= 64` instead of panicking
- [ ] New `LoadError` variants added (e.g., `UnknownWireFormat { value: u8 }`, `UnknownBufferStrategy { value: u8 }`)
- [ ] Callers in `fidius-host/src/loader.rs` updated to use `?` propagation

## Implementation Notes

### Technical Approach

1. In `fidius-core/src/descriptor.rs`:
   - Change `buffer_strategy_kind()` to return `Result<BufferStrategyKind, u8>` (returning the unknown value on error).
   - Change `wire_format_kind()` to return `Result<WireFormat, u8>` (same).
   - Change `has_capability(bit)` to return `false` for `bit >= 64` instead of panicking.
2. In `fidius-host/src/loader.rs`, propagate the `Err` as new `LoadError` variants.
3. Update callers in `fidius-host/src/loader.rs` to use `?` propagation.

### Dependencies

None.

## Status Updates

*To be added during implementation*