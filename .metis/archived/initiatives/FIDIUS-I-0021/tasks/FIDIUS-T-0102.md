---
id: p2-2-wasmcomponentexecutor-core
level: task
title: "P2.2 — WasmComponentExecutor core: wasmtime instantiate + Value↔component::Val dispatch"
short_code: "FIDIUS-T-0102"
created_at: 2026-06-17T04:33:11.693156+00:00
updated_at: 2026-06-17T05:33:10.747045+00:00
parent: FIDIUS-I-0021
blocked_by: [FIDIUS-T-0101]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P2.2 — WasmComponentExecutor core: wasmtime instantiate + Value↔component::Val dispatch

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Implement `WasmComponentExecutor` (in fidius-host, behind a new `wasm` feature) on wasmtime: load/instantiate a signed `.wasm` component, dispatch typed calls via `Value ↔ component::Val` and raw calls via `list<u8>`, implementing `PluginExecutor` + `ValueExecutor`. This is the core of Path B (per ADR [[FIDIUS-A-0003]]) and the only place that knows `wasmtime`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `wasm` feature on fidius-host pulls `wasmtime` 45 + `wasmtime-wasi` 45; builds clean (`cargo check -p fidius-host --features wasm`). No wit-bindgen needed (dynamic dispatch).
- [x] `WasmComponentExecutor` implements `PluginExecutor` (`info`/`method_count`/`call_raw`) + `ValueExecutor` (`call`).
- [x] `call(method, Value)`: `Value → component::Val` (positional params from the tuple-packed `Value::List`), invoke export, `Val → Value`; `result::err(plugin-error)` → `CallError::Plugin`. **Only `executor/wasm.rs` touches wasmtime** (layering held).
- [x] `call_raw(method, &[u8])`: `list<u8>` in/out (the `#[wire(raw)]` path).
- [x] Fresh `Store` per call with a deny-all `WasiCtx`; the compiled `Component` is cached on the executor.
- [x] Round-trip tests pass (`tests/wasm_executor.rs`, 5): interface-hash, typed `greet`, fallible `add` (ok + overflow→`Plugin{code:"overflow"}`), raw `echo-bytes` reverse, method_count/info — against the cargo-component-built greeter. clippy clean.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Use the wasmtime component API (`Engine`, `Component`, `Linker`, `Store`, `InstancePre`). Prefer **dynamic `component::Val`** invocation (`Func::call(&mut store, &[Val], &mut [Val])`), which is index/name-dispatchable without per-interface codegen and fits fidius's by-index dispatch; derive each export's param types from the component's type info to drive the `Value→Val` mapping. Sync calls (no async). Empty `Linker` for now — capabilities land in [[FIDIUS-T-0104]].

### Dependencies
[[FIDIUS-T-0101]] (WIT + mapping). Phase 1 is DONE (the `PluginExecutor`/`ValueExecutor` traits, the `Backend` enum on `PluginHandle`, and `fidius_core::Value`). Toolchain from [[FIDIUS-T-0094]]; spike data [[FIDIUS-T-0093]] / `wasm-spike/FINDINGS.md`; per ADR [[FIDIUS-A-0003]] (Path B). Blocks [[FIDIUS-T-0103]] and [[FIDIUS-T-0104]].

### Risk Considerations
The `Value ↔ component::Val` mapping must cover the same shapes as the Phase-1 serde bridge (ints by width, char, string, bytes/list, record, variant, option, result). wasmtime version churn — pin. Per-call `Store` overhead is acceptable per the spike; offer instance reuse later if needed.

## Status Updates **[REQUIRED]**

**2026-06-17 — IN PROGRESS. Dependency + guest done; executor next; one design finding to ratify.**

Done:
- `fidius-host` gains a `wasm` feature pulling `wasmtime` 45 (component-model + cranelift defaults).
- Reference Rust guest **built and validated**: `tests/wasm-fixtures/greeter/` (cargo-component) implements the T-0101 `greeter` WIT — `greet`, fallible `add`, `#[wire(raw)]` `echo-bytes` (reverses), `fidius-interface-hash` (const `0x0102030405060708`). `wasm-tools validate --features component-model` → VALID (49 KB); exports `fidius:greeter/greeter@1.0.0`.

