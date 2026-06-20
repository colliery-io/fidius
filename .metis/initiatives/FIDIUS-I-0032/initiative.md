---
id: bidirectional-streaming-stream-in
level: initiative
title: "Bidirectional streaming — Stream in both arg and return (synchronous lazy-pull)"
short_code: "FIDIUS-I-0032"
created_at: 2026-06-20T22:20:06.272365+00:00
updated_at: 2026-06-20T22:20:06.272365+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
initiative_id: bidirectional-streaming-stream-in
---

# Bidirectional streaming — Stream in both arg and return (synchronous lazy-pull)

## Context **[REQUIRED]**

Server-streaming (FIDIUS-I-0026, plugin produces) and client-streaming (FIDIUS-I-0030,
host produces) both ship. **Bidirectional** — `fn transform(&self, input: Stream<In>) ->
Stream<Out>` — composes them: consume a stream, produce one. Per **ADR-0010** we build it
as a **synchronous lazy-pull composition** (NOT a concurrent two-pump): the returned
`Stream<Out>` is lazy, and pulling an `Out` re-enters the input producer to pull `In` on
demand — one re-entrant call stack, single-threaded, no deadlock. This is the additive
co-occurrence of the two shipped mechanisms, decomposed per backend exactly like the CS2.x
client-streaming arc. Supersedes ADR-0007 §4.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- A method may take `Stream<In>` AND return `Stream<Out>`; the macro accepts it, the hash
  carries both markers (`<stream` + `!stream`), and codegen composes input-pull with
  output-stream on **cdylib, WASM, and Python** (all three this initiative — per the
  user's scope choice).
- Output drives input by lazy re-entrant pull (ADR-0010 model). Drop = cancel the whole
  chain; backpressure = host output pull rate.
- An E2E per backend: a transform plugin (e.g. running-sum, or filter/double) fed a
  host-produced `In` stream, host pulls the `Out` stream, asserts the transformed sequence.
- The re-entrancy invariant (host re-entered through its producer during an output pull) is
  explicitly tested.

**Non-Goals:**
- Concurrent/decoupled in-out rates (ADR-0010 rejected the two-pump model; uneven rates are
  the plugin's internal buffering).
- User-typed (`#[derive(WitType)]`) stream items in bidi on WASM/cdylib (inherits the same
  primitives/String limit as client-streaming; a shared follow-on).
- Async plugin bodies / new cancellation primitives (drop-cancel is inherited).

## Detailed Design **[REQUIRED]**

Per ADR-0010, each backend composes its existing client-streaming (input) + server-streaming
(output) shapes:

- **cdylib** — a combined vtable entry `BidiStreamFn(instance, input_producer_handle, args,
  *out_stream_handle) -> status`: the client-streaming producer handle is passed in, the
  server-streaming output stream handle is returned. The macro shim builds the user method's
  `Stream<In>` from the input handle (`HostStream`, CS2.2), calls the method to get
  `Stream<Out>`, and hands back an output stream handle (server-streaming, CS.1/ST). The host
  drives `out.next()`, which re-enters `input_producer.next()`.
- **WASM** — the component imports `fidius:stream-pull/pull.next` (input, CS2.3) AND exports
  the streaming resource (output, WS). The macro emits the method WIT with no stream param and
  a resource return; the guest body builds the input `Stream` from `WasmHostStream` and returns
  the lazy output. Host sets the producer + pulls the output resource (re-enters the import).
- **Python** — the method receives the `HostFedStream` iterator (input, CS2.4) AND returns a
  generator (output, ST). Host feeds items + pulls the generator (which pulls the iterator).

**Re-entrancy invariant:** the host producer state must not be locked across the output-`next`
call. The shipped `host_producer_handle` already owns its state behind the handle (fn-pointer
invocation), so this holds; tested per backend.

## Alternatives Considered **[REQUIRED]**

See ADR-0010's table. Summary: **concurrent two-pump** rejected (deadlock-avoidance + async/
threads across FFI × 3 backends; fights the pull model; no proven demand for decoupled rates).
**Status quo** (round-trip through the host as two calls) rejected as the motivating reason to
build this — no single plugin-owned transform.

## Implementation Plan **[REQUIRED]**

Mirrors the CS2.x arc. Tasks BD.1–BD.5:

- **BD.1** — macro/IR: accept a method with BOTH `Stream` arg and `Stream` return; combined IR
  (`client_stream_item` + `stream_item` co-present) + hash carries both markers; macro routing
  scaffold (no per-backend codegen yet). Foundation.
- **BD.2** — cdylib: `BidiStreamFn` vtable shape + macro shim (input `HostStream` → user method →
  output stream handle) + host call path (`call_bidi_streaming`) + re-entrancy + E2E.
- **BD.3** — WASM: import (input) + resource export (output) co-occurrence; macro wasm codegen;
  host wires producer + output resource pump; E2E.
- **BD.4** — Python: `HostFedStream` arg + generator return; host call path; E2E.
- **BD.5** — docs (streaming.md bidirectional section) + a transform-connector example + un-defer;
  cross-ref ADR-0010; note the user-typed-item follow-on.
