---
id: p1-angreal-coverage-task-via-cargo
level: task
title: "P1 — `angreal coverage` task via cargo-llvm-cov (workspace + wasm/streaming; local HTML+lcov) + dev-setup docs"
short_code: "FIDIUS-T-0178"
created_at: 2026-06-23T17:32:29.443392+00:00
updated_at: 2026-06-23T22:03:35.659170+00:00
parent: FIDIUS-I-0033
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0033
---

# P1 — `angreal coverage` task via cargo-llvm-cov (workspace + wasm/streaming; local HTML+lcov) + dev-setup docs

## Parent Initiative

[[FIDIUS-I-0033]] — Phase 1 (coverage measurement). This is the instrument every
other phase reads against.

## Objective

Add an `angreal coverage` task that runs `cargo llvm-cov --workspace` across the
feature surface that matters (including `wasm` / `streaming`) and emits an HTML +
lcov report locally, with dev-setup docs covering install and usage.

## Acceptance Criteria

## Acceptance Criteria

- [x] `angreal coverage` produces a per-crate HTML + lcov report locally.
- [x] The task covers the meaningful feature surface (native + `streaming`; `wasm`
      excluded by design — see status).
- [x] Dev-setup docs explain installing `cargo-llvm-cov` and running the task.

## Implementation Notes

- Tool: **`cargo-llvm-cov`** (source-based LLVM coverage) — chosen over tarpaulin
  for accurate source-based coverage and clean multi-crate workspace support.
- The `wasm` feature needs the component toolchain; mirror the existing `wasm`
  job's setup (or scope to native first and add the wasm slice as a follow-on).

### Dependencies
None — this is the first slice of Phase 1; T-0179 and T-0180 build on it.

## Status Updates

**2026-06-23 — implemented.** Added `.angreal/task_coverage.py` (`angreal coverage`):
runs `cargo llvm-cov --no-report --workspace` over the meaningful feature surface
(defaults + `wasm` + `streaming`; `python` off by default, mirroring `angreal test`),
then renders a per-crate summary to stdout + HTML (`target/coverage/html/`) + lcov
(`target/coverage/lcov.info`) from the single run. Flags: `--open`, `--no-wasm`
(skip the component toolchain), `--all-features`. Report-only — never gates on a
threshold; output dir is under git-ignored `/target`. Verified the task registers
(`angreal tree`) and parses (`angreal coverage --help`). Dev-setup docs added to
`docs/how-to/development-workflow.md` (install + usage + report-only note).
**2026-06-23 — corrected + verified; done.** The first run (wasm on) FAILED: the
wasm tests build `wasm32-wasip2` component fixtures at test time, and those
sub-builds inherit cargo-llvm-cov's `-C instrument-coverage` flags, which the wasm
target rejects (instrument-coverage unsupported there) — exactly the
"WASM/component-toolchain coupling" risk the initiative flagged. Per the initiative's
mitigation, scoped the task to the **native + `streaming`** surface by default and
made `wasm` an opt-in `--wasm` flag (documented best-effort). Re-ran `angreal coverage`:
**green**, produced `target/coverage/lcov.info` (590 KB) + `target/coverage/html/index.html`
+ a per-crate summary (TOTAL **76.56%** region / 76.49% line). Docs updated to match
(native default + the wasm-exclusion rationale). All acceptance criteria met.