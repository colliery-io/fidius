---
id: 001-bidirectional-streaming-via
level: adr
title: "Bidirectional streaming via synchronous lazy-pull composition"
number: 1
short_code: "FIDIUS-A-0010"
created_at: 2026-06-20T22:19:09.133616+00:00
updated_at: 2026-06-20T22:22:36.986424+00:00
decision_date: 
decision_maker: dylan.storey
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-10: Bidirectional streaming via synchronous lazy-pull composition

## Context **[REQUIRED]**

fidius now ships both one-directional streaming halves:

- **Server-streaming** (FIDIUS-I-0026): a method *returns* `fidius::Stream<T>`. The
  **plugin produces**, the host pulls lazily — a stream handle/resource the host calls
  `next()` on (`ChunkStream`). Drop = cancel; backpressure is the host's pull rate.
- **Client-streaming** (FIDIUS-I-0030 / ADR-0007): a method *takes* `fidius::Stream<T>`
  as an argument. The **host produces**, the plugin pulls — a host producer handle
  (cdylib), the `fidius:stream-pull` import (WASM), or a host-fed iterator (Python).

**Bidirectional** — `fn transform(&self, input: Stream<In>) -> Stream<Out>` — consumes
a stream *and* produces one. ADR-0007 deferred it as "a separate, later decision … two
pumps concurrently (deadlock-avoidance)". That framing assumed the input and output run
as **independent concurrent pumps** — which is what makes it expensive. This ADR records
that we will instead build it as the **synchronous composition** of the two shipped
mechanisms, which removes the concurrency (and the deadlock class) entirely.

The motivating shape is a **streaming transform/filter connector**: read an inbound
stream, emit a transformed outbound stream, with the plugin controlling how it batches
(e.g. parse → enrich → re-emit, or windowed aggregation that emits one `Out` per N `In`).

## Decision **[REQUIRED]**

Build bidirectional streaming as a **synchronous lazy-pull composition**, not a
concurrent two-pump.

1. **Runtime model.** The returned `Stream<Out>` is **lazy** (exactly like
   server-streaming). When the host pulls an `Out` item, the plugin's output iterator
   runs and **pulls `In` items from the input stream on demand** (exactly like
   client-streaming, host-produced). It is one re-entrant call stack, single-threaded
   per call:

   > host → `output.next()` (into plugin) → plugin → `input.next()` (back into host
   > producer) → host yields `In` → plugin computes → returns `Out`.

   No threads, no channels, no concurrent pumps, **no deadlock**. The plugin's body is
   typically `Stream::from_iter(input.map(transform))` or a stateful adapter that pulls
   0..N `In` per emitted `Out` (internal buffering is the plugin's choice). In/out rates
   are **coupled through the plugin's own iterator** — acceptable because a plugin that
   needs uneven rates buffers internally; we found no connector use case that needs
   truly independent pumps.

2. **Per-backend shape — additive, composing the two existing shapes:**
   - **cdylib** — a combined vtable entry: `BidiStreamFn(instance, input_producer_handle,
     args, *out_stream_handle) -> status`. The client-streaming **producer handle is
     passed in**; the server-streaming **output stream handle is returned**. The host
     then drives `out_stream_handle.next()`, which re-enters `input_producer.next()`.
   - **WASM** — the component **both** imports `fidius:stream-pull/pull.next` (input)
     **and** exports the streaming **resource** (output). Pulling the output resource's
     `next()` re-enters the host's `pull.next` import. (A host import invoked from inside
     a host-initiated guest call is standard wasmtime; same import-version discipline as
     ADR-0005/CS2.3.)
   - **Python** — the method **receives** the host-fed iterator (input, ADR-0007 CS2.4
     `HostFedStream`) **and returns** a generator (output, I-0026). Pulling the generator
     pulls the iterator. Single-threaded Python.

3. **Interface hash.** A bidirectional method carries **both** markers — the arg-position
   `<stream` (ADR-0007 §3) **and** the return-position `!stream` — so its hash is
   distinct from unary, server-streaming, and client-streaming. Both markers already
   exist; bidirectional is their co-occurrence, requiring no new marker.

4. **Re-entrancy is the load-bearing correctness property.** The host is re-entered
   through its input producer *while servicing an output pull*. The host producer state
   must therefore not be held under a lock across the output-`next` call (the shipped
   `host_producer_handle` already owns its state behind the handle and is invoked by
   function pointer, so this holds). This is documented and tested explicitly per backend.

5. **Cancellation & backpressure fall out for free.** Dropping the output stream tears
   down the whole chain (the plugin's iterator drops, releasing the input handle);
   backpressure is the host's output pull rate, propagated to the input pull rate by the
   coupling. No new cancellation/backpressure machinery.

## Alternatives Analysis

| Option | Pros | Cons | Risk Level | Implementation Cost |
|--------|------|------|------------|-------------------|
| **Synchronous lazy-pull composition (chosen)** | Additive — reuses both shipped streaming halves in composition; no new concurrency model; drop/backpressure inherited; no deadlock class | In/out rates coupled (uneven rates need plugin-internal buffering); a combined ABI shape + re-entrancy correctness per backend | Medium | L |
| **Concurrent two-pump** | Fully decoupled in/out rates | Two concurrent pumps + bounded channels + deadlock-avoidance; async/threads across the FFI boundary on all 3 backends; fights fidius's single-threaded-per-call pull model | High | XL |
| **No bidirectional (status quo)** | Zero cost | A transform-stream connector must round-trip through the host (pull server-stream, re-feed via client-stream) as two calls — no single plugin-owned transform | Low | none |

## Rationale **[REQUIRED]**

The expensive part of bidirectional is *concurrency*, not *direction*. ADR-0007 priced
it as XL because it assumed independent pumps. But fidius streaming is **pull-based and
single-threaded per call** in both directions already — so composing them as a lazy
output that pulls input on demand is just *nesting* two mechanisms that each work, on one
call stack. That collapses the cost from "new concurrent ABI × 3 backends" to "a combined
vtable/import/iterator shape × 3 backends, plus re-entrancy correctness" — the same
additive character as client-streaming itself. The only thing we give up is independent
in/out *rates*, which a plugin recovers with internal buffering; no adopter shape we can
find needs more. Recording the synchronous model as the decision keeps bidirectional in
the same architectural family as everything else fidius streams.

## Consequences **[REQUIRED]**

### Positive
- Additive: no new concurrency primitive; reuses client- + server-streaming in composition.
- Cancellation (drop tears down the chain) and backpressure (host pull rate) are inherited,
  not re-engineered.
- Hash safety is free — the co-occurrence of the two existing markers is already distinct.

### Negative
- Input and output rates are coupled through the plugin's iterator; uneven rates require
  plugin-internal buffering (no independent pumping).
- A combined ABI shape per backend (cdylib vtable entry, WASM import+export co-occurrence,
  Python iterator-in/generator-out) plus a re-entrancy invariant to maintain and test.

### Neutral
- Supersedes ADR-0007 §4's "separate later decision" for bidirectional; ADR-0007 remains
  the client-streaming record.
- The implementation lands as its own initiative (FIDIUS-I-0032) decomposed per backend,
  mirroring the CS2.x arc.

## Review Schedule

### Review Triggers
- An adopter needs genuinely **decoupled** in/out rates that plugin-internal buffering
  cannot express (would reopen the concurrent two-pump alternative under a new ADR).
- A backend cannot satisfy the re-entrancy invariant (host re-entered through its producer
  during an output pull) — would force a redesign for that backend.