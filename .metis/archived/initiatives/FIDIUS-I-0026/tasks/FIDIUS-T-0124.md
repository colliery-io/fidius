---
id: st-1-core-streaming-types-frame
level: task
title: "ST.1 — Core streaming types: Frame wire + StreamExecutor trait + ChunkStream (async)"
short_code: "FIDIUS-T-0124"
created_at: 2026-06-18T18:13:31.533459+00:00
updated_at: 2026-06-19T01:32:42.562174+00:00
parent: FIDIUS-I-0026
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0026
---

# ST.1 — Core streaming types: Frame wire + StreamExecutor trait + ChunkStream (async)

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0026]] · Phase 1 foundation · implements design decisions D1/D2/D3.

## Objective **[REQUIRED]**

Land the backend-agnostic core of the streaming primitive — wire framing, the executor seam, and the host-facing async stream type — with **no backend implementation yet**. This is the shared scaffolding every backend (Python/WASM/cdylib) plugs into. Purely additive: existing unary paths untouched.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] `Frame` type + (de)serialization in `fidius-guest`/`fidius-core`: `[tag: u8][len: u32][payload]`, `tag ∈ {ITEM, END, ERROR}` (D2). `ITEM` payload = existing bincode/`Value` bytes; `ERROR` = serialized `PluginError`; `END` = empty. Round-trip + truncated/garbage-frame tests.
- [ ] `StreamExecutor` trait (extends `PluginExecutor`): `async fn call_streaming(&self, method: usize, args: Value) -> Result<ChunkStream, CallError>` (async-trait or RPITIT) (D1).
- [ ] `ChunkStream`: concrete `Send` newtype over `Pin<Box<dyn futures::Stream<Item = Result<Value, CallError>> + Send>>`; `.next().await` yields items until `END`; an `ERROR` frame surfaces as `Err(CallError::…)` then terminates.
- [ ] Drop-cancel contract documented on the trait: dropping `ChunkStream` must propagate teardown to the producer (mechanism per-backend; this task only fixes the contract + a `Drop` hook point) (D3).
- [ ] `CallError` gains any streaming-specific variants needed (e.g. malformed frame, stream aborted) without breaking existing variants (additive).
- [ ] No backend wired; trait has no impls yet beyond a `stream_of`-style in-memory test double sufficient to unit-test `ChunkStream` decode/EOS/error/drop semantics.
- [ ] Existing unary cdylib/Python suites unchanged and green; new code is feature-gated consistently with existing async gating.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
- Put `Frame` next to the existing `wire` module (bincode is already the per-item currency); reuse `wire::serialize`/`deserialize` for `ITEM` payloads so streaming inherits the unary wire verbatim.
- `ChunkStream` is the only host-facing type; keep the `dyn Stream` boxed so backends can return wildly different concrete streams (mpsc receiver, wasmtime poll adapter) behind one type.
- Lean on `fidius-core`'s existing `FIDIUS_RUNTIME` and feature-gated async (FIDIUS-T-0010); mirror that gating so non-async builds are unaffected.

### Dependencies
- None upstream — this is the root of the Phase 1 graph. Blocks ST.2 (macro), ST.3 (Pyo3 backend), ST.4 (harness).

### Risk Considerations
- async-trait vs RPITIT: pick based on whether `StreamExecutor` needs object safety (the `Backend` enum may avoid `dyn`, as the executor module already prefers an enum over `Box<dyn>`). Decide here and document.
- Keep `END` vs `ERROR` precedence unambiguous: an `ERROR` frame terminates the stream; no items may follow. Encode that in the decode state machine + tests.

## Status Updates **[REQUIRED]**

### 2026-06-18 — ST.1 complete ✅
- **`fidius_guest::frame`** (wasm-buildable, default build): `Frame {Item(Vec<u8>), End, Error(PluginError)}`, `FrameError`, tags `FRAME_ITEM/END/ERROR`, `[tag:u8][len:u32 LE][payload]` codec — `encode` / `decode` (single-frame) / `read` (walk concatenated frames, returns bytes consumed). Re-exported via `fidius_core::frame`. **11 unit tests** (round-trips, empty item, truncated header/payload, unknown tag, END-with-payload, trailing-byte, garbage-no-panic).
- **`fidius_host::stream`** behind new **non-default `streaming` feature**: `StreamExecutor` (async-trait, `async fn call_streaming -> ChunkStream`) + `ChunkStream` (a `futures::Stream` of `Result<Value, CallError>`). Constructors: `new` (native Value-rail), `from_frame_bytes`/`from_frames` (terminal-frame state machine + caller-supplied item decoder). **6 async unit tests** (items→END, native Values, ERROR-terminates, missing-terminal→abort, malformed→stop, empty).
- **`CallError`**: added `MalformedFrame(String)` + `StreamAborted` (additive; existing variants untouched).
- **Deps**: workspace gained `futures`, `async-trait`; host gained optional `futures`/`async-trait`/`tokio` gated on `streaming`.
- **Design correction (shapes ST.3):** vanilla bincode cannot reconstruct a self-describing `Value` (`deserialize_any` unsupported — matches the executor.rs note). So there is **no fixed bytes→Value decode**: ITEM-payload decoding is caller-supplied (the typed client knows `T`), and the Python in-process bridge will produce `Value`s **natively** via `ChunkStream::new`, not through frame bytes. Frame bytes are for the serialized backends (WASM/cdylib).
- **Verified**: `fidius-guest frame::` 11/11 · `fidius-host --features streaming --lib stream::` 6/6 · default-feature `cargo check` (core/guest/host) green → REQ-006 additive confirmed.
- **Decision**: async-trait over RPITIT (object-safety not needed yet; reads cleanly, keeps the door open). Revisit if `Backend` enum dispatch makes RPITIT preferable in ST.3.