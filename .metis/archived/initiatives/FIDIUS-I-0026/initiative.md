---
id: streaming-execution-primitive-pull
level: initiative
title: "Streaming execution primitive — pull-based backpressured Value streams across all backends"
short_code: "FIDIUS-I-0026"
created_at: 2026-06-18T17:16:09.044790+00:00
updated_at: 2026-06-19T21:29:34.922155+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
initiative_id: streaming-execution-primitive-pull
---

# Streaming execution primitive — pull-based backpressured Value streams across all backends Initiative

> Decision basis: **[[FIDIUS-A-0004]]** — *Streaming as Mechanism, Not Protocol*. This initiative builds only the mechanism that ADR scopes into core. Anything that ADR pushes above the line (connector protocol, checkpointing semantics, orchestration) is explicitly out of scope here.

## Context **[REQUIRED]**

fidius methods are unary today: `fn apply(&self, In) -> Out`, fully buffered, crossing the executor seam as bincode (cdylib) or self-describing `Value` (Python/WASM). Adopters — led by an Airbyte-like ingestion / reverse-ETL product — need long-lived calls that emit or consume an *unbounded sequence* of items at the plugin's own cadence, with bounded memory and clean cancellation.

FIDIUS-A-0004 decided the boundary: fidius ships **the typed pipe, not the connector protocol**. This initiative delivers that pipe — a streaming transport primitive uniform across all three backends — plus the trait-macro surface to declare it, plus an unsupported test harness for composition. It does *not* build `Message`/`State` envelopes, checkpointing, catalogs, or orchestration; those belong to adopters.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- A **pull-based, backpressured, framed streaming primitive** carrying `Value` (and raw bytes for `#[wire(raw)]`), with cancellation via handle-drop, exposed through the executor seam.
- A **`fidius::Stream<T>` marker** the interface macro understands, generating the streaming ABI shim and covering it in the interface hash, for **server-streaming** first: `fn read(&self, args) -> fidius::Stream<T>`.
- **Uniform host-facing abstraction** (e.g. `StreamExecutor` / a `ChunkStream` pull handle) with **three native backend implementations**: Python (generators) and WASM (resource + `next()`) on the `Value` rail first; cdylib (FFI pull-iterator handle) after.
- An **unsupported composition harness** in `fidius-test`: `stream_of(Vec) -> Stream`, `collect(Stream) -> Vec`, `pump(out, into)` — correct-but-not-semver-committed.
- A worked **end-to-end streaming example** in `pluggable-poc` (source emits → transform → sink collects) proving cross-backend composition.

**Non-Goals:**
- No connector protocol: no `Message`/`Record`/`State` envelope, no checkpoint/resume, no cursor/incremental logic, no catalog/`discover()` schema. (Adopter-owned; see FIDIUS-A-0004.)
- No orchestration: no scheduler, retry, parallelism, observability, or pipeline DSL. No supported `connect()`/`orchestrate()` API.
- No brokered network I/O — tracked separately (see Dependencies/Risks). A deny-all WASM sandbox can't open a socket; that capability work is its own initiative and is *not* delivered here.
- No bidirectional or client-streaming in the first cut (kept in the design; deferred).
- No change to existing unary call paths or the cdylib concrete-bincode wire.

## Requirements

### System Requirements
- **Functional Requirements**
  - REQ-001: Host can initiate a streaming method call and receive a pull handle that yields `Result<Value, CallError>` items until an end-of-stream marker or an error frame.
  - REQ-002: Dropping the pull handle signals cancellation to the plugin, which observes it and releases resources (close cursor/connection) via its native drop/finalizer path.
  - REQ-003: Backpressure is structural — the plugin makes no progress beyond a bounded buffer unless the host pulls.
  - REQ-004: The transport frames each item independently (length-delimited bincode/`Value`); a distinct end marker and a distinct error frame are representable.
  - REQ-005: `fidius::Stream<T>` in a trait method participates in the interface hash so producer/consumer mismatch is rejected at load, exactly as unary signatures are today.
  - REQ-006: The streaming surface is additive — existing unary `PluginExecutor::call_raw` / `ValueExecutor::call` and the cdylib bincode path are unchanged.
