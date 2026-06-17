---
id: g-1-scaffold-fidius-guest-move
level: task
title: "G.1 — Scaffold fidius-guest + move guest modules + fidius-core re-exports (the split)"
short_code: "FIDIUS-T-0110"
created_at: 2026-06-17T11:25:23.136950+00:00
updated_at: 2026-06-17T11:31:52.445749+00:00
parent: FIDIUS-I-0022
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0022
---

# G.1 — Scaffold fidius-guest + move guest modules + fidius-core re-exports (the split)

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0022]]

## Objective **[REQUIRED]**

Create the wasm-buildable `fidius-guest` crate, move the guest-essential modules into it, and have `fidius-core` re-export them so every `fidius_core::*` path is unchanged.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `crates/fidius-guest` created (version `0.2.1`, lockstep with core for `ABI_VERSION`); deps limited to `serde`/`serde_json`/`bincode`/`thiserror` — no host-only deps.
- [x] `hash`, `descriptor`, `value`, `wire`, `error`, `status`, `wasm_descriptor`, `python_descriptor` moved into `fidius-guest` (verified no host-only refs first).
- [x] `fidius-core` depends on + re-exports `fidius_guest::{…}`; existing `fidius_core::*` paths and the facade re-exports resolve unchanged (no public-API churn).
- [x] **`cargo check -p fidius-guest --target wasm32-wasip2` passes** (the gate proving the split worked).
- [x] Full workspace builds + tests green (39 ok sections, 0 failures; the moved unit tests now run in `fidius-guest`, 11 passed).

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
`git mv` the 8 modules into `crates/fidius-guest/src`; new `fidius-guest` crate + lib; `fidius-core` `pub use fidius_guest::{…}` + keep `pub use descriptor::*` / `PluginError` / `status::*` / value re-exports (they resolve via the re-exported module names). `package`/`registry`/`async_runtime` + the `inventory` re-export stay in core.

### Dependencies
First task of FIDIUS-I-0022. Blocks [[FIDIUS-T-0111]] and [[FIDIUS-T-0112]]; unblocks [[FIDIUS-T-0106]] once the initiative completes.

### Risk Considerations
`ABI_VERSION` derives from `CARGO_PKG_VERSION` — `fidius-guest` pinned to `0.2.1` to keep it at 200. Pre-checked that no moved module references host-only modules.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.** `fidius-guest` created + 8 modules moved + `fidius-core` re-exports. `cargo check -p fidius-guest --target wasm32-wasip2` ✓ (the gate); native `fidius-guest` + `fidius-core` + full workspace build; `cargo test --workspace` 39 ok / 0 failed (moved tests run in `fidius-guest`, 11 passed). No public-API churn.