---
id: ws-1-wasm-streaming-foundation-wit
level: task
title: "WS.1 — WASM streaming foundation: WIT resource contract + iterator-backed Stream<T> + descriptor streaming flag"
short_code: "FIDIUS-T-0129"
created_at: 2026-06-19T03:28:05.326341+00:00
updated_at: 2026-06-19T03:38:32.710113+00:00
parent: FIDIUS-I-0026
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0026
---

# WS.1 — WASM streaming foundation: WIT resource contract + iterator-backed Stream<T> + descriptor streaming flag

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0026]] · Phase 2 foundation · implements W-D1/W-D2/W-D3/W-D6. Root of the Phase-2 graph.

## Objective **[REQUIRED]**

Lay the backend-agnostic groundwork for WASM server-streaming, with **no macro codegen or host backend yet**: lock the WIT resource contract (as a spec/doc + the descriptor shape), make `fidius::Stream<T>` iterator-backed so a Rust WASM guest can actually produce items, and add a `streaming` flag to the WASM method descriptor so routing knows which exports stream.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] **WIT resource contract documented** (W-D1/W-D2): a streaming method `m(args) -> fidius::Stream<T>` maps to an exported `resource m-stream { next: func() -> result<option<T>, plugin-error>; }` and the function `m: func(args) -> own<m-stream>`. `some`=item, `none`=clean end, `err`=mid-stream error; resource-drop = guest dtor = cancel (D3). Captured in `docs/`/ABI spec + a module doc-comment (no codegen here).
- [ ] **`fidius::Stream<T>` becomes iterator-backed** (W-D3): keeps its current marker role for the trait/Python path but gains `Stream::from_iter(impl IntoIterator<Item = T>)` storing `Box<dyn Iterator<Item = T> + Send>` and an internal `next() -> Option<T>` the macro adapter will drive. Additive — existing `Stream::new()`/`Default` and the Phase-1 Python path are unchanged. Lives in `fidius-guest` (must stay `wasm32-wasip2`-buildable: no host deps).
- [ ] **`WasmMethodDesc` (guest) + `WasmMethod` (host) gain `streaming: bool`** (W-D6), defaulted `false`, plumbed but not yet consumed. Additive to the descriptor; existing non-streaming WASM plugins unaffected.
- [ ] Unit tests: `Stream::from_iter` yields its items then `None`; `fidius-guest` still builds for `wasm32-wasip2` (or a `cargo check --target` smoke if the target is installed).
- [ ] Existing WASM tests (`--features wasm`) and the Phase-1 streaming tests stay green; no behavioural change yet.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
- `Stream<T>` internal: `enum Repr { Marker, Iter(Box<dyn Iterator<Item=T> + Send>) }` or simply `Option<Box<dyn Iterator…>>` (None = the pure marker form used by the interface/Python path). Add `pub(crate) fn next_item(&mut self) -> Option<T>` for the macro adapter (WS.2) to call.
- Keep `Stream<T>` in `fidius-guest::stream_marker`; do **not** pull `futures`/host types in (wasm-buildable).
- Thread `streaming` into `WasmMethodDesc`/`WasmMethod` constructors; the macro (WS.2) sets it from `MethodIR.streaming`, the host (WS.3) reads it.

### Dependencies
- Builds on ST.1–ST.3 (Phase 1). Blocks WS.2 (macro), WS.3 (host).

### Risk Considerations
- `Box<dyn Iterator + Send>` must not break `wasm32-wasip2` builds — it won't (no_std-incompatible bits avoided; `Box`/`dyn` are fine on wasip2). Verify with a target check.
- Don't over-spec the WIT now: the *contract* is the deliverable; the *generator* that emits it is WS.2.

## Status Updates **[REQUIRED]**

### 2026-06-19 — WS.1 complete ✅
- **WIT resource contract documented** (W-D1/W-D2) as a module doc-comment on `fidius_guest::stream_marker::Stream` (the `resource <m>-stream { next: func() -> result<option<T>, plugin-error> }` + `m -> own<…>` shape; some/none/err semantics; resource-drop=cancel). A fuller ABI-spec doc entry can ride along with WS.2's generator.
- **`fidius::Stream<T>` is now iterator-backed** (W-D3): `Stream::from_iter(impl IntoIterator<Item=T>)` (iter `Send + 'static`) stored as `Option<Box<dyn Iterator<Item=T> + Send>>`, plus `pub fn next_item(&mut self) -> Option<T>` for the WS.2 adapter. `new()`/`Default` keep the empty marker form (interface/Python path). `#[allow(clippy::should_implement_trait)]` on `from_iter` (inherent ctor needs `Send+'static`, which `FromIterator` can't express). **3 unit tests** (from_iter→items→None, range, marker empty).
- **`streaming: bool` added** to `WasmMethodDesc` (guest) + `WasmMethod` (host), set by the macro from `MethodIR.streaming` (interface.rs) and read by the host loader (host.rs). Updated the 8 hand-authored literals in `wasm_executor.rs` (+`WasmMethod`) and `benches/backends.rs`.
- **Verified**: `fidius-guest stream_marker::` 3/3; **`cargo check -p fidius-guest --target wasm32-wasip2` green** (Stream<T> stays wasm-buildable); `cargo test -p fidius-host --features wasm --test wasm_executor` **21/21** (no regression); full `fidius-macro` suite green; `angreal build` green. wasm32-wasip2 target confirmed installed (good for WS.2/WS.4).
- No behavioural change yet — purely additive foundation. Next: WS.2 (macro resource adapter) — flagged as the crux; spike the wit-bindgen resource shape first.