- **Non-Functional Requirements**
  - NFR-001: Per-item overhead stays competitive with the existing per-call benchmarks (reuse cached `InstancePre` / typed raw-bytes path for WASM; no full re-instantiation per item).
  - NFR-002: Streaming calls run under the existing sandbox + capability + signing guarantees with no weakening.
  - NFR-003: Bounded memory regardless of stream length (no full-buffering of the sequence anywhere in the host transport).

## Use Cases

### Use Case 1: Server-streaming source (the motivating adopter)
- **Actor**: An ingestion-product author writing a source connector (Python or WASM) against their own `Source` trait, which fidius sees only as `fn read(&self, args) -> fidius::Stream<Value>`.
- **Scenario**: Host calls `read`, receives a pull handle, pulls items as its (adopter-owned) destination accepts them; on cancellation it drops the handle and the connector closes its DB cursor.
- **Expected Outcome**: Unbounded record delivery at the plugin's cadence, bounded host memory, clean cancel — with checkpoint/envelope semantics layered *above* fidius by the adopter.

### Use Case 2: Incremental results beyond ingestion
- **Actor**: A plugin author returning a long or latency-sensitive result (LLM token stream, log tail, progress, search hits).
- **Scenario**: Method declares `-> fidius::Stream<Token>`; host renders items as they arrive.
- **Expected Outcome**: First-item-fast, incremental delivery — same primitive, no connector machinery, validating the generality the ADR banked on.

### Use Case 3: Testing composition
- **Actor**: Any plugin author / fidius contributor.
- **Scenario**: `let out = collect(transform.process(stream_of(rows)));` to unit-test one streaming plugin in isolation; `pump(source.read(cfg), &sink)` to integration-test real composition.
- **Expected Outcome**: Composition is trivial to test without fidius shipping a supported orchestrator.

## Architecture

### Overview
One host-facing pull abstraction, three native backend implementations, sitting alongside (not replacing) the existing executor traits:

```
                 host pulls →  ChunkStream (Result<Value> items, drop = cancel)
                                   ▲
              ┌────────────────────┼────────────────────┐
        Python (generator      WASM (resource +      cdylib (FFI iterator
        / __next__ bridge)     next(); stream<T>     handle + next() export;
                               when stable)          callback alt rejected)
```

- New seam: `trait StreamExecutor: PluginExecutor { async fn call_streaming(&self, method, args: Value) -> Result<ChunkStream, CallError>; }` (async-trait / RPITIT), implemented by the `Value`-rail backends first.
- `ChunkStream`: a `futures::Stream<Item = Result<Value, CallError>>` (concrete `Send` newtype over `Pin<Box<dyn Stream + Send>>`); host pulls via `.next().await`; drop = cancel.
- Wire: reuse the existing bincode/`Value` framing per item; add end + error frame markers. No new serialization format.

### Sequence (server-streaming)
1. Host → backend: `call_streaming(method, args)`.
2. Backend instantiates/locates the plugin's stream producer, returns a handle.
3. Host pulls; backend advances producer one frame (or to a bounded buffer); returns item / end / error.
4. Host drops handle → backend signals producer cancellation → producer finalizer runs.

## Detailed Design **[REQUIRED]**

