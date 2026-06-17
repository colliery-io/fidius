---
id: p1-1-provision-wasm-component
level: task
title: "P1.1 — Provision WASM component toolchain (cargo-component, wasm-tools, wasm32-wasip2) + dev-setup docs"
short_code: "FIDIUS-T-0094"
created_at: 2026-06-17T03:23:54.575785+00:00
updated_at: 2026-06-17T03:35:50.435401+00:00
parent: FIDIUS-I-0021
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P1.1 — Provision WASM component toolchain (cargo-component, wasm-tools, wasm32-wasip2) + dev-setup docs

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Provision the WASM Component Model build toolchain so Phase 2 component work is unblocked, and document the dev setup. The main fidius repo has no Flox env — it uses ambient rustup/cargo, so tools install globally. (Most of this was started during decomposition: the `wasm32-wasip2` target was added and `cargo install cargo-component wasm-tools` was kicked off.)

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `wasm32-wasip2` target installed (`rustup target add wasm32-wasip2`).
- [x] `cargo-component` installed and on PATH — `cargo-component 0.21.1`.
- [x] `wasm-tools` installed and on PATH — `wasm-tools 1.252.0`.
- [x] A trivial component builds and validates end-to-end — `cargo component build` produced a component; `wasm-tools validate --features component-model` → VALID; `wasm-tools component wit` round-tripped.
- [x] Dev-setup docs list the required toolchain with install commands + verified versions — `docs/how-to/wasm-component-toolchain.md` (linked from `development-workflow.md`).
- [x] CI decision documented: do **not** add a CI job until Phase 2 has component code; then a separate job installs the `wasm32-wasip2` target + pinned `cargo-component`/`wasm-tools`. Captured in the how-to's CI section.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
`rustup target add wasm32-wasip2`; `cargo install cargo-component wasm-tools`. No Flox manifest in the main repo. Optionally add a `rust-toolchain.toml` pin and/or an angreal task that verifies the toolchain is present. Record exact installed versions in docs.

### Dependencies
None (foundational). Unblocks the Phase 2 WASM component executor and any component-build steps. Independent of the other Phase 1 tasks.

### Risk Considerations
`cargo-component`/`wasm-tools` track the fast-moving Component Model — pin versions to avoid drift. CI runners need the same tools; note non-trivial install/compile time. Keep this isolated from the trait refactor so it can land first.

## Status Updates **[REQUIRED]**

**2026-06-16 — toolchain installed and verified working.** `wasm32-wasip2` target added; `cargo install cargo-component wasm-tools` completed (exit 0). Verified: `cargo-component 0.21.1`, `wasm-tools 1.252.0`; a `cargo component new --lib` smoke project builds a component that `wasm-tools validate --features component-model` accepts and `wasm-tools component wit` round-trips. Installed globally into `~/.cargo/bin` (no Flox env in this repo). **Phase 2 note:** a default `cargo component new` imports `wasi:cli`/`wasi:io` — fidius's deny-by-default capability policy must strip these.

**2026-06-16 — COMPLETE.** Added `docs/how-to/wasm-component-toolchain.md` (install commands, verified version table, end-to-end verify steps, deny-by-default capability note) and linked it from `docs/how-to/development-workflow.md` prerequisites. CI decision recorded in that doc: no CI job until Phase 2 has component code, then a separate job with the `wasm32-wasip2` target + pinned `cargo-component`/`wasm-tools`. Deferred `rust-toolchain.toml`/angreal verify task as optional (not needed yet). All acceptance criteria met.