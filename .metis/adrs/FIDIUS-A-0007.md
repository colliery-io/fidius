---
id: 001-client-streaming-via-a-host
level: adr
title: "Client-streaming via a host-provided pull channel (deferred; design of record)"
number: 1
short_code: "FIDIUS-A-0007"
created_at: 2026-06-20T14:44:31.195473+00:00
updated_at: 2026-06-20T14:47:18.981136+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-1: Client-streaming via a host-provided pull channel (deferred; design of record)

## Context **[REQUIRED]**

Server-streaming shipped (FIDIUS-I-0026): a method returns `fidius::Stream<T>`, the
*plugin produces* and the host pulls (`ChunkStream`). The dual — **client-streaming**,
where `Stream<T>` is an *argument* (the *host produces*, the plugin consumes) — and
**bidirectional** are deliberately refused today: `Stream<T>` in argument position is
a compile error (`crates/fidius-macro/src/ir.rs:519`, pinned by the trybuild test
`compile_fail/stream_in_arg_position.rs`):

> *fidius v1 supports server-streaming only: `Stream<T>` is not allowed in argument
> position (client-streaming and bidirectional are deferred).*

The motivating use case is a **writer/sink connector** that wants *plugin-controlled
consumption* — `fn write(&self, rows: Stream<Row>) -> Summary` where the plugin
batches N rows, looks ahead, or backpressures the source itself. This ADR records the
**design we'll use when we build it** and **why it stays deferred** for now.

## Decision **[REQUIRED]**

1. **Deferred** — client-streaming is not built in the near term. The supported way
   to feed a sequence into a plugin remains the **host-pump pattern** (`examples/04_pipeline`,
   `multi_plugin_pipeline.rs`): the host pulls a server-stream and calls a *unary*
   method per item. This covers the common ingestion/transform/bulk-load case.
2. **Design of record (when built):** a **host-provided pull channel**, NOT a
   re-architecture of the one-directional ABI. The plugin's method *calls back into
   the host* for the next item; the host owns the producer:
   - **cdylib** — the host passes in a callback the plugin invokes in a loop:
     `next(state) -> (status, item_bytes)` (+ a drop/cancel). A small, additive
     host→plugin function table — the inverse of the existing iterator-handle ABI.
   - **WASM** — the component **imports** a `fidius-stream-next` function the host
     supplies; the method pulls via the import (same import-version discipline as
     wasi:http, ADR-0005).
   - **Python** — the method receives a **host-backed generator** (PyO3 bridge in the
     pull direction).
3. **Interface hash** gains a distinct arg-position marker (e.g. `<stream`), separate
   from the return-position `!stream`, so a client-streaming method's hash can't
   collide with a unary or server-streaming one.
4. **Bidirectional is a separate, later decision** — it composes a host→plugin pull
   with a plugin→host push concurrently (deadlock-avoidance, two pumps); out of scope
   here. See [[FIDIUS-I-0030]] for the client-streaming implementation plan.

## Alternatives Analysis

| Option | Pros | Cons | Risk | Cost |
|--------|------|------|------|------|
| **Host-provided pull channel (chosen design; deferred)** | Additive — reuses the per-backend streaming machinery in the inverse direction; the host stays the orchestrator | A new bidirectional ABI shape × 3 backends; new hash marker; inverted backpressure/cancel | Medium | L |
| **Host-pump only (status quo)** | Zero new ABI; works today; host owns wiring | No *plugin-controlled* consumption — the plugin can't batch/look-ahead/backpressure; one host call per item | Low | none |
| **`Vec<T>` argument ("chunked unary")** | Already supported; trivial | Unbounded host memory (the whole sequence buffers before the call); defeats streaming | Low | none |
| **Full bidirectional now** | Most general | Two concurrent pumps + deadlock-avoidance; highest cost; little proven demand | High | XL |

## Rationale **[REQUIRED]**

The host-pump already delivers ~80% of "stream a sequence into a plugin," so the
*marginal* value of client-streaming is the narrow case where the **plugin** needs
control over how it pulls (batch-then-flush, look-ahead, source backpressure). That
value doesn't justify a new bidirectional ABI across three backends *yet* — but when
an adopter does need it, the pull-channel design keeps it **additive** (the inverse of
the shipped server-streaming handles) rather than a re-architecture. Recording the
design now means the deferral is a *scheduling* choice, not an unknown.

## Consequences **[REQUIRED]**

### Positive
- The deferral is principled and reversible: a specced design ([[FIDIUS-I-0030]]) ready
  to execute when demand appears.
- The host-pump remains the documented, sufficient answer for the common case.

### Negative
- Until built, a plugin cannot control its own consumption (no plugin-side batching
  /look-ahead/backpressure) — only host-driven per-item calls.
- When built: a third ABI shape to maintain per backend; a new hash marker.

### Neutral
- The compile-fail guard stays as the explicit "not yet" until [[FIDIUS-I-0030]] lands.

## Review Schedule

### Review Triggers
- An adopter needs plugin-controlled consumption (batch/look-ahead/source backpressure)
  that the host-pump can't express.
- Bidirectional streaming is requested (would supersede/extend this with its own ADR).