---
id: cover-cdylib-executor-error-paths
level: task
title: "Cover cdylib executor error paths (fidius-host executor/cdylib.rs ~49%)"
short_code: "FIDIUS-T-0192"
created_at: 2026-06-23T22:04:36.297106+00:00
updated_at: 2026-06-23T22:04:36.297106+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
initiative_id: NULL
---

# Cover cdylib executor error paths (fidius-host executor/cdylib.rs ~49%)

## Objective

The FIDIUS-I-0033 coverage baseline (2026-06-23, native + streaming) shows
`fidius-host/src/executor/cdylib.rs` at **~49%** region coverage — the largest
under-covered host module (843 regions, ~433 uncovered). The happy path is well
exercised by the e2e suites; the gaps are error/edge paths: malformed FFI returns,
status-code mismatches, buffer/free edge cases, arena vs plugin-allocated branches.

Add focused unit/integration tests for those branches so the cdylib dispatch
failure modes are covered.

## Acceptance Criteria

- [ ] `executor/cdylib.rs` error/edge branches gain meaningful coverage (well above
      the ~49% baseline).
- [ ] Tests cover malformed-return / status-mismatch / buffer-free edge paths.

## Notes

Filed from the FIDIUS-I-0033 Phase 1 baseline (T-0180). The `fidius` facade (0% own
tests) and the broader backend matrix are tracked separately by T-0189 / T-0190.
Report-only — a prioritized gap, not a regression.
