---
id: ws-5-polyglot-wasm-streaming-proof
level: task
title: "WS.5 — Polyglot WASM streaming proof: a non-Rust streaming guest + host E2E"
short_code: "FIDIUS-T-0133"
created_at: 2026-06-19T03:28:11.917537+00:00
updated_at: 2026-06-19T16:22:39.216206+00:00
parent: FIDIUS-I-0026
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0026
---

# WS.5 — Polyglot WASM streaming proof: a non-Rust streaming guest + host E2E

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0026]] · Phase 2 · the moat (polyglot). Depends on [[FIDIUS-T-0131]].

## Objective **[REQUIRED]**

Prove a **non-Rust** WASM guest can serve the same streaming interface — the differentiated "sandboxed connectors without Docker, in any language" pitch. A guest authored in another toolchain (or hand-written WIT + a thin guest) implements the streaming resource contract and is driven by the *same* host `call_streaming` path as the Rust component.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] A **non-Rust streaming guest** (reusing the existing polyglot fixture approach — e.g. the `greeter-js`/`greeter-go`/`greeter-py` pattern, or a hand-authored WIT+guest) exports the streaming resource (`next() -> result<option<T>, plugin-error>`) for a small streaming interface.
- [ ] The host loads it via `PluginHost` and `call_streaming` yields the expected items through `ChunkStream` — identical host code to the Rust path (WS.4), proving the contract is language-neutral.
- [ ] Clean end + (if the toolchain allows) a cancel/drop check; at minimum items + end.
- [ ] Sandbox parity (deny-all WASI + allow-list).
- [ ] Gated/built like the existing polyglot fixtures; documented toolchain requirement.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
- Pick the lowest-friction polyglot path already in the repo (see `tests/wasm-fixtures/greeter-{js,go,py}` + `crates/fidius-host/tests/*polyglot*`). The streaming resource raises the bar over the greeter (which is unary) — a hand-written WAT/WIT guest may be the most controllable proof if a full language toolchain's resource support is immature.
- Reuse the WS.4 host test harness; only the fixture changes.

### Dependencies
- Depends on [[FIDIUS-T-0131]] (host backend) and the WS.1/WS.2 WIT contract. Independent of WS.4 except for shared host test scaffolding.

### Risk Considerations
- Non-Rust toolchains' component-model **resource export** support varies; if the chosen language can't export a resource cleanly yet, fall back to a hand-authored WIT + minimal guest (still "non-Rust-macro" — proves the contract, not the macro) and note the toolchain limitation.

## Status Updates **[REQUIRED]**

### 2026-06-19 — WS.5 complete ✅ (JavaScript / jco)
A **non-Rust** guest serves the same streaming resource:
- **Fixture** `tests/wasm-fixtures/ticker-js` — `ticker.js` implements the `ticker` WIT's streaming resource as a JS class (`class TickStream { next() { ... } }`, `tick(count) -> new TickStream(...)`), built to a component with **`jco componentize`** (`build.sh`) against the *same* `tests/wasm-fixtures/ticker/wit`. Committed `ticker_js.wasm` (≈12.5 MB — embeds a JS engine).
- **E2E** (added to `crates/fidius-host/tests/wasm_streaming_e2e.rs`, **2 JS tests green**, skip-if-`ticker_js.wasm`-absent like the existing `greeter-js`/`greeter-py` polyglot tests): the host drives it with **identical code** to the Rust path — `call_streaming(tick, 5)` → `[0..4]`, plus the 10M-item bounded/cancellable proof. jco maps the exported resource → a JS class, `result<option<u64>, plugin-error>` → return BigInt | undefined / throw.
- **Proves the streaming contract is language-neutral** (the "sandboxed connectors in any language" pitch): the host's `WasmComponentExecutor` resource-pump (WS.3) drives a Rust *and* a JavaScript guest with zero per-language code.
- Resource export via jco 1.0.0 worked directly — no fallback to hand-authored WIT needed. Toolchain: Node + `npx @bytecodealliance/jco` (TinyGo wasn't installed, so Go wasn't attempted; JS is sufficient for the polyglot proof).
- Sandbox parity: same deny-all `instantiate()` path. Slower test (JS engine instantiation) but gated behind `--features wasm,streaming` + skip-if-missing.
- **Verified**: `cargo test -p fidius-host --features wasm,streaming --test wasm_streaming_e2e` → 5/5 (3 Rust + 2 JS).