- **Pull over push.** Pull yields backpressure and cancellation for free; push would force inventing a stop signal and complicate panic/`Send` safety across the FFI boundary. Decision recorded; callback-style push is rejected for the primitive.
- **`Value` rail first.** Ingestion records are schema-dynamic, so the high-value backends are Python and WASM (both already implement `ValueExecutor`). cdylib’s concrete-bincode path is lower priority and uses an FFI iterator-handle (`init → next(handle) → drop(handle)` exports).
- **WASM transport.** Start on stable Component Model using a `resource` with a `next()` method (poll pattern). Track the async `stream<T>`/`future<T>` ABI; adopt when wasmtime stabilizes it (ADR review trigger). Per-call optimizations from FIDIUS-T-0120 (cached `InstancePre`, typed raw-bytes) carry over to per-item.
- **Python transport.** Map the primitive onto a generator / `__next__` bridge; mind GIL and the in-process/subprocess models already present (`pyo3_thread`/`pyo3_process`/`pyo3_zerocopy` from the poc) for where backpressure cost lands.
- **Macro surface.** `fidius::Stream<T>` is an explicit, macro-legible marker (not inferred from `impl Trait`). The macro emits init/next/drop shims instead of the unary shim and folds the stream shape into the interface hash.
- **Harness correctness.** `fidius-test` helpers are correct-but-crude and *not* semver-committed; they must not model a wrong backpressure/error pattern since reference code gets copied to production.

### Phase 0 — Design decisions (LOCKED — signed off 2026-06-18 by Dylan Storey)

Five forks lock the primitive's shape. All signed off; D1 amended from the proposal (sync → async) on the grounds that every consuming host is a tokio app.

- **D1 — Host handle is a NATIVE ASYNC stream (v1).** `call_streaming` is async and yields a `ChunkStream: futures::Stream<Item = Result<Value, CallError>>` (concrete `Send` newtype over `Pin<Box<dyn Stream + Send>>`). *Rationale:* every consuming host is a tokio app, so native async avoids a thread-per-stream and a later sync→async retrofit; fidius-core already owns a tokio runtime (`FIDIUS_RUNTIME`) and feature-gated async (FIDIUS-T-0010). *Consequence (drives the backend bridges):* each backend adapts its native producer onto the stream — Python generator pumped on a blocking thread feeding a `tokio::mpsc`; WASM via wasmtime async (`call_async`); cdylib sync `next()` via `spawn_blocking` onto a channel. Cancellation (D3) = dropping the stream drops the bridge task, which closes the channel / aborts and triggers producer teardown. (Rejected: sync pull iterator — would force a thread-per-stream in the host and a retrofit we already know we need to avoid.)
- **D2 — Wire framing: tagged length-delimited frames.** Each frame = `[tag: u8][len: u32][payload]`, `tag ∈ {ITEM, END, ERROR}`. `ITEM` payload = the existing bincode/`Value` bytes; `ERROR` payload = serialized `PluginError`; `END` carries no payload. One framing, identical across backends (WIT `list<u8>` per `next()`, Python `bytes`, cdylib buffer). Keep a dedicated frame tag rather than overloading `STATUS_*`. Scheme stays batch-compatible (multiple `ITEM` frames).
- **D3 — Uniform `next()` + drop-cancel contract.** Backend exposes "advance one frame → one `Frame`"; `ChunkStream` wraps it. Cancellation = drop `ChunkStream` → backend tears down the producer in its native idiom (WASM resource drop; Python `close()`/`GeneratorExit`; cdylib `drop(handle)` export). Finalizer-runs-on-drop is REQ-002 and a required test.
- **D4 — Macro surface: return-position `fidius::Stream<T>` only (server-streaming) in v1.** `T` must be a wire/`WitType` type, same rules as unary returns. The signature string gains a `stream` marker so a streaming method hashes differently from a unary method of the same name/types (extends REQ-005). Client-streaming/bidi argument-position streams are out of v1.
- **D5 — Single-item pull granularity (v1).** `next()` yields one item; `next_batch(n)` is reserved as the NFR-001 optimization and is deliberately not in the v1 ABI shape, but the D2 frame scheme is chosen to admit it without a wire break.

