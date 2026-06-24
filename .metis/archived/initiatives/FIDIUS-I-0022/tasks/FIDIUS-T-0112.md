---
id: g-3-verify-wasm32-wasip2-builds
level: task
title: "G.3 — Verify wasm32-wasip2 builds (fidius-guest + macro author crate) + full regression"
short_code: "FIDIUS-T-0112"
created_at: 2026-06-17T11:25:26.026213+00:00
updated_at: 2026-06-17T11:48:27.429597+00:00
parent: FIDIUS-I-0022
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0022
---

# G.3 — Verify wasm32-wasip2 builds (fidius-guest + macro author crate) + full regression

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0022]]

## Objective **[REQUIRED]**

Verify the split achieved its goal: `fidius-guest` and a real macro-using author crate both build for `wasm32-wasip2`, with the full native suite still green.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `cargo check -p fidius-guest --target wasm32-wasip2` passes (T-0110).
- [x] A macro author fixture (`tests/wasm-fixtures/macro-greeter`: `#[plugin_interface]`+`#[plugin_impl]`, `crate = "fidius_guest"`, deps `fidius-guest`+`fidius-macro`+`wit-bindgen`) **builds to a valid wasm component** — `cargo build --target wasm32-wasip2 --release` + `wasm-tools validate --features component-model` ✓ (44 KB).
- [x] The component **exports `fidius:greeter/greeter@0.1.0`** (matches the macro descriptor's `interface_export`) with the `plugin-error` record + `greet`/`echo`/`fidius-interface-hash` — the auto-export adapter (T-0106) compiled for the first time.
- [x] Full native regression green (`cargo test --workspace`: 39 ok sections, 0 failed).
- [ ] Load-through-host round-trip + hash-match: deferred to FIDIUS-T-0106 (its acceptance), now unblocked.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
New standalone fixture `tests/wasm-fixtures/macro-greeter` (out of the workspace; `[lib] crate-type=["cdylib"]`). Build artifacts gitignored. Surfaced two real adapter fixes (now in T-0106 codegen): the `crate` attr takes a **string literal**; the adapter needs `use super::*;` + the hash const path through the companion module.

### Dependencies
Depends on [[FIDIUS-T-0110]] + [[FIDIUS-T-0111]]. Completing it satisfies FIDIUS-I-0022 and unblocks [[FIDIUS-T-0106]].

### Risk Considerations
Follow-on: add the macro-greeter wasm build to the CI `wasm` job as a regression guard; do the load-through-host E2E under T-0106.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.** `fidius-guest` + the `macro-greeter` author fixture both build for `wasm32-wasip2`; the fixture produces a valid 44 KB component exporting `fidius:greeter/greeter@0.1.0`. The auto-export adapter compiled + produced a real component for the first time. Native suite green (39 ok). Load-through-host E2E deferred to the now-unblocked T-0106.