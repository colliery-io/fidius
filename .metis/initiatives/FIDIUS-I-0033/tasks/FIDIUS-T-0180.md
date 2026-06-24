---
id: p1-baseline-analysis-capture-per
level: task
title: "P1 — Baseline analysis: capture per-crate numbers, document worst gaps, file follow-up backlog items"
short_code: "FIDIUS-T-0180"
created_at: 2026-06-23T17:32:32.707801+00:00
updated_at: 2026-06-23T22:05:43.136206+00:00
parent: FIDIUS-I-0033
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0033
---

# P1 — Baseline analysis: capture per-crate numbers, document worst gaps, file follow-up backlog items

## Parent Initiative

[[FIDIUS-I-0033]] — Phase 1 (coverage measurement).

## Objective

Capture the first coverage baseline: record per-crate numbers, enumerate the worst
gaps in writing, and file the highest-value gaps as follow-up backlog items.

## Acceptance Criteria

## Acceptance Criteria

- [x] A written coverage baseline exists with per-crate numbers.
- [x] The worst gaps are explicitly enumerated (notably the `fidius` facade with 0
      tests and known error-path gaps).
- [x] The highest-value gaps are filed as follow-up backlog items.

## Implementation Notes

Coverage is a *map*, not a target — do not chase 100% on FFI/`unsafe`/platform paths
that are legitimately hard to cover. The baseline + follow-up backlog items are the
forcing function (report-only posture means there is no gate to lean on yet).

### Dependencies
[[FIDIUS-T-0178]], [[FIDIUS-T-0179]] (needs the instrument running first).

## Status Updates

**2026-06-23 — baseline captured (native + `streaming`, report-only).** Source:
`angreal coverage` → `cargo llvm-cov report --summary-only`. **TOTAL: 76.56% region
/ 73.01% function / 76.49% line** (13,319 regions, 3,122 uncovered). Artifacts:
`target/coverage/lcov.info` + `target/coverage/html/`. The `wasm` feature is NOT in
this number (instrument-coverage can't build the wasm fixtures — see T-0178), so
`executor/wasm.rs` is a known blind spot pending the wasm-coverage follow-on.

Per-crate / per-file region coverage (worst first):

| File | Region % | Note |
| --- | --- | --- |
| `fidius/src/lib.rs` (facade) | **0%** | 0 own tests → **T-0189** |
| `fidius-host/src/types.rs` | 25% | tiny (12 regions) |
| `fidius-python/src/handle.rs` | 26% | → **T-0191** |
| `fidius-python/src/loader.rs` | 46% | → **T-0191** |
| `fidius-host/src/executor/cdylib.rs` | 49% | → **T-0192** |
| `fidius-host/src/package.rs` | 53% | manifest/unpack error paths |
| `fidius-macro/src/lib.rs` | 62% | |
| `fidius-python/src/value_bridge.rs` | 64% | → **T-0191** |
| `fidius-host/src/arena.rs` | 67% | |
| `fidius-host/src/host.rs` | 70% | |
| `fidius-host/src/loader.rs` | 72% | |
| `fidius-host/src/signing.rs` | 74% | |
| `fidius-host/src/arch.rs` | 77% | |
| `fidius-macro/src/impl_macro.rs` | 78% | |
| `fidius-wit/src/generate.rs` | 79% | |
| (well-covered ≥85%) | | `stream.rs` 96%, `client_stream.rs` 96%, `interface.rs` 99%, `wire.rs` 100%, `fidius-wit/lib.rs` 93%, `fidius-test/*` 95–98% |

**Worst gaps + disposition:**
- `fidius` facade (0% own tests) → already owned by **T-0189** (in this initiative).
- Python backend dispatch/error paths (handle 26% / loader 46% / value_bridge 64%)
  → filed **[[FIDIUS-T-0191]]** (tech-debt backlog).
- cdylib executor error paths (`executor/cdylib.rs` 49%) → filed **[[FIDIUS-T-0192]]**
  (tech-debt backlog).
- wasm executor blind spot → folded into the documented wasm-coverage follow-on
  (T-0178/T-0179), not a separate item.
- `host/types.rs` 25% is only 12 regions (trivial getters) — not worth a dedicated
  item; will rise incidentally with T-0190's matrix work.

All acceptance criteria met. Baseline lives here (Metis system of record) so it
survives; re-run `angreal coverage` to refresh.