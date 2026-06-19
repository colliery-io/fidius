---
id: 001-streaming-as-mechanism-not
level: adr
title: "Streaming as Mechanism, Not Protocol — fidius ships the typed pipe, not the connector runtime"
number: 1
short_code: "FIDIUS-A-0004"
created_at: 2026-06-18T17:16:07.589697+00:00
updated_at: 2026-06-18T17:19:57.045901+00:00
decision_date: 
decision_maker: Dylan Storey
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-1: Streaming as Mechanism, Not Protocol — fidius ships the typed pipe, not the connector runtime

## Context **[REQUIRED]**

Adopters are asking "what about streaming?" The motivating adopter is building an **Airbyte-like ingestion / reverse-ETL product**: long-lived sources that emit an unbounded sequence of records, destinations that consume that sequence, with checkpointing, backpressure, and (for untrusted community connectors) sandboxing.

Every fidius method today is **unary**: `fn apply(&self, In) -> Out`, fully buffered both ways, serialized as bincode (cdylib) or a self-describing `Value` (Python/WASM) across the executor seam (`PluginExecutor::call_raw`, `ValueExecutor::call`). The uniformity of that `bytes -> bytes` / `Value -> Value` seam is what makes the three-backend architecture clean.

The seductive-but-wrong move is to read the adopter's request as a spec and build the connector runtime *into fidius*: a `Source`/`Destination` trait pair, an `AirbyteMessage`-style envelope, in-band `State`/checkpoint semantics, catalog/discover, incremental cursors, orchestration. That would re-tie a general mechanism to one domain — fidius would become "a worse Airbyte" and stop being reusable infrastructure.

The decision this ADR records is **where the layering boundary sits**: what streaming functionality belongs *in* fidius-the-library versus *above* it in the adopter's product.

## Decision **[REQUIRED]**

**fidius ships the pipe, not the protocol.** We adopt the Unix kernel discipline — *mechanism, not policy*:

- **In fidius (mechanism, semver-supported):**
  1. A **streaming transport primitive** — a pull-based, backpressured, framed channel of `Value` (and raw bytes for `#[wire(raw)]`), uniform across cdylib / Python / WASM. Pull gives backpressure and cancellation for free (stop pulling / drop the handle → plugin observes it and releases resources).
  2. The **`fidius::Stream<T>` marker** in the trait macro so an interface method can declare it yields/consumes a stream — codegen, ABI shim, and interface-hash coverage. The trait remains the contract; the hash still proves both ends agree.
  3. The existing **sandbox + capability allow-list + signing** continue to apply to streaming calls unchanged (the capability *system* is mechanism; *which* connector gets *which* capability is policy and stays the adopter's).

- **NOT in fidius (policy — lives above, in the adopter's product or an optional non-core crate):**
  - The connector protocol: `Message`/`Record`/`State` envelope, checkpoint/resume semantics, cursor/incremental logic, catalog/`discover()` schema.
  - Orchestration: scheduling, retries, parallelism, observability, the sync engine.
  - Concrete I/O brokers (e.g. a host-side HTTP capability impl) MAY ship as optional, separately-versioned crates (`fidius-cap-*`), never in core.

- **Composition is demoted to an unsupported testing affordance**, not a library API. A small, *correct-but-not-semver-committed* harness in `fidius-test` (`stream_of(Vec) -> Stream`, `collect(Stream) -> Vec`, `pump(out, into)`) makes composition trivial to test without signaling that fidius orchestrates pipelines. Production adopters write their own pump loop; the `pluggable-poc` orchestrator remains the worked example.

The key enabling insight: the `Message` envelope, checkpoints, and catalog are all just an agreed-upon *schema of `Value`s* plus a *trait convention*. fidius does not need to understand any of them to transport them — exactly as the kernel moves bytes without understanding JSON-lines. Therefore they must not live in fidius.

A second clarifying insight: ingestion records are **schema-dynamic** (a "connect to any Postgres" source learns its columns at runtime via `discover()`), so connectors live on the self-describing `Value` rail — which is exactly the Python and WASM backends. cdylib's concrete-bincode superpower is irrelevant to this adopter, and that convergence reinforces the boundary.

## Alternatives Analysis **[CONDITIONAL: Complex Decision]**

| Option | Pros | Cons | Risk Level | Implementation Cost |
|--------|------|------|------------|-------------------|
| **A. Pipe only — streaming primitive + `Stream<T>` marker; protocol & orchestration stay above (CHOSEN)** | Stays general; one mechanism serves connectors *and* LLM token streams, log tailing, progress, search; keeps three-backend seam uniform; preserves "infrastructure not app" positioning; small supported surface | Adopter must build their own connector protocol + orchestration; fidius doesn't "do streaming connectors" out of the box | Low | Medium (one primitive, three backend impls, macro marker) |
| **B. Connector runtime in fidius — `Source`/`Destination` traits, `Message` envelope, checkpoints, catalog, orchestration** | "Batteries-included" for the one adopter; turnkey ingestion | Ties general library to one domain; becomes "a worse Airbyte"; huge forever-maintenance surface; competes in a crowded app market; serious adopters route around our opinions | High | Very High |
| **C. Chunked-unary only — no new ABI; plugin takes `Vec<Object>`, host drives the loop** | Zero ABI change; already exists | Cannot hold live connection state / cursors across calls; no in-band checkpoint stream; no plugin-driven cadence; fails the actual ingestion use case | Low | None (already present) |
| **D. Supported `fidius::orchestrate()` / pipeline DSL** | Turnkey composition | Dictates orchestration/retry/observability to an adopter whose product *is* those opinions; crosses from library into runtime | Medium | High |

## Rationale **[REQUIRED]**

- **The only genuine mechanism gap is the streaming primitive.** Everything else the adopter needs is expressible as ordinary typed methods plus a streamed `Value` channel once that primitive exists. Building more would be building policy.
- **Generality compounds.** Scoping the primitive to "typed sandboxed pipe" instead of "Airbyte transport" lights up many use cases (token streams, progress, log/event subscriptions) for the same cost, and keeps fidius positioned as infrastructure every such product wants rather than one product competing in ETL.
- **The moat is the sandbox + capabilities + signing + multi-backend hash-verified ABI**, not connector semantics. "Typed, sandboxed, backpressured pipes with a verified interface contract across native/Python/WASM" is a hard, narrow, valuable thing to be. The Airbyte pitch ("connectors without Docker") is *validation that the pipe is worth building*, not a spec for fidius to implement.
- **Chunked-unary (C) genuinely can't do it** — it can't hold a live DB cursor / HTTP keep-alive / auth-token / rate-limit budget across calls, and has no in-band checkpoint channel. True server-streaming earns its cost precisely where the connector holds live connection state and emits at its own cadence.
- **Unsupported composition harness** resolves the "do we ship `connect()`" tension: adopters get a trivial way to *test* composition; we avoid signaling that fidius orchestrates; the disclaimer is real where it matters (our maintenance burden; the serious adopter's mental model) and harmlessly leaks where it doesn't (casual copy-paste of correct code). The harness must be *correct-but-crude*, never *misleading*, because a reference implementation gets copied into production whether blessed or not.

## Consequences **[REQUIRED]**

### Positive
- Small, sharp, semver-supported surface; large maintenance liabilities (envelope, checkpoint, orchestration) never enter core.
- One primitive serves many domains beyond ingestion.
- Clear message to adopters — "fidius is the kernel; you own the OS" — reinforced by composition being test-tier, not API.
- Three-backend seam stays uniform; streaming is one new method shape, not a per-backend protocol.

### Negative
- The motivating adopter must build their own connector protocol and orchestration on top; fidius is not turnkey for ingestion.
- "Streaming" for a sandboxed WASM connector is inseparable from **brokered network I/O** (a deny-all sandbox can't open a socket). That capability-broker work is a real, separate critical-path track this ADR explicitly pushes *outside* core — risk is that adopters perceive a gap until at least a reference broker exists.
- Risk that an unsupported-but-good test harness is used in production anyway; mitigated by keeping its semantics correct.

### Neutral
- Records cross as self-describing `Value`, so this adopter lives on the Python/WASM rail; the cdylib concrete-bincode path is unaffected and simply not the connector path.
- Bidirectional and client-streaming shapes remain in the design space but out of the first cut (see FIDIUS-I-0026).

## Review Schedule **[CONDITIONAL: Temporary Decision]**

### Review Triggers
- Multiple independent adopters converge on the *same* connector-protocol shape (envelope/checkpoint), suggesting a candidate for an optional, non-core `fidius-connector` crate (still outside core).
- The WASM Component Model `stream<T>`/`future<T>` async ABI stabilizes in wasmtime, changing the cost calculus of the primitive's WASM implementation.
- Demand for supported composition/orchestration becomes loud enough to reconsider the test-tier decision for a *minimal* `connect()` — re-evaluate against the mechanism/policy line before conceding.

### Scheduled Review
- **Next Review Date**: After FIDIUS-I-0026 ships its first backend and one external adopter has built a connector protocol on top.
- **Review Criteria**: Did the boundary hold? Did adopters need anything from core we refused? Did the unsupported harness leak harmfully?
- **Sunset Date**: N/A — boundary decision is intended to be durable; the *primitive's* shape may evolve under FIDIUS-I-0026 without reopening this boundary.