---
id: p2-ci-fuzz-smoke-time-boxed-per-pr
level: task
title: "P2 — CI fuzz smoke: time-boxed per-PR run, corpus persistence, docs for longer local/scheduled campaigns"
short_code: "FIDIUS-T-0183"
created_at: 2026-06-23T17:32:36.970005+00:00
updated_at: 2026-06-23T22:18:33.414644+00:00
parent: FIDIUS-I-0033
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0033
---

# P2 — CI fuzz smoke: time-boxed per-PR run, corpus persistence, docs for longer local/scheduled campaigns

## Parent Initiative

[[FIDIUS-I-0033]] — Phase 2 (fuzzing the wire/FFI boundary).

## Objective

Wire a **time-boxed fuzz smoke** into CI per PR (find crashers fast without long
jobs), persist the corpus across runs, and document how to run longer local /
scheduled campaigns.

## Acceptance Criteria

## Acceptance Criteria

- [x] A per-PR CI fuzz smoke runs each target for a bounded time budget.
- [x] The corpus persists across CI runs.
- [x] Docs explain longer local/scheduled fuzz campaigns.
- [x] The smoke is report-only / non-flaky (a found crasher is surfaced loud +
      uploaded, but `continue-on-error` keeps it non-gating).

## Implementation Notes

Pin and isolate the nightly toolchain used for `cargo-fuzz`. Per-PR is a *smoke*
only; full campaigns run scheduled/local.

### Dependencies
[[FIDIUS-T-0181]], [[FIDIUS-T-0182]] (the targets it runs).

## Status Updates

**2026-06-23 — implemented.** Added a `fuzz-smoke` job to `.github/workflows/ci.yml`:
pinned nightly (`nightly-2026-03-09`, isolated to this job), `taiki-e/install-action@cargo-fuzz`,
`actions/cache` on `crates/fidius-host/fuzz/corpus` (keyed by run id + `restore-keys`,
so the corpus grows across runs from the committed seeds), then a loop running each
of the 4 targets for **60s** (`-max_total_time=60`). A crasher writes to
`$GITHUB_STEP_SUMMARY` (with a reproduce-locally command) and uploads
`fuzz/artifacts/` via `actions/upload-artifact`; `continue-on-error: true` keeps it
**report-only / non-gating** per the locked posture (one line to flip to gating).

Docs: added a "Fuzzing" section to `docs/how-to/development-workflow.md` (target
table, local `cargo fuzz run`, seed-corpus location, `cmin`, crasher reproduction,
CI-smoke behavior).

Verified: `yamllint` passes; the per-target command is exactly the locally-verified
`cargo fuzz run <t> -- -max_total_time=…` (all 4 ran clean here). GitHub Actions runs
on push, so the live job shows on the next push. **Phase 2 (fuzzing) complete.**

**2026-06-23 — moved off the per-PR gate (maintainer request).** Fuzz is expensive,
so the job moved from `ci.yml` to `.github/workflows/nightly.yml` (**nightly schedule
+ `v*` tags + `workflow_dispatch`**, no longer per-PR). Since it's no longer gating
PRs, the per-target budget was bumped 60s → **120s**. Corpus cache + crasher
surfacing/upload + report-only (`continue-on-error`) all unchanged. The docs "Fuzzing"
section was updated to describe a nightly fuzz (not per-PR smoke).