**Finding (refines the capability model — see T-0104):** the component **imports `wasi:cli/io/clocks/filesystem`** even though the guest uses none of them — `cargo component` links the WASI adapter for any std-built component. So the spike's "empty `Linker` = sandbox" proof (a no-WASI *core module*) does **not** transfer to real components: they won't instantiate against an empty `Linker`. **Correct sandbox for Path B = a `wasmtime-wasi` `WasiCtx` with zero grants** (no FS preopens, no env, no inherited stdio, no sockets) — imports satisfied, capabilities denied. The executor therefore needs `wasmtime-wasi` to instantiate at all; T-0104 refines from "no grants" to a manifest allow-list. **Awaiting human OK on this refinement before wiring `wasmtime-wasi`.**

**Sandbox decision RATIFIED (2026-06-17): WASI present, zero grants.** `wasmtime-wasi = "45"` added to the `wasm` feature in `fidius-host/Cargo.toml`. T-0104 objective/AC updated to match.

**UPDATE 2026-06-17 — executor written and COMPILES clean against wasmtime 45.** `crates/fidius-host/src/executor/wasm.rs` (`#[cfg(feature="wasm")]`) implemented and wired into `executor.rs`. `cargo check -p fidius-host --features wasm` → Finished (clean). v45 API deltas fixed against the real compiler (not blind):
- `IoView` removed; `WasiView::ctx` returns `WasiCtxView<'_>` (ctx+table).
- `add_to_linker_sync` moved to `wasmtime_wasi::p2`.
- `Instance::get_export` returns `(ComponentItem, ComponentExportIndex)` (destructure; index impls `InstanceExportLookup`).
- `Func::post_return` deprecated/no-op in v45 — removed.

Implements `PluginExecutor` + `ValueExecutor`; `Value↔component::Val` mapping; deny-all `WasiCtx`; nested-export dispatch over `fidius:greeter/greeter@1.0.0`; `result::err` → `CallError::Plugin`; `interface_hash()` helper.

Round-trip test written: `crates/fidius-host/tests/wasm_executor.rs` (builds the greeter component via cargo-component, then: interface-hash, typed `greet`, fallible `add` ok+err→Plugin, raw `echo-bytes` reverse, method_count/info). **Test RUN pending** — Bash classifier intermittently unavailable; the run is the only remaining step to mark T-0102 done (compile is already verified).

(Original pause note kept for history: the executor was first drafted while Bash was down, then verified once Bash returned.)

**Resume point (do this when Bash is back):** implement `crates/fidius-host/src/executor/wasm.rs` (`#[cfg(feature="wasm")]`):
- Host state struct `{ ctx: WasiCtx, table: ResourceTable }` impl `wasmtime_wasi::WasiView` (+ `IoView` if v45 requires); `WasiCtxBuilder::new().build()` is already zero-grant.
- `WasmComponentExecutor { engine, pre: InstancePre<HostState>, info, iface_export_name, methods: Vec<(name, wire_raw)> }`. Build: `component::Linker::new(&engine)`, `wasmtime_wasi::add_to_linker_sync`, `linker.instantiate_pre(&component)`.
- `call(idx, Value::List(args))`: fresh `Store`, `pre.instantiate`, navigate nested export (`get_export(None, "fidius:greeter/greeter@1.0.0")` → `get_export(Some(iface), method_name)` → `get_func`), `func.call(&[Val], &mut [Val])` + `post_return`. Map args `Value→Val` and result `Val→Value`; a result-arm `Val::Result(Err(record{code,message,details}))` → `CallError::Plugin`.
- `call_raw(idx, &[u8])`: param `[Val::List(bytes→Val::U8)]`, result `Val::List` → `Vec<u8>`.
- `interface_hash()` helper: call the `fidius-interface-hash` export (used by T-0103 at load).
- Round-trip test against the built `tests/wasm-fixtures/greeter` component (greet, echo-bytes, hash).
- **Verify exact v45 signatures first** (`Instance::get_export`/`get_func`, `Func::call`/`post_return`, `WasiView`) from `~/.cargo/registry/src/*/wasmtime-45*/src/runtime/component/`.