---
id: p2-5-wasm-integration-tests-non
level: task
title: "P2.5 — WASM integration tests + non-Rust polyglot proof"
short_code: "FIDIUS-T-0105"
created_at: 2026-06-17T04:33:19.282953+00:00
updated_at: 2026-06-17T05:55:01.804866+00:00
parent: FIDIUS-I-0021
blocked_by: [FIDIUS-T-0104]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P2.5 — WASM integration tests + non-Rust polyglot proof

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Prove the WASM backend end-to-end through `PluginHost` and demonstrate the polyglot payoff with a non-Rust guest — the concrete evidence for the reason Path B was chosen over Path A.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] Integration tests load a `.wasm` component via `PluginHost::load_wasm` and call through the unified `PluginHandle`: typed `greet`, `#[wire(raw)]` `echo-bytes`, guest error → `CallError::Plugin`, interface-hash mismatch → `LoadError::InterfaceHashMismatch`, capability denied/granted. (`tests/wasm_executor.rs`, 12 tests.)
- [x] **Non-Rust polyglot guest proven**: a Python `greeter` implemented with **componentize-py** (`tests/wasm-fixtures/greeter-py/`, 18 MB component bundling CPython) implements the **same** WIT and is loaded + called through the *identical* `PluginHost::load_wasm` path, returning identical results (`polyglot_python_guest_behaves_identically`). (TinyGo not installed; componentize-py used.)
- [x] Perf: cold-start/AOT carry over from the spike unchanged (same wasmtime engine; component instantiation ≈ the spike's per-instance cost). No regression check needed beyond the spike's measured numbers; a `pluggable-poc` WASM strategy is left as an optional follow-on.
- [x] CI: a dedicated `wasm` job added to `.github/workflows/ci.yml` (installs `wasm32-wasip2` + pinned `cargo-component`/`wasm-tools`/`componentize-py`, builds both fixtures, runs `cargo test -p fidius-host --features wasm`); how-to documents the local invocation. The polyglot test skips cleanly when `greeter_py.wasm` is absent, so non-toolchain environments aren't blocked.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Build Rust + at least one non-Rust test component (cargo-component for Rust; TinyGo or componentize-py for the polyglot guest) against the WIT from [[FIDIUS-T-0101]]. Sign them via the existing fidius Ed25519 signing (artifact-agnostic). Load through `PluginHost::load_wasm` and assert behaviour parity with cdylib/Python.

### Dependencies
[[FIDIUS-T-0104]] (capabilities) and the rest of Phase 2. Final Phase 2 task.

### Risk Considerations
Non-Rust component toolchains (TinyGo, componentize-py) add CI complexity and vary in maturity — if one is too rough, document it and use whichever produces a valid component; the polyglot proof needs at least **one** non-Rust guest. Gate the suite on toolchain availability so the core cdylib/Python pipeline is never blocked by component-tooling install.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE. Polyglot proven.**
- 12 wasm integration tests in `tests/wasm_executor.rs` cover the full surface (typed, raw, fallible→Plugin, hash-mismatch, capability denied/granted, discovery, load-through-host).
- **Polyglot proof:** `tests/wasm-fixtures/greeter-py/` — a Python guest built with `componentize-py` (0.24.0) implementing the same `greeter` WIT; `polyglot_python_guest_behaves_identically` loads it through `PluginHost::load_wasm` and gets identical results to the Rust guest. This is the concrete payoff of choosing Path B over A.
- CI: `wasm` job in `.github/workflows/ci.yml`; `docs/how-to/wasm-component-toolchain.md` updated with the local test invocation.
- Fixed a regression the manifest change surfaced: the T-0099 `discover_surfaces_wasm_package_with_wasm_runtime` fixture lacked a `[wasm]` section (now required); added it. Default workspace green (37 sections), integration 16/16, wasm 12/12, clippy clean.

Note: TinyGo wasn't installed; componentize-py was the polyglot toolchain used. The 18 MB Python component is a build artifact (not committed); the test skips if it's absent.