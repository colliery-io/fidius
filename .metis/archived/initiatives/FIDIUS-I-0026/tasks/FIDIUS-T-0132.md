---
id: ws-4-rust-wasm-streaming-e2e
level: task
title: "WS.4 — Rust WASM streaming E2E: component fixture + items/bounded/drop-cancel-runs-dtor through PluginHost"
short_code: "FIDIUS-T-0132"
created_at: 2026-06-19T03:28:10.512473+00:00
updated_at: 2026-06-19T04:28:25.416149+00:00
parent: FIDIUS-I-0026
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0026
---

# WS.4 — Rust WASM streaming E2E: component fixture + items/bounded/drop-cancel-runs-dtor through PluginHost

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0026]] · Phase 2 capstone (Rust path). Depends on [[FIDIUS-T-0130]], [[FIDIUS-T-0131]].

## Objective **[REQUIRED]**

Prove WASM server-streaming end-to-end with a real Rust component: a `Ticker`-style streaming guest compiled to `wasm32-wasip2`, loaded via `PluginHost`, driven through `call_streaming`. Validate items, bounded memory/backpressure, and drop-cancel-runs-the-guest-destructor — the WASM analog of ST.5.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] A Rust **streaming component fixture** (under `tests/wasm-fixtures/`) implements a streaming interface (`tick(count) -> fidius::Stream<u64>`) via `#[plugin_impl]`, builds to a component, and is packaged/loaded like the existing wasm fixtures.
- [ ] **Yields all items**: `call_streaming(tick, N)` → `[0..N]` through `ChunkStream`.
- [ ] **Bounded + cancellable**: a huge `N` (e.g. 10M), pull a few then drop → returns promptly (REQ-003/NFR-003 + cancel), mirroring ST.5's deterministic behavioural proof.
- [ ] **Drop-cancel runs the guest resource destructor** (REQ-002/D3): the guest's resource `Drop`/dtor sets an observable side-effect (e.g. increments a host-visible counter via an allowed capability, or writes through a host import) that the test asserts ran after the stream is dropped mid-flight.
- [ ] **Sandbox parity**: the fixture runs under deny-all WASI + the manifest allow-list; no FS, exactly as unary WASM.
- [ ] Runs under `--features wasm,streaming`; CI builds the component (gated like existing wasm fixtures).

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
- Model on `tests/wasm-fixtures/greeter` (build/package pattern) + the ST.5 `py-ticker` test shape (`stage`, `load`, `call_streaming`, assert).
- For the dtor side-effect without FS: simplest is the huge-stream behavioural proof for cancel; for a *direct* dtor assertion, consider a host-import capability the guest calls from its resource `Drop`, or assert via a deterministic signal the executor exposes. If a clean host-visible signal is hard, lean on the behavioural (huge-stream) proof + the WS.3 unit-level resource-drop test and note it.

### Dependencies
- Depends on [[FIDIUS-T-0130]] (component exports the resource) + [[FIDIUS-T-0131]] (host pumps it). Phase-2 Rust exit gate.

### Risk Considerations
- Asserting the guest dtor ran from the host is the fiddly part (no FS in the sandbox). Prefer a deterministic, capability-clean signal; fall back to behavioural + unit proof if needed (documented).
- Component build in CI must be gated/available like the existing wasm fixtures (cargo-component / wasm-tools toolchain).

## Status Updates **[REQUIRED]**

### 2026-06-19 — WS.4 complete ✅ (delivered with WS.2 Stage C)
The Rust WASM streaming E2E landed as the proof for the macro codegen:
- **Fixture** `tests/wasm-fixtures/macro-ticker` — a fidius plugin written purely with the macros: `#[plugin_impl] fn tick(&self, count: u32) -> fidius_guest::Stream<u64> { Stream::from_iter(0..count) }`. Builds to a component via `cargo build --target wasm32-wasip2`.
- **E2E** `crates/fidius-host/tests/macro_wasm_streaming.rs` (**3 tests green**): loaded via `PluginHost::load_wasm` against the macro-emitted `Ticker_WASM_DESCRIPTOR` (hash auto-matches the component's `fidius-interface-hash`); `call_streaming(tick, 5)` → `[0,1,2,3,4]`; **10M-item pull-3-then-drop returns in <0.3s** (bounded + backpressure + cancel through the wasmtime resource stack); descriptor marks `tick` streaming.
- Plus the hand-authored-fixture E2E from WS.3 (`tests/wasm-fixtures/ticker` + `wasm_streaming_e2e.rs`, 3 tests) covers the same path from a non-macro component.
- **Sandbox parity**: runs through the standard `instantiate()` (deny-all WASI + allow-list), same as unary WASM.
- **Scope delta (deliberate, like ST.5)**: drop-cancel is proven *behaviourally* (the 10M test would hang/OOM if not lazily pulled + torn down) + the WS.3 unit-level `resource_drop` path, rather than a host-visible guest-dtor side-effect (which needs an FS or host-import capability the deny-all sandbox withholds). The behavioural + drop-path proof is deterministic and capability-clean; a direct dtor-counter via a host import is a possible follow-on.
- **Verified**: `cargo test -p fidius-host --features wasm,streaming` all green; `angreal test`/`build` (default) green.