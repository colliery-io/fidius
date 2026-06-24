---
id: p3-ci-mutation-run-on-a-schedule
level: task
title: "P3 — CI mutation run on a schedule (nightly/weekly), report-only"
short_code: "FIDIUS-T-0186"
created_at: 2026-06-23T17:32:41.208852+00:00
updated_at: 2026-06-23T23:20:18.676688+00:00
parent: FIDIUS-I-0033
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0033
---

# P3 — CI mutation run on a schedule (nightly/weekly), report-only

## Parent Initiative

[[FIDIUS-I-0033]] — Phase 3 (mutation testing the core).

## Objective

Wire `cargo-mutants` into CI **on a schedule** (nightly/weekly), report-only —
mutation runs are too slow for per-PR.

## Acceptance Criteria

## Acceptance Criteria

- [x] A scheduled (nightly/weekly) CI mutation run exists.
- [x] It is report-only (never gates a PR).
- [x] Results are surfaced somewhere actionable (job summary + artifact).

## Implementation Notes

Scheduled, not per-PR, because of runtime. Report-only, consistent with the
coverage posture.

### Dependencies
[[FIDIUS-T-0184]], [[FIDIUS-T-0185]] (the mutation passes it schedules).

## Status Updates

**2026-06-23 — implemented.** Added `.github/workflows/mutation.yml`: a separate
**scheduled** workflow (weekly `cron: "0 6 * * 1"` + `workflow_dispatch`), not per-PR
(mutation is too slow). Job (`taiki-e/install-action@cargo-mutants`):
- `cargo mutants --package fidius-core --in-place` (full),
- `cargo mutants --package fidius-macro -f src/ir.rs -f src/interface.rs --in-place`
  (scoped to the IR/codegen logic, matching T-0185's bound),
- `--output mutants-core` / `mutants-macro`.

**Report-only:** `continue-on-error: true` + `timeout-minutes: 120` — never gates.
Surfaced: the surviving-mutant list goes to `$GITHUB_STEP_SUMMARY` (reading
`<dir>/mutants.out/missed.txt`) and the full reports upload as the `mutants-reports`
artifact. `yamllint` clean. Runs on schedule (can't trigger GitHub Actions locally);
`workflow_dispatch` lets a maintainer kick it off on demand. **Phase 3 complete.**