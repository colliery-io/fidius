---
id: r-04-move-signature-verification
level: task
title: "R-04: Move signature verification before dlopen"
short_code: "FIDIUS-T-0044"
created_at: 2026-03-29T16:29:53.668160+00:00
updated_at: 2026-03-29T17:05:18.778579+00:00
parent: FIDIUS-I-0007
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0007
---

# R-04: Move signature verification before dlopen

**Addresses**: COR-05, SEC-02, API-03

## Parent Initiative

[[FIDIUS-I-0007]]

## Objective

Move signature verification before `dlopen` in both `discover()` and `load()`. Currently, `dlopen` executes constructor code in the dylib before any validation. `discover()` opens every dylib it finds with no signature check, enabling code execution from untrusted files placed in search paths. This is a code execution vulnerability.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `verify_signature` called before `load_library()` in `discover()` when `require_signature` is true
- [ ] `verify_signature` called before `load_library()` in `load()` when `require_signature` is true
- [ ] Per-file rejection reasons collected in `discover()` (optionally returned)
- [ ] Architecture checking (header byte inspection) remains the only pre-signature gate and does not execute code

## Implementation Notes

### Technical Approach

1. The `signing::verify_signature` function already operates on file paths.
2. In both `discover()` and `load()`, call `verify_signature` before `load_library()` when `require_signature` is true.
3. For `discover()`, collect and optionally return per-file rejection reasons.
4. Document that architecture checking (header byte inspection) is the only pre-signature gate, and that it does not execute code.

### Dependencies

Depends on FIDIUS-T-0039 (R-05: descriptor panic fixes). R-09 (observability) would complement this by making rejection reasons visible.

## Status Updates

*To be added during implementation*