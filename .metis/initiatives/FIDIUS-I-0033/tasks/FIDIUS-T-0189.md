---
id: p5-first-class-tests-for-the
level: task
title: "P5 — First-class tests for the fidius facade crate across feature combos (currently 0 own tests)"
short_code: "FIDIUS-T-0189"
created_at: 2026-06-23T17:32:45.461560+00:00
updated_at: 2026-06-23T23:11:12.884108+00:00
parent: FIDIUS-I-0033
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0033
---

# P5 — First-class tests for the fidius facade crate across feature combos (currently 0 own tests)

## Parent Initiative

[[FIDIUS-I-0033]] — Phase 5 (expand the backend matrix).

## Objective

Give the `fidius` facade crate first-class tests of its own (it currently has only
re-export compile-guards, 0 real tests) across its feature combinations (`host`,
`wasm`, `streaming`).

## Acceptance Criteria

## Acceptance Criteria

- [x] The `fidius` facade has first-class tests beyond re-export compile-guards.
- [x] Tests exercise the `host`, `wasm`, and `streaming` feature combinations.
- [x] The facade's public surface (the thing downstream consumers actually use) is
      covered.

## Implementation Notes

The facade is what downstream host apps / white-label re-export crates depend on;
its 0-test state is the single worst gap called out in the baseline.

### Dependencies
Baseline (T-0180) names this as a top gap; otherwise standalone.

## Status Updates

**2026-06-23 — implemented + verified.** Added 4 facade test files under
`crates/fidius/tests/` (was 0):
- `facade_core.rs` (no feature) — wire round-trip, `Value` bridge, `PluginError`,
  `hash::fnv1a` determinism, `ABI_VERSION`/`FIDIUS_MAGIC`, and the headline:
  defining an interface via `fidius::plugin_interface(crate = "fidius")` and asserting
  the generated hash — proving the macro's codegen resolves against the facade path.
  (6 tests.)
- `facade_host.rs` (`host`) — `PluginHost::builder().search_path(..).build()` builds
  an empty host through the facade + a host-type re-export guard. (2 tests.)
- `facade_wasm.rs` (`wasm`) — implements `EgressPolicy` (naming `http_types::request::Parts`)
  and asserts the **default-deny `authorize_tcp`/`authorize_udp`** (FIDIUS-I-0033) +
  `EgressDenied`. (2 tests.)
- `facade_streaming.rs` (`streaming`) — re-export guard for `ChunkStream`/`StreamExecutor`/`Stream<T>`
  (runtime streaming behaviour is owned by host's e2e). (1 test.)

These are real consumer-style exercises, not `use`-guards, so a broken/renamed
re-export fails a test. Verified across combos: default, `--features wasm`,
`--features streaming` — all green. fmt + clippy clean (codegen-shape lints scoped-
allowed in `facade_core`). NB: the facade `lib.rs` is pure `pub use`, so its coverage
% stays ~structural; the value here is guarding the surface consumers depend on.