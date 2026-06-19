---
id: cdylib-streaming-typed-bincode
level: task
title: "cdylib streaming: typed-bincode item fast path (skip JSON(Value))"
short_code: "FIDIUS-T-0137"
created_at: 2026-06-19T17:24:40.111905+00:00
updated_at: 2026-06-19T17:57:49.987308+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# cdylib streaming: typed-bincode item fast path (skip JSON(Value))

> Follow-up from [[FIDIUS-T-0136]] (CS.1 cdylib streaming). Optimization, not a correctness issue — cdylib streaming works today.

## Objective **[REQUIRED]**

Give cdylib server-streaming a **typed, concrete-bincode item fast path** so it reclaims its "fastest backend" position, instead of paying the self-describing **JSON(`Value`)** encoding CS.1 ships today.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

### Type
- [x] Tech Debt — performance/refactor

### Priority
- [x] P3 — cdylib streaming already works at ~1.6 µs/item (native-class); this is a speed optimization for high-throughput in-process streams.

### Technical Debt Impact
- **Current problem**: CS.1's `next` shim encodes each item as `serde_json::to_vec(item)` and the host decodes JSON→`Value`. Measured **~1.64 µs/item for cdylib vs ~1.37 µs for wasm/Rust** (`stream_drain` bench) — cdylib is *slower than the sandboxed wasm path* despite being in-process native, purely because of the JSON hop. (bincode can't reconstruct a self-describing `Value`, so JSON was the pragmatic v1 wire that needs no compile-time `T`.)
- **Benefit**: cdylib's whole reason for existing is the fastest path. A typed item wire would likely put it back below wasm/Rust per item.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] cdylib streaming items cross as **concrete bincode of `T`** (no JSON, no `Value` hop) on the item hot path, matching the unary cdylib `call_method` wire.
- [ ] The host yields decoded `T`→`Value` only where the `ChunkStream<Value>` boundary requires it, with the decode driven by a **caller-supplied decoder** (the typed Client knows `T`) — reusing `ChunkStream::from_frame_bytes(frames, decode_item)` which ST.1 built for exactly this, rather than the host needing `T`.
- [ ] Measured per-item cdylib ≤ wasm/Rust in the `stream_drain` bench (or documented why not).
- [ ] No change to Python/WASM streaming; the error/end framing is preserved; drop-cancel still runs `drop_fn`.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
- The crux ST.1 already anticipated: `ChunkStream` is `Value`-typed, but cdylib items are concrete `T`. Options: (a) add a typed `PluginHandle::call_streaming_typed<O>` that the generated Client uses for cdylib (decodes `bincode(O)` per item, then `to_value`), routing the executor's raw frame-byte stream through `from_frame_bytes` with a `wire::deserialize::<O>` decoder; (b) keep `Value` items but use a self-describing **binary** codec (messagepack) instead of JSON — smaller win, no API change. (a) is the real fast path.
- Touch points: `fidius-guest::stream_ffi` (item encoder), the macro `next` shim, `CdylibExecutor::call_streaming_raw`, and the generated Client streaming method (currently skipped — would need to thread the decoder).

### Dependencies
- Builds on [[FIDIUS-T-0136]]. Relates to the deferred "typed streaming client" (the Client streaming method skipped in WS.2/ST.3).

## Status Updates **[REQUIRED]**

### 2026-06-19 — implemented, but the hypothesis was mostly wrong ⚠️
Implemented the typed-bincode item path and **merged it** (it's a cleaner design regardless):
- Macro `next` shim now `wire::serialize`s the item (concrete bincode — byte-identical to the unary cdylib wire) instead of `serde_json`. Removed `stream_item_encode`/`stream_item_decode` from `fidius-guest::stream_ffi`.
- `PluginHandle::call_streaming<I, O>` gained the item type `O`; the cdylib arm threads `cdylib_stream_decode::<O>` (a `fn(&[u8]) -> Result<Value, _>` = `wire::deserialize::<O>` + `to_value`) into `CdylibExecutor::call_streaming_raw`. Python/WASM ignore `O`. ~20 call sites annotated `::<_, u64>`.
- All streaming suites green; default workspace 50/0.

**Measured result (same-run `stream_drain`, N=10k):** cdylib **1.68 µs → 1.61 µs** (~4%). wasm/Rust 1.41 µs same run. **The gap narrowed but did NOT flip** — cdylib is still ~14% above wasm/Rust.

**Conclusion — JSON was not the dominant cost.** For tiny integers, JSON encode+parse was only ~70 ns. The real per-item overhead is the **PluginAllocated handshake**: the guest `malloc`s a `Box<[u8]>` per item and the host calls back through a *second* FFI boundary (`free_buffer`) to free it — **2 FFI crossings + alloc + free per item**, where wasm does **1** wasmtime call and manages component memory internally. That ~0.2 µs is structural to the buffer model, not the wire.

**Kept anyway** because it makes cdylib streaming use the same concrete-bincode wire as unary cdylib (consistency) and removes `serde_json` from the hot path. AC #1/#2/#4 met; **AC #3 (cdylib ≤ wasm) NOT met** — the real fix is arena-style item buffering, spun out to [[FIDIUS-T-0138]].