---
id: ws-2-macro-wasm-streaming-resource
level: task
title: "WS.2 — Macro: wasm streaming resource adapter codegen + relax #[plugin_impl] guard for wasm + WIT generator"
short_code: "FIDIUS-T-0130"
created_at: 2026-06-19T03:28:07.212006+00:00
updated_at: 2026-06-19T04:27:52.638123+00:00
parent: FIDIUS-I-0026
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0026
---

# WS.2 — Macro: wasm streaming resource adapter codegen + relax #[plugin_impl] guard for wasm + WIT generator

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0026]] · Phase 2 · implements W-D2/W-D4. Depends on [[FIDIUS-T-0129]].

## Objective **[REQUIRED]**

Teach `#[plugin_impl]` and the WIT generator to emit a server-streaming **resource** for a `-> fidius::Stream<T>` method when targeting wasm: a `resource <m>-stream { next: func() -> result<option<T>, plugin-error>; }`, the function `m -> own<…>`, and a `Guest` impl whose `next()` drives the guest's iterator-backed `Stream<T>`. Relax the ST.2 native-streaming guard for wasm targets only.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] **Guard relaxed for wasm** (W-D4): `#[plugin_impl]` no longer rejects a `-> Stream<T>` method under `#[cfg(target_family = "wasm")]`; the cdylib (non-wasm) path still rejects with the existing "Phase 3" error. (The `native_streaming_impl` trybuild fixture, which builds for the host target, stays red as today.)
- [ ] **WIT generation** emits, for each streaming method, the resource type + `next` signature (W-D2) and the owning return, alongside the existing primitives/records path. Works in both the inline-WIT (primitives-only) and `build.rs` (`fidius-build`/`fidius-wit`) paths, or is explicitly scoped to one with the other deferred + noted.
- [ ] **Guest adapter**: the generated `Guest` resource impl holds the plugin method's returned `Stream<T>` and `next()` pulls one item (`Some(v)` → `Ok(some(v))`, `None` → `Ok(none)`), mapping a plugin error to the `err` arm.
- [ ] **`WasmMethodDesc.streaming`** is set from `MethodIR.streaming` in the interface codegen (so WS.3's host loader sees it).
- [ ] A Rust streaming guest fixture **compiles to a component** for `wasm32-wasip2` (the deep compile/run validation is WS.4; here, at minimum, macro-expansion + a targeted build).
- [ ] Existing macro suite (incl. the 2 streaming compile-fail fixtures) + non-streaming WASM E2E stay green.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
- Extend `generate_wasm_adapter` (`impl_macro.rs`): branch streaming methods to a resource-export path. Reuse `wit::*` helpers (`return_to_wit`, `rust_type_to_wit`) for the item type `T`.
- The guard relaxation: in the `ret_is_stream` check, only `return Err(...)` when `!cfg!(target_family = "wasm")` — i.e. emit the error as a `#[cfg(not(target_family="wasm"))] compile_error!` (mirroring the existing `wasm_unsupported` pattern) so cdylib builds fail and wasm builds proceed.
- WIT resource emission: confirm `wit-bindgen`'s `generate!` supports an exported resource in the inline form; if only the `path:"wit"` form supports resources cleanly, scope the macro to that and note the inline limitation.

### Dependencies
- Depends on [[FIDIUS-T-0129]] (`Stream<T>::next_item`, descriptor flag, WIT contract). Blocks [[FIDIUS-T-0132]]/[[FIDIUS-T-0133]] (fixtures/E2E).

### Risk Considerations
- **wit-bindgen resource codegen is the crux** — guest-exported resources with methods are more intricate than plain funcs. Spike the simplest hand-written WIT+guest first to confirm the wasmtime/wit-bindgen shape, then drive codegen to match.
- Keep the interface hash stable: the `!stream` marker (ST.2) already differentiates; the WIT shape change must not alter the *hash* (hash is computed from the Rust signature, not the WIT).

## Status Updates **[REQUIRED]**

### 2026-06-19 — Spike succeeded + Stage A (WIT layer) done
**Spike (the crux) — PROVEN.** Hand-authored `tests/wasm-fixtures/ticker` exports a streaming `resource`; **`cargo component build` succeeds** — wit-bindgen 0.44 handles a guest-exported `resource tick-stream { next: func() -> result<option<u64>, plugin-error> }`. Generated bindings shape confirmed: `Guest { type TickStream: GuestTickStream; fn tick(count) -> TickStream }`, `GuestTickStream { fn next(&self) -> Result<Option<u64>, PluginError> }`, construct via `TickStream::new(impl)`. The host side (WS.3) drives it end-to-end (3 E2E tests green).

**Stage A — `fidius-wit` resource rendering (DONE, 15 tests green):**
- `WitMethod` gained `stream_item: Option<String>`; `pub fn stream_item_type(ty) -> Option<&Type>` detects `Stream<T>`.
- `render_wit_full` emits `resource <m>-stream { next: func() -> result<option<T>, plugin-error>; }` (before funcs) + `<m>: func(params) -> <m>-stream;`.
- Source generator (`generate.rs`) detects `Stream<T>` returns and sets `stream_item` (instead of erroring on the unknown `Stream` type).
- Unit tests: streaming render + `stream_item_type` detection. Macro builds (non-streaming `WitMethod` sites updated to `stream_item: None`).

**Stage B — macro guest codegen (REMAINING, the heavy lift). Worked-out plan:**
1. `MethodInfo` gains `stream_item: Option<&Type>` (via `wit::stream_item_type`).
2. **Guard restructure**: the macro can't see the *target* (it runs on the host), so a `cfg!(target_family)` check in the macro is wrong. Instead: if **any** method is streaming, *skip the cdylib codegen* (shims/vtable/descriptor/registration) and emit `#[cfg(not(target_family="wasm"))] compile_error!("native streaming → build as a wasm component")`; always emit the wasm adapter. Non-wasm builds fail loudly (keeps `native_streaming_impl` trybuild red); wasm builds compile. (Replaces the current hard `return Err` guard.)
3. **`generate_wasm_adapter`**: classify the streaming method's *item* type (not the `Stream` wrapper) in collect/validation (`eff_ret = m.stream_item.or(m.ret_type)`); build `WitMethod.stream_item`; in the `!has_user` (primitives) branch emit, per streaming method: a module-level `struct __Fidius<Pascal> { stream: RefCell<Stream<T>> } + impl Guest<Pascal>Stream { fn next(&self){ Ok(self.stream.borrow_mut().next_item()) } }`, and inside `impl Guest` a `type <Pascal> = __Fidius<Pascal>;` + `fn m(..) -> <Pascal> { <Pascal>::new(__Fidius<Pascal>{ stream: RefCell::new(super::INSTANCE.m(..)) }) }`. **Scope: primitives-only item types** (reject streaming + `#[derive(WitType)]` records for now → `wasm_unsupported`); the build.rs/user-type path is a follow-up.
4. **Stage C (WS.4)**: a macro-based streaming guest fixture built to a component + reuse the WS.3 E2E harness (`wasm_streaming_e2e.rs`) to prove the macro-generated component loads & streams identically to the hand-built ticker.

**Checkpointed here**: the host + contract + WIT layer are proven/done; Stage B is a large, slow-to-verify proc-macro change (needs a macro→component build loop). Resuming it is well-scoped above.

### 2026-06-19 — Stage B + C done ✅ — WS.2 COMPLETE (end-to-end)
Followed the plan exactly; it worked.
- **Stage B (macro guest codegen):** `MethodInfo.stream_item`; removed the hard `return Err` guard → **if any method streams, the cdylib codegen is skipped** and a `#[cfg(not(target_family="wasm"))] compile_error!` is emitted (streaming plugins are wasm-only), while the wasm adapter always emits. `generate_wasm_adapter` now classifies the stream *item* type (not the `Stream` wrapper); the `!has_user` branch emits, per streaming method: a module-level `struct __Fidius<Pascal> { stream: RefCell<Stream<T>> }` + `impl Guest<Pascal> for it { fn next(&self) -> Ok(self.stream.borrow_mut().next_item()) }`, and inside `impl Guest` a `type <Pascal> = __Fidius<Pascal>;` + `fn m(..) -> <Pascal> { <Pascal>::new(...) }`. `kebab_to_pascal` derives the wit-bindgen resource names (`tick`→`TickStream`/`GuestTickStream`). Streaming + `#[derive(WitType)]` user-type items → `wasm_unsupported` (deferred; primitive/String items supported).
- **trybuild**: `native_streaming_impl.stderr` regenerated (new wasm-only message); cdylib streaming still rejected on the host target.
- **Stage C (the proof):** new `tests/wasm-fixtures/macro-ticker` — a fidius plugin written purely with the macros (`fn tick(&self, count: u32) -> fidius_guest::Stream<u64>` + `Stream::from_iter(0..count)`). **`cargo build --target wasm32-wasip2` produces a valid component.** Host E2E (`crates/fidius-host/tests/macro_wasm_streaming.rs`, **3 tests green**): the macro descriptor marks `tick` streaming + `interface_export = fidius:ticker/ticker@0.1.0`; `load_wasm` succeeds (macro-derived hash == component's `fidius-interface-hash`); `call_streaming(tick, 5)` → `[0..5]`; 10M-item bounded/cancellable.
- **Verified**: full `fidius-macro` suite green (incl. 7 compile_fail); `fidius-wit` 15/15; `cargo test -p fidius-host --features wasm,streaming` all green; `angreal test`/`angreal build` (default) green. **Write a trait → get a sandboxed streaming WASM plugin.**
- **Deferred (noted, non-blocking)**: streaming with `#[derive(WitType)]` record item types (build.rs/user-type path).