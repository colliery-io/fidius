---
id: ws-3-host-wasmcomponentexecutor
level: task
title: "WS.3 — Host: WasmComponentExecutor StreamExecutor (resource next() pump) + routing + capability/signing parity"
short_code: "FIDIUS-T-0131"
created_at: 2026-06-19T03:28:08.622207+00:00
updated_at: 2026-06-19T03:48:51.598373+00:00
parent: FIDIUS-I-0026
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0026
---

# WS.3 — Host: WasmComponentExecutor StreamExecutor (resource next() pump) + routing + capability/signing parity

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0026]] · Phase 2 · implements W-D5/W-D6. Depends on [[FIDIUS-T-0129]], [[FIDIUS-T-0130]].

## Objective **[REQUIRED]**

Make `WasmComponentExecutor` implement `StreamExecutor`: call a streaming export to get the resource handle, then pump its `next()` on a dedicated thread into a bounded channel → `ChunkStream`, with dropping the stream releasing the resource handle (guest dtor = cancel). Route streaming WASM calls through `PluginHandle::call_streaming` and preserve the existing sandbox/capability/signing guarantees.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] `WasmComponentExecutor` implements `StreamExecutor::call_streaming` (gated `streaming`+`wasm`): resolves the streaming export, calls it to obtain the resource handle, holds the `Store`+instance+resource for the stream's lifetime.
- [ ] **Pump bridge** (W-D5, mirrors ST.3): a dedicated thread owns the `Store`+resource and `blocking_send`s items into a bounded `tokio::mpsc`; each pull calls `next()` (sync wasmtime), maps `ok(some(v))`→`Value`, `ok(none)`→end (drop tx), `err`→`Err(CallError::Plugin)`. `ChunkStream` via `unfold` over the receiver.
- [ ] **Drop-cancel** (D3/REQ-002): dropping the `ChunkStream` drops the receiver → pump exits → the resource handle is dropped under the `Store`, invoking the guest's resource destructor. Verified in WS.4 (dtor side-effect).
- [ ] **Backpressure/bounded** (REQ-003/NFR-003): bounded channel parks the pump; a huge guest stream pulled-few-then-dropped returns promptly (asserted in WS.4).
- [ ] **Routing**: `PluginHandle::call_streaming`'s `Backend::Wasm` arm now dispatches (was a "Phase 2" error); reads `WasmMethod.streaming` to pick the resource path; a non-streaming index returns a clear error.
- [ ] **Parity** (NFR-002): the stream runs under the same deny-all `WasiCtx` + capability allow-list + signature verification as unary WASM calls; `validate_capabilities` unchanged.
- [ ] Existing `--features wasm` suite stays green.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
- wasmtime resource handling: the streaming export returns a `ResourceAny`; call the resource's `next` method via the component instance, drop the `ResourceAny` (or drop the whole `Store`) to trigger the guest dtor. Confirm the wasmtime 45 API for calling a method on a returned resource and for resource drop semantics.
- The pump thread owns `Store<HostState>` (Send) + the resource handle; mirror the ST.3 structure (`tokio::sync::mpsc`, `blocking_send`, `futures::stream::unfold`). Reuse `val_to_value`/`plugin_error_from_val`.
- Keep `instance_pre` caching; per-stream still does one `instantiate()` (one Store per stream, not per item).

### Dependencies
- Depends on [[FIDIUS-T-0129]] (descriptor flag) + [[FIDIUS-T-0130]] (the component actually exports the resource). Consumed by [[FIDIUS-T-0132]] (Rust E2E) and [[FIDIUS-T-0133]] (polyglot).

### Risk Considerations
- **Resource lifetime across the pump thread** is the crux: the `Store` and `ResourceAny` must move to the thread and stay valid for the stream; getting drop order right (resource before/with Store) is what makes the dtor fire exactly once.
- wasmtime sync calls block the pump thread (fine — dedicated thread, like Python's GIL thread). Do NOT call them on a tokio worker.

## Status Updates **[REQUIRED]**

### 2026-06-19 — WS.3 complete ✅ (built via the WS.2 spike)
The WS.2 spike (hand-authored `tests/wasm-fixtures/ticker` component exporting a streaming `resource`) gave a known-good target, so the host backend went in cleanly and is **end-to-end green**:
- **`WasmComponentExecutor: StreamExecutor`** (gated `wasm`+`streaming`): calls the streaming export → `Val::Resource(ResourceAny)`; resolves `[method]<m>-stream.next`; a **dedicated pump thread** owns the `Store`+resource and `blocking_send`s items into a bounded `tokio::mpsc` (cap 4); maps `ok(some(v))`→`Value`, `ok(none)`→end, `err`→`Err(Plugin)`. On consumer drop → `resource.resource_drop(store)` (guest dtor = D3 cancel) then drop `Store`. `ChunkStream` via `futures::stream::unfold`.
- **wasmtime 45 API confirmed** (the spike's whole point): owned-resource return is `Val::Resource(ResourceAny)` (`ResourceAny` is `Copy`, reusable across `next()` calls); the method export is `[method]<resource>.next`; `ResourceAny::resource_drop(&mut store)` triggers the guest dtor. Compiled first try.
- **Routing**: `PluginHandle::call_streaming` `Backend::Wasm` arm now dispatches (was the "Phase 2" stub); reads `WasmMethod.streaming`.
- **Parity**: streams run through the same `instantiate()` (deny-all `WasiCtx` + allow-list); one `Store` per *stream* (not per item) — `instance_pre` caching preserved.
- **Verified** (`crates/fidius-host/tests/wasm_streaming_e2e.rs`, **3 tests green**): `tick(5)`→`[0..5]`; **10M-item generator pulled-3-then-dropped returns in 0.38s** (bounded + backpressure + cancel through the wasmtime resource stack); empty stream. Full `--features wasm,streaming` suite green; default build green; no regressions.
- Naming convention locked: resource for method `m` is `m-stream`, poll export `[method]m-stream.next` — **WS.2's macro codegen must follow this.**