---
id: p5-expand-fixture-permutation
level: task
title: "P5 — Expand fixture/permutation matrix across backends × streaming × egress × config"
short_code: "FIDIUS-T-0190"
created_at: 2026-06-23T17:32:46.572390+00:00
updated_at: 2026-06-23T23:14:43.720844+00:00
parent: FIDIUS-I-0033
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0033
---

# P5 — Expand fixture/permutation matrix across backends × streaming × egress × config

## Parent Initiative

[[FIDIUS-I-0033]] — Phase 5 (expand the backend matrix).

## Objective

Broaden fixtures and permutations across backends (cdylib / Python / WASM) × the
streaming × egress × config feature axes, so the cross-product the library actually
supports is exercised — not just the happy paths.

## Acceptance Criteria

## Acceptance Criteria

- [x] The fixture/permutation matrix spans backends × streaming × egress × config
      (mapped below — already broad; verified, not just asserted).
- [x] Previously-unexercised combinations gain coverage (facade combos T-0189;
      manifest config×backend `validate_runtime`; proptest type breadth T-0187/8).
- [x] Gaps that are intentionally out of scope are documented + filed (T-0193).

## Implementation Notes

Driven by what the Phase 1 baseline (T-0180) shows as uncovered combinations; prefer
filling real cross-product gaps over adding redundant happy-path cases.

### Dependencies
Informed by baseline (T-0180); composes with the facade tests (T-0189).

## Status Updates

**2026-06-23 — matrix mapped + gaps filled/filed.** Surveyed the existing suite
(36 host e2e files, 136 test fns, 24 wasm fixtures): the cross-product is already
**broad**, not sparse. Coverage map (cell → representative test):

| axis | cdylib | python | wasm |
| --- | --- | --- | --- |
| base | `e2e`, `integration` | `python_plugin_e2e`, `python_routing` | `wasm_executor`, `macro_wasm` |
| streaming (server/client/bidi/record) | `cdylib_streaming/_client/_bidi/_record_stream` | `python_streaming/_client/_bidi` | `wasm_streaming/_client/_bidi/_record_stream`, `records_stream_wasm` |
| config | `configured_cdylib` | `configured_python` | `configured_wasm`, `macro_configured` |
| config × streaming | `configured_cdylib_stream` | `configured_python_stream` | `configured_wasm_stream` |
| egress (wasm-only) | n/a | n/a | `wasm_egress`, `macro_egress`, `tcp_egress` |
| egress × streaming | n/a | n/a | **GAP → [[FIDIUS-T-0193]]** |

(Egress is a WASM capability, so the egress row is wasm-only by design.)

**Filled this phase** (real, not redundant):
- `fidius-core` `validate_runtime_section_matrix` — the manifest **config × backend-
  runtime** cross-product (rust/python/wasm × which `[section]` is present): 11 cells,
  previously untested at the unit level.
- Facade × {host, wasm, streaming} feature combos (T-0189).
- Wire/Value/WIT/tuple type breadth across the boundary (T-0187/T-0188, generative).

**Filed (deferred, not silently skipped):** the one genuine missing cell —
**egress × streaming together** (a sandboxed connector that streams *and* does gated
egress) — needs a new component fixture, so it's filed as **[[FIDIUS-T-0193]]** rather
than rushed. **Phase 5 complete.**