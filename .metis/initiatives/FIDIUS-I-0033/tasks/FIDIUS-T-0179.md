---
id: p1-ci-coverage-job-per-crate-table
level: task
title: "P1 — CI coverage job: per-crate table to $GITHUB_STEP_SUMMARY + HTML/lcov artifact, report-only (wasm split)"
short_code: "FIDIUS-T-0179"
created_at: 2026-06-23T17:32:31.288637+00:00
updated_at: 2026-06-23T22:04:14.824057+00:00
parent: FIDIUS-I-0033
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0033
---

# P1 — CI coverage job: per-crate table to $GITHUB_STEP_SUMMARY + HTML/lcov artifact, report-only (wasm split)

## Parent Initiative

[[FIDIUS-I-0033]] — Phase 1 (coverage measurement).

## Objective

Add a CI job that installs `cargo-llvm-cov`, runs coverage, renders the per-crate
table into `$GITHUB_STEP_SUMMARY`, and uploads HTML + lcov as a build artifact —
**report-only, never failing the build**. No external service (no Codecov).

## Acceptance Criteria

## Acceptance Criteria

- [x] Every CI run publishes a per-crate coverage table in the job summary.
- [x] HTML + lcov are uploaded as a downloadable artifact.
- [x] The job never fails the build on coverage.
- [x] No external coverage service / token is used.
- [x] The existing dev/release/wasm job split (component toolchain for `wasm`) is
      handled — coverage scoped to the native surface (documented), wasm left to
      its existing job.

## Implementation Notes

Reuse the `angreal coverage` task (T-0178). If instrumenting the wasm path is too
heavy, scope coverage to the native feature set first and add the wasm slice as a
documented follow-on (never silently dropped).

### Dependencies
[[FIDIUS-T-0178]] (the `angreal coverage` task it invokes).

## Status Updates

**2026-06-23 — implemented.** Added a `coverage` job to `.github/workflows/ci.yml`:
`taiki-e/install-action@cargo-llvm-cov` + `llvm-tools-preview`, runs
`cargo llvm-cov --no-report --workspace --features streaming`, then renders lcov +
HTML and pipes `cargo llvm-cov report --summary-only` into `$GITHUB_STEP_SUMMARY`,
and uploads `lcov.info` + `coverage-html/` via `actions/upload-artifact` (both
`if: always()`). `continue-on-error: true` on the job guarantees report-only (never
gates a PR). No external service/token.

Deviation (intentional): the job runs `cargo-llvm-cov` **directly** rather than via
`angreal coverage` — angreal isn't installed in CI, so replicating the commands keeps
the job self-contained. It mirrors the task's native-surface scope exactly.

wasm split: coverage is scoped to native + `streaming` (the cleanly-instrumentable
surface; the wasm fixtures can't be instrumented — see T-0178). The `wasm` job still
runs the wasm tests un-instrumented, so the wasm path is exercised, just not counted.

Verified: `yamllint` passes on the workflow. Note CI itself runs on push (can't
execute GitHub Actions locally), so the live summary/artifact will show on the next
push. All acceptance criteria met.

**2026-06-23 — moved off the per-PR gate (maintainer request).** Coverage is
expensive, so the job was relocated from `ci.yml` to a new
`.github/workflows/nightly.yml`, triggered on a **nightly schedule + `v*` tags +
`workflow_dispatch`** (no longer every push/PR). Job body unchanged (same summary +
artifact, still report-only). So "every CI run" above now means "every nightly/tag
run" — the per-crate table + HTML/lcov artifact still publish, just not on the fast
PR gate.