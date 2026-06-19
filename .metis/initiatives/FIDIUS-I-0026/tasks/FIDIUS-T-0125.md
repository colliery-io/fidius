---
id: st-2-macro-surface-fidius-stream-t
level: task
title: "ST.2 — Macro surface: fidius::Stream<T> return marker → streaming shim codegen + interface-hash treatment"
short_code: "FIDIUS-T-0125"
created_at: 2026-06-18T18:14:23.770723+00:00
updated_at: 2026-06-19T02:50:44.768006+00:00
parent: FIDIUS-I-0026
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0026
---

# ST.2 — Macro surface: fidius::Stream<T> return marker → streaming shim codegen + interface-hash treatment

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0026]] · Phase 1 · implements D4 (+D5 single-item). Depends on [[FIDIUS-T-0124]].

## Objective **[REQUIRED]**

Teach the interface macro the `fidius::Stream<T>` return-position marker so a trait method declares server-streaming, generating the streaming shim instead of the unary shim, and fold the stream shape into the interface hash so a streaming method can never be confused with a unary one of the same name/types.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] `#[fidius::plugin_interface]` accepts a method returning `fidius::Stream<T>` (explicit, macro-legible marker — not inferred from `impl Trait`); `T` follows the same wire/`WitType` rules as a unary return.
- [ ] Macro emits the streaming shim (init/next/drop) for such methods and the existing unary shim for all others; mixed streaming + unary methods on one trait compile.
- [ ] The canonical signature string gains a `stream` marker so `interface_hash` differs between `fn f(..) -> Stream<T>` and `fn f(..) -> T` (extends REQ-005); add hash vectors proving the distinction.
- [ ] Host-side typed `Client` proxy generates a method returning `ChunkStream` (or a typed wrapper decoding `Value → T`) for streaming methods.
- [ ] `trybuild`/compile-fail coverage: `Stream<T>` in argument position is rejected with a clear "server-streaming only in v1" error (client-streaming/bidi deferred).
- [ ] Macro-expansion snapshot tests for a streaming method; existing macro snapshots unchanged.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
- Extend the IR (`fidius-macro/src/ir.rs`) with a per-method `streaming: bool` derived from detecting the `Stream<T>` return path; branch shim codegen on it.
- Thread the marker into `hash::signature_string` so the streaming bit is part of the hashed signature; keep ordering/format stable with existing vectors.
- For WIT/WASM emission (consumed later by ST in Phase 2), record the stream return in `WasmInterfaceDescriptor` now even though no WASM backend ships in Phase 1 — cheaper than retrofitting.

### Dependencies
- Depends on [[FIDIUS-T-0124]] for `ChunkStream`/`Frame` types referenced by generated code. Blocks ST.3 end-to-end typed calls and ST.5 E2E.

### Risk Considerations
- `Stream<T>` name resolution must respect the configurable `crate_path` (FIDIUS-T-0062) — don't hardcode `fidius::`.
- Keep the hash change additive-but-distinct: a previously-unary method must keep its old hash; only methods actually declared streaming change.

## Status Updates **[REQUIRED]**

### 2026-06-18 — paused pending re-slice (coupling found)
Read the macro internals (`ir.rs`, `interface.rs`, `hash.rs`). Two findings change this task's boundary:
1. **The "Client returns `ChunkStream`" criterion couples to ST.3.** The generated client method would call `PluginHandle::call_streaming(...)`, but that routing method is added by ST.3 (backend plumbing). The client codegen cannot compile or be tested until the handle method exists, so it belongs with ST.3, not here.
2. **The cdylib `init/next/drop` shim is Phase-3 code Python never runs.** Phase-1's wedge is Python (`Pyo3Executor` over `.py` files), which doesn't use `#[plugin_impl]`'s cdylib shim at all. Emitting a full streaming FFI ABI now is pulling Phase-3 work forward.

**Proposed re-slice (pending human approval):**
- ST.2 (this task) → **interface-side only**: parse `fidius::Stream<T>` return marker in `ir.rs` (+ `streaming` flag, item type), reject arg-position `Stream<T>` (compile-fail), add the `!stream` marker to `hash::signature_string` so streaming/unary hash distinctly, and record streaming in the Python/WASM descriptors. Unit + trybuild tested, no PyO3, no host coupling.
- Move **Client `ChunkStream` codegen into ST.3** (alongside `PluginHandle::call_streaming`).
- `#[plugin_impl]` emits a clear "cdylib streaming impl is Phase 3" error for streaming methods (native plugins fail loudly; Python unaffected); the real cdylib shim is Phase 3.

Doing ST.4 (harness — isolated, verifiable) next; this task awaits re-slice confirmation.

### 2026-06-19 — ST.2 (interface-side, re-sliced) complete ✅
Re-slice approved by Dylan (interface-only; Client `ChunkStream` codegen → ST.3; cdylib shim → Phase 3). Delivered:
- **`fidius::Stream<T>` marker** — new `fidius_guest::stream_marker::Stream<T>` (PhantomData marker), re-exported via `fidius_core::Stream` and `fidius::Stream`. Makes streaming trait decls type-check; the macro detects it.
- **Hash treatment** — `hash::signature_string` gained a `streaming` param appending `!stream` (after any `!raw`); a streaming method's canonical return is the per-item type `T` → `read:String->u64!stream`. Non-streaming signatures are byte-identical to before, so existing interface hashes are unchanged (REQ-005 additive-but-distinct). Updated all 3 `fidius-cli` call sites.
- **IR** — `MethodIR.streaming` + `stream_item_type`; `stream_item_type()` detects `Stream<T>` by final path segment; arg-position `Stream<T>` rejected with a clear "server-streaming only" error.
- **`#[plugin_impl]`** — rejects a `-> Stream<T>` method (`ret_is_stream` guard) with a "native streaming is a later phase; use Python" message.
- **Tests** — new IR unit tests (detection, item type, streaming-vs-unary hash divergence, bare-marker, arg-position rejection) + 2 trybuild compile-fail fixtures (`stream_in_arg_position`, `native_streaming_impl`).
- **Verified**: full `fidius-macro` suite green (incl. 7 compile_fail), guest/core/cli green, `angreal build` green. pyo3/fidius-python compiled → embedded-Python toolchain present in this env.
- **Deferred to ST.3**: host-side Client method returning `ChunkStream` (needs `PluginHandle::call_streaming`). **Deferred to Phase 3**: cdylib `init/next/drop` FFI shim. **Deferred (later)**: `python-stub` CLI streaming support + a `streaming` flag on Python/WASM descriptors (hash already encodes it, so not needed for Phase 1).