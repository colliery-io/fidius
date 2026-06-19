---
id: st-3-pyo3-streaming-backend
level: task
title: "ST.3 — Pyo3 streaming backend: generator bridge → tokio mpsc → ChunkStream + drop-cancel"
short_code: "FIDIUS-T-0126"
created_at: 2026-06-18T18:14:25.188694+00:00
updated_at: 2026-06-19T03:12:37.282268+00:00
parent: FIDIUS-I-0026
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0026
---

# ST.3 — Pyo3 streaming backend: generator bridge → tokio mpsc → ChunkStream + drop-cancel

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0026]] · Phase 1 (the wedge) · first real backend. Depends on [[FIDIUS-T-0124]], [[FIDIUS-T-0125]].

## Objective **[REQUIRED]**

Make `Pyo3Executor` implement `StreamExecutor`: a Python plugin method that `yield`s items (a generator / async generator) is bridged onto a host-side `ChunkStream`. This is the proof that the primitive works end-to-end on the first backend and the adoption path for the existing (Python) connector ecosystem.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] `Pyo3Executor` implements `StreamExecutor::call_streaming`, returning a `ChunkStream` fed from the Python plugin's generator.
- [ ] Bridge design: the generator is pumped on a blocking thread (GIL held only while advancing) that emits `Frame`s into a `tokio::mpsc`; the receiver side becomes the `ChunkStream`. Backpressure is structural — the pump blocks on a bounded channel when the host stops pulling (REQ-003).
- [ ] Drop-cancel (D3/REQ-002): dropping the `ChunkStream` closes the channel; the pump observes it and calls `generator.close()` (raising `GeneratorExit`) so the Python side runs its `finally`/cleanup; verified by a test asserting cleanup ran.
- [ ] A Python plugin error mid-stream surfaces as an `ERROR` frame → `Err(CallError)` and terminates the stream (reusing the existing Python error mapping).
- [ ] Per-item `Value ↔ PyObject` conversion reuses the existing `value_bridge`; no new conversion path.
- [ ] Capability/runtime parity: streaming Python plugins load and route through `PluginHost` exactly like unary ones; manifest unchanged.
- [ ] Works under the in-process model first; note (don't necessarily solve) interaction with the `pyo3_process`/`pyo3_thread` subprocess variants from the poc.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
- Detect a streaming method via the descriptor/interface metadata (set by ST.2) and dispatch to the generator-pump path instead of the single-call path.
- Hold the GIL only for `next(gen)`; release while the host awaits, so a slow consumer never pins the GIL.
- Size the mpsc small (e.g. 1–8) so memory stays bounded (NFR-003) and backpressure is real rather than buffering the whole stream.

### Dependencies
- Depends on [[FIDIUS-T-0124]] (`ChunkStream`/`Frame`/`StreamExecutor`) and [[FIDIUS-T-0125]] (macro marks the method streaming). Consumed by [[FIDIUS-T-0128]] (E2E) and [[FIDIUS-T-0127]] (harness drives it).

### Risk Considerations
- GIL + blocking-thread + tokio interplay is the crux — get cancellation-on-drop right so a dropped stream can't leak a parked Python thread.
- `GeneratorExit` must actually reach the generator even if the host drops mid-item; test the drop-during-yield race explicitly.

## Status Updates **[REQUIRED]**

### 2026-06-19 — ST.3 mechanism complete ✅ (E2E load-test → ST.5)
Delivered the full server-streaming backend path:
- **`fidius_python::stream`** — `PythonStream` (owns the generator's iterator, `Send`) + `PyStreamStep {Item(json)|End|Error(PluginError)}`. `next()` advances under the GIL for one step only; `cancel()` calls `gen.close()` → `GeneratorExit` → `finally`. `PythonPluginHandle::call_streaming_start` calls the method, `try_iter()`s the result (a generator is its own iterator, so `close()` reaches it). **3 embedded-Python unit tests green**: items→End, exception→Error (terminal), **cancel-runs-finally** (the drop-cancel crux).
- **`Pyo3Executor: StreamExecutor`** (host, gated `streaming`+`python`) — starts the generator, spawns a **dedicated pump thread** that `blocking_send`s native `Value` items into a bounded `tokio::mpsc` (cap 4 = the backpressure/memory window), and on `send` error (consumer dropped the `ChunkStream`) calls `stream.cancel()`. Items are native `Value`s (no framing — bincode can't reconstruct a `Value`; JSON→Value via `serde_json::from_value`). GIL held only per `next()`, released during `blocking_send` (backpressure wait). `ChunkStream` built from `futures::stream::unfold` over the receiver.
- **`PluginHandle::call_streaming<I>`** (gated `streaming`) — async; routes Python→bridge, cdylib/wasm→clear "not supported yet (Phase 3/2)" backend error.
- **Macro Client codegen (moved here from ST.2)** — `generate_client` now emits, for a streaming method, `pub async fn m(..) -> Result<ChunkStream, CallError>` calling `self.handle.call_streaming(index, &(args,)).await`. Prevents the previously-broken unary codegen for `-> Stream<T>`.
- **Facade plumbing** — `fidius` gains a `streaming` feature (`["host", "fidius-host/streaming"]`) re-exporting `ChunkStream`/`StreamExecutor`.
- **Verified**: `fidius-python stream::` 3/3; `cargo build -p fidius-host --features streaming,python` green; full `fidius-macro` suite green; `angreal build` green.
- **Design note**: streaming dispatch is chosen by the *caller* (Client/`call_streaming`), not a descriptor flag — so no Python SDK/descriptor change was needed; a streaming method is just a normal registered function that returns a generator.
- **Deferred to ST.5 (shared E2E)**: the load-a-real-Python-package test (`#[plugin_impl]` rejects native streaming, so the fixture must be a Python-only `#[plugin_interface]` with a matching `__interface_hash__` that includes `!stream`). The full-stack pump + bounded-memory + drop-cancel-through-`PluginHost` + `pluggable-poc` slice land there and double as ST.3's end-to-end proof. The risky GIL/cancel/finally logic is already unit-tested here.

### 2026-06-19 — E2E confirmed (via ST.5) ✅ — closing
ST.5's `python_streaming_e2e` (4 tests, green) drives this backend through the full `PluginHost` stack: a real Python generator plugin loads (cross-language `!stream` hash matched), `call_streaming` yields all items, a **10M-item generator pulled-3-then-dropped returns in 0.02s** (definitive bounded-memory + backpressure + cancel proof through the whole stack), and the `fidius-test` harness pumps it into a sink. Full `fidius-host --features python,streaming` suite green; `angreal test` (default) green — no regressions. **Client-method codegen descoped** to skip (route via `PluginHandle::call_streaming`) — documented in interface.rs — to avoid a fragile cross-crate `streaming`-feature coupling; typed Client sugar can return behind a clean feature story later. All ST.3 ACs met (Frame→native-Value substitution per ST.1 finding; descriptor-flag→caller-chosen-dispatch per design note).