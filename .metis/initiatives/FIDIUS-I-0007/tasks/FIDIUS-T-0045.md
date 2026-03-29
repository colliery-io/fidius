---
id: r-16-fix-loadpolicy-lenient
level: task
title: "R-16: Fix LoadPolicy::Lenient signature semantics"
short_code: "FIDIUS-T-0045"
created_at: 2026-03-29T16:29:54.596825+00:00
updated_at: 2026-03-29T17:08:46.059977+00:00
parent: FIDIUS-I-0007
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0007
---

# R-16: Fix LoadPolicy::Lenient signature semantics

**Addresses**: SEC-03

## Parent Initiative

[[FIDIUS-I-0007]]

## Objective

Fix `LoadPolicy::Lenient` signature semantics. When `require_signature` is true, always enforce signature verification regardless of load policy. The current behavior -- requiring signatures but ignoring failures under `Lenient` -- creates a false sense of security. `Lenient` should only affect non-security validation (hash mismatch, version mismatch, wire format mismatch).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] When `require_signature` is true and verification fails, always return `Err` regardless of `Lenient` policy
- [ ] `Lenient` only affects behavior for `WireFormatMismatch`, `BufferStrategyMismatch`, and `InterfaceHashMismatch`
- [ ] Documentation updated to clarify what `Lenient` controls
- [ ] Test added: Lenient + require_signature + invalid sig = error (not warning)

## Implementation Notes

### Technical Approach

1. In `fidius-host/src/host.rs`, remove the `Lenient` fallback for signature verification errors. If `require_signature` is true and verification fails, always return `Err`.
2. `Lenient` should only affect the behavior for `WireFormatMismatch`, `BufferStrategyMismatch`, and `InterfaceHashMismatch`.
3. Update documentation to clarify what `Lenient` controls.
4. Add tests: Lenient + require_signature + invalid sig = error (not warning).

### Dependencies

Depends on FIDIUS-T-0044 (R-04: signature before dlopen) for a coherent signing story.

## Status Updates

*To be added during implementation*