**Phase-0 exit criteria:** these five signed off (or amended) → transition `design → ready`, then `ready → decompose` and cut Phase 1–4 tasks.

### Phase 2 — WASM design decisions (LOCKED — signed off 2026-06-19 by Dylan Storey)

Phase 1 (Python) is complete. These six lock the WASM server-streaming shape. Grounding facts from the existing `WasmComponentExecutor`: it is **all-sync wasmtime**, does **per-call instantiation** (fresh `Store`), and caches `InstancePre`. A stream instead holds **one Store+instance+resource for the stream's lifetime**.

- **W-D1 — WIT transport: exported `resource` with a `next()` poll method** (NOT the async `stream<T>`/`future<T>` ABI). Stable Component Model today; async streams are bleeding-edge and stay deferred (ADR review trigger). Resource-drop = the guest destructor = our D3 cancel.
- **W-D2 — `next()` signature: `next: func() -> result<option<T>, plugin-error>`.** `some(item)` = item, `none` = clean end, `err` = mid-stream error. Carries all three terminal states; mirrors the `Frame` ITEM/END/ERROR shape from ST.1 onto the resource.
- **W-D3 — `fidius::Stream<T>` becomes iterator-backed.** A Rust WASM guest must actually *produce* items, so the marker grows a real form (`Stream::from_iter(impl Iterator<Item=T>)`, holding `Box<dyn Iterator<Item=T> + Send>`). Additive: the Python/interface case never constructs it in Rust, so Phase 1 is unaffected. Lives in `fidius-guest` (wasm-buildable).
- **W-D4 — Relax the `#[plugin_impl]` streaming guard for wasm targets only.** ST.2 rejects all native streaming; WASM streaming *does* flow through `#[plugin_impl]`→component, so under `#[cfg(target_family="wasm")]` the macro emits the resource adapter; the cdylib (non-wasm) path keeps rejecting (Phase 3).
- **W-D5 — Host bridge: dedicated pump thread owns Store+instance+resource**, blocking `next()` → bounded `tokio::mpsc` → `ChunkStream`; dropping the `ChunkStream` drops the resource handle → guest dtor runs. Structurally identical to the ST.3 Python bridge (wasmtime is sync here), so the async/backpressure/cancel story is the proven one.
- **W-D6 — `WasmMethodDesc`/`WasmMethod` gains a `streaming` flag.** A streaming export returns `own<resource>`, not a value, so the executor must route to the resource path — it needs to know which methods stream (the macro sets the flag from `MethodIR.streaming`).

**Phase-2 decomposition (WS.1–WS.6):** foundation (WIT contract + iterator-backed `Stream<T>` + descriptor flag) → macro (resource adapter + relax guard + WIT gen) → host backend (`StreamExecutor` + routing + cap/signing parity) → Rust WASM E2E → polyglot proof → per-item perf/bench.

## Testing Strategy

### Unit Testing
- **Strategy**: Per-backend stream produce/consume, end-of-stream, mid-stream error frame, and drop-cancel-runs-finalizer. Use `stream_of`/`collect` to exercise single plugins in isolation.
- **Tools**: existing fidius-test harness + backend-specific fixtures (wasm-fixtures, python fixtures).

### Integration Testing
- **Strategy**: `pump()` a real streaming source plugin into a real streaming sink across each backend; assert output equivalence and bounded memory under a long (millions-of-items) synthetic stream; assert cancellation closes resources.
- **Test Environment**: workspace cargo tests + `pluggable-poc` E2E.

### System Testing
- **Performance Testing**: extend FIDIUS-T-0119/0120 benches to per-item streaming throughput/latency vs unary baseline and vs a TCP/UDS streaming control; confirm NFR-001/003.

## Alternatives Considered **[REQUIRED]**

- **Build the connector runtime in fidius** (Source/Destination + envelope + checkpoint + orchestration) — rejected in FIDIUS-A-0004 (policy, not mechanism; ties library to one domain).
- **Chunked-unary only** (`Vec<Object>` in/out, host loops) — already exists; can't hold live connection/cursor state or carry in-band checkpoints; fails the ingestion use case.
- **Push/callback transport** — rejected for the primitive: host loses free backpressure/cancel; FFI panic/`Send` hazards.
- **Supported `connect()`/pipeline DSL** — rejected; crosses from library into runtime and dictates orchestration to adopters whose product *is* orchestration. Composition stays test-tier.
- **WASM async `stream<T>` ABI now** — deferred; bleeding edge. Use the stable resource+`next()` poll pattern first.

## Implementation Plan **[REQUIRED]**

Sequencing is deliberately backend-by-backend (the ADR-blessed "prove on one backend first"); decompose into tasks at design-phase exit with human sign-off.

- **Phase 0 — Design lock**: finalize the `StreamExecutor`/`ChunkStream` shape, frame markers, and `fidius::Stream<T>` macro surface + hash treatment. Exit: signed-off design, no code.
- **Phase 1 — Python server-streaming (the wedge)**: primitive + macro marker + Pyo3 generator bridge + `fidius-test` harness (`stream_of`/`collect`/`pump`) + unit/integration tests.
- **Phase 2 — WASM server-streaming (the moat)**: resource+`next()` transport, capability/signing parity, per-item perf using cached `InstancePre`; polyglot proof reusing existing wasm-fixtures.
- **Phase 3 — cdylib server-streaming**: FFI iterator-handle ABI (`init/next/drop`), bincode framing, regression gate that unary paths are byte-identical.
- **Phase 4 — E2E + perf + docs**: `pluggable-poc` streaming example across backends; streaming throughput/latency benches; how-to doc; explicitly document the composition harness as unsupported and the connector-protocol/brokered-I/O boundary as adopter-owned.

**Dependencies / explicitly out of scope (tracked elsewhere):**
- **Brokered network I/O for the sandbox** — a deny-all WASM connector can't open a socket; sources need brokered HTTP/DB through the capability layer. Separate initiative; on the critical path for the *adopter*, not for this primitive. Flag before Phase 2 ships externally.
- Connector protocol + orchestration — adopter-owned per FIDIUS-A-0004.

## Progress Log

### 2026-06-18 — Phase 1 build started (Ralph)
- **ST.1 (T-0124) ✅** — `frame` wire (guest) + `StreamExecutor`/`ChunkStream` (host, `streaming` feature) + `CallError` variants. 17 tests green. Key finding: bincode can't reconstruct a self-describing `Value`, so ITEM→Value decode is caller-supplied and the Python bridge produces `Value`s natively via `ChunkStream::new`.
- **ST.4 (T-0127) ✅** — `fidius_test::stream` harness (`stream_of`/`collect`/`pump`/`StreamSink`/`CollectSink`), unsupported test-tier per ADR. 5 tests green. Closed independently of ST.3.
- **ST.2 (T-0125) ⏸ paused-active** — re-slice needed (logged on the task): "Client returns `ChunkStream`" couples to ST.3's `PluginHandle::call_streaming`; cdylib `init/next/drop` shim is Phase-3 code Python never runs. Proposed: ST.2 → interface-side only (parse marker + hash `!stream` + compile-fail), move Client codegen into ST.3, `#[plugin_impl]` errors on streaming methods until Phase 3.
- **ST.3 (T-0126) / ST.5 (T-0128)** — not started. These are the PyO3 generator-bridge + embedded-CPython E2E — the heaviest, most environment-sensitive work (GIL/drop-cancel race; embedded interpreter build+run).
- **Awaiting human decision**: confirm the ST.2 re-slice + whether to push autonomously into the PyO3 work or checkpoint here.

### 2026-06-19 — Phase 1 COMPLETE (all 5 tasks ✅)
Re-slice confirmed by Dylan; pushed through the PyO3 work.
- **ST.1 (T-0124) ✅** Frame wire + `StreamExecutor`/`ChunkStream` + `CallError` variants (17 tests).
- **ST.2 (T-0125) ✅** interface-side macro: `fidius::Stream<T>` marker + `!stream` hash + arg-position/native-impl rejection (IR unit tests + 2 trybuild fixtures).
- **ST.3 (T-0126) ✅** Pyo3 generator→`ChunkStream` bridge (GIL-per-step, bounded mpsc, drop→`close()`/finally) + `PluginHandle::call_streaming` (3 embedded-Python unit tests).
- **ST.4 (T-0127) ✅** `fidius-test` composition harness, unsupported test-tier (5 tests).
- **ST.5 (T-0128) ✅** capstone E2E: real Python generator plugin through `PluginHost`; 10M-item pull-3-drop in 0.02s proves bounded/backpressure/cancel; harness composition slice (4 tests). **Phase-1 exit gate PASSED.**
- **Verified**: `angreal build` + `angreal test` (default) green; `cargo test -p fidius-host --features python,streaming` green; no regressions.
- **Carried forward**: typed Client streaming method (descoped to `PluginHandle::call_streaming` to avoid cross-crate feature coupling); `python-stub`/descriptor `streaming` flag (hash already encodes it); `pluggable-poc` streaming example + per-item bench → Phase 4.
- **Initiative left in `decompose`** for human review; **Phase 2 (WASM) not yet decomposed** — awaiting go-ahead.

### 2026-06-19 — Phase 2 (WASM) essentially complete
Design-locked (W-D1…W-D6) and decomposed WS.1–WS.6; built with a spike-first approach.
- **WS.1 (T-0129) ✅** iterator-backed `Stream<T>` + WASM descriptor `streaming` flag + WIT contract.
- **WS.2 (T-0130) ✅** macro codegen — `#[plugin_impl]` auto-generates the streaming resource component (guest adapter driving `Stream<T>`); streaming plugins are wasm-only (cdylib path → `compile_error!`). `fidius-wit` renders the resource. Verified end-to-end via a macro-generated component.
- **WS.3 (T-0131) ✅** host `WasmComponentExecutor: StreamExecutor` — wasmtime `ResourceAny` pump (one `instantiate()`/stream, bounded channel, `resource_drop`=cancel). Spike (`tests/wasm-fixtures/ticker`) proved the wasmtime/wit-bindgen shape; **compiled first try**.
- **WS.4 (T-0132) ✅** Rust WASM E2E — `macro-ticker` fixture + `macro_wasm_streaming.rs` (3 tests): load via `load_wasm`, `tick(5)`→`[0..5]`, 10M bounded/cancellable.
- **WS.5 (T-0133) ✅** polyglot — a **JavaScript** guest (`ticker-js`, built via `jco`) serves the same streaming resource; host drives it with identical code (2 tests). Contract is language-neutral.
- **WS.6 (T-0134) 🟡** per-item bench (`stream_drain` group in `benches/backends.rs`, `streaming`-gated) added + compiles + runs; capturing numbers. Cost model: one `instantiate()`/stream, per-item = a `next()` on the live resource.
- **Piping / composition**: the `fidius-test` `pump` harness composes a real streaming plugin into a sink on **both** Python (`composition_pump_into_sink`) and WASM (`wasm_composition_pump_into_sink`) — backend-neutral "pipes of plugins".
- **Adopter dependency surfaced**: [[FIDIUS-T-0135]] (capability-gated `wasi:http` outbound HTTP for WASM guests) filed by the weir team — the brokered-I/O track FIDIUS-A-0004 anticipated; needed for real REST source connectors in the sandbox.
- **Verified**: `angreal build`/`test` (default) green; `cargo test -p fidius-host --features wasm,streaming` all green (incl. 6 wasm E2E + 2 JS polyglot); full macro + wit suites green.