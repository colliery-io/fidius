---
id: p2-1-wit-interface-mapping
level: task
title: "P2.1 — WIT interface mapping + bindings strategy + interface-hash-equivalent validation"
short_code: "FIDIUS-T-0101"
created_at: 2026-06-17T04:33:10.778143+00:00
updated_at: 2026-06-17T04:49:04.349477+00:00
parent: FIDIUS-I-0021
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P2.1 — WIT interface mapping + bindings strategy + interface-hash-equivalent validation

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Define how a fidius `#[plugin_interface]` projects onto a WIT `world`/`interface`, choose the bindings strategy, and preserve interface-hash-equivalent validation for WASM components (parity with the cdylib `interface_hash` and the Python `__interface_hash__` constant). Foundational design task for the WASM backend.

**Design decision to settle at task start (human-in-the-loop):** WIT **generated** from the Rust interface (the macro emits `.wit` — codegen itself is Phase 3) vs **authored** by the plugin author and validated against the interface. For Phase 2, a hand-written WIT for one test interface is acceptable regardless.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] Type mapping documented in `docs/explanation/wasm-component-abi.md` — a `fidius_core::Value` ↔ WIT ↔ wasmtime `Val` table (bool, sized ints, floats, char, string, bytes→`list<u8>`, list/tuple, record, option, variant/enum, unit, map→`list<tuple<k,v>>`), plus `#[wire(raw)]`→`list<u8>` and the `result<T, plugin-error>` mapping for fallible methods (→ `CallError::Plugin`; traps → `CallError::Backend`).
- [x] **Bindings strategy decided: hand-authored WIT + dynamic `component::Val` dispatch** (no host build-time wit-bindgen codegen); host dispatches by index/name via `Func::call(&[Val])`, matching fidius's existing by-index model. Macro-generated WIT deferred to Phase 3 (T-0013). Rationale recorded in the doc.
- [x] Interface-hash validation defined: the component exports `fidius-interface-hash: func() -> u64` (parity with cdylib `interface_hash` / Python `__interface_hash__`); host calls it at load and rejects a mismatch at `LoadError` level. Documented as an integrity check (signing is the security boundary).
- [x] Reference WIT committed: `tests/wasm-fixtures/greeter/wit/world.wit` — a `greeter` interface (typed `greet`, multi-arg fallible `add`, `#[wire(raw)]` `echo-bytes`, `fidius-interface-hash`) + `greeter-plugin` world. Validated with `wasm-tools component wit` (parses + round-trips).

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Study wit-bindgen host-side `generate!` and the Component Model value space. Map the fidius interface IR → WIT (the variant set of `fidius_core::Value` was deliberately shaped to the Component Model value space, so this should be close to 1:1). Decide the hash carrier (a top-level exported func returning the u64 hash mirrors the Python constant and is index-independent).

### Dependencies
Phase 1 complete (the `PluginExecutor`/`ValueExecutor` traits, the `Backend` enum on `PluginHandle`, and `fidius_core::Value` — all DONE). Toolchain (`cargo-component`/`wasm-tools`/`wasm32-wasip2`) from [[FIDIUS-T-0094]]. Spike data: [[FIDIUS-T-0093]] / `wasm-spike/FINDINGS.md`. Per ADR [[FIDIUS-A-0003]] (Path B). Blocks [[FIDIUS-T-0102]].

### Risk Considerations
WIT type coverage must match the types fidius interfaces actually use. The hash carrier is an integrity check (catch wrong-interface), not a security control — signing provides security. Component Model tooling churns; pin versions (per T-0094).

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.**
- Design doc: `docs/explanation/wasm-component-abi.md` (strategy, dispatch model, type-mapping table, fallible-method mapping, interface-hash carrier).
- Reference WIT: `tests/wasm-fixtures/greeter/wit/world.wit`, validated by `wasm-tools component wit`.

**Design decision (made in lieu of the flagged human-in-the-loop, per "run it"):** Phase 2 uses **hand-authored WIT + dynamic `component::Val` dispatch** — no host-side wit-bindgen codegen. This keeps the host generic/by-index (one path for all interfaces, like cdylib/Python) and avoids coupling the host build to a WIT toolchain version. Macro-emitted WIT is deferred to Phase 3 (the Rust-author ergonomic path). Reversible if review prefers generated bindings; flag on the initiative if so.