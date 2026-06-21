<!--
Copyright 2026 Colliery, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
-->

# Streaming

A normal plugin method is request/response: one call, one return value. Some plugins
instead work with *sequences*. fidius streams in **three directions**, all pull-based,
backpressured, and drop-to-cancel, on cdylib, WASM, and Python:

- **Server-streaming** — the plugin produces (`Stream<T>` return).
- **Client-streaming** — the host produces (`Stream<T>` argument).
- **Bidirectional** — both (`Stream` in *and* out): a plugin-owned transform.

## Server-streaming

A connector paginating an API, a query returning rows, a tail of a log: buffering all
of it into one `Vec` return is wasteful (unbounded memory) and high-latency (nothing is
usable until everything is ready). **Server-streaming** lets a method yield items one at
a time, with the consumer pulling them lazily.

## Declaring a streaming method

A method is server-streaming when its return type is the marker `fidius::Stream<T>`:

```rust
#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Source: Send + Sync {
    /// Yields `count` records, one at a time.
    fn read(&self, count: u32) -> fidius::Stream<Record>;
}
```

The implementation returns a `Stream<T>` built from any iterator:

```rust
#[fidius::plugin_impl(Source)]
impl Source for MySource {
    fn read(&self, count: u32) -> fidius::Stream<Record> {
        fidius::Stream::from_iter((0..count).map(|i| Record { id: i }))
    }
}
```

## Client-streaming: `Stream<T>` in argument position

The dual also works (FIDIUS-I-0030 / ADR-0007): a `Stream<T>` **argument** is a
*client-streaming* method — the **host produces** the items and the plugin **pulls**
them, so a writer/sink can control its own consumption (batch, look-ahead,
backpressure) rather than the host pushing one call per item.

```rust
#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Sink: Send + Sync {
    fn load(&self, rows: fidius::Stream<Row>) -> Summary; // host produces `rows`
}
```

The host feeds it with `PluginHandle::call_client_streaming(method, items, &args)`.
It works on **all three backends**: cdylib (a host callback the plugin pulls), WASM
(the `fidius:stream-pull` import, composed like wasi:http), and Python (a host-fed
iterator the method receives). Drop = cancel; backpressure inverts (the plugin's
consumption rate parks the host).

> **Python convention — the stream argument comes first.** In a Python plugin, the
> host-fed stream iterator is passed as the method's **first positional argument**; any
> non-stream args follow. So write `def load(rows, mode):`, not `def load(mode, rows):`.
> (Rust cdylib/WASM honor the stream's declared position anywhere in the signature; only
> Python pins it first. It's a positional contract — the same shape duck-typed Python
> already relies on.) This applies to client-streaming and bidirectional methods.

## Bidirectional: `Stream<T>` in *both* arg and return

A method can take `Stream<In>` **and** return `Stream<Out>` (FIDIUS-I-0032 / ADR-0010) —
a streaming **transform** the plugin owns end to end (parse → enrich → re-emit, windowed
aggregation, filter):

```rust
#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Transformer: Send + Sync {
    fn transform(&self, input: fidius::Stream<Row>) -> fidius::Stream<Out>;
}
```

It is built as the **synchronous lazy-pull composition** of the two halves above — *not*
a second concurrent pump. The returned `Stream<Out>` is lazy; when the host pulls an
`Out`, the plugin's iterator pulls `In` **on demand**, re-entering the host producer on
the same call stack:

> host → `output.next()` → plugin → `input.next()` → host yields `In` → plugin computes →
> returns `Out`

So there are no threads, no channels, and **no deadlock** — the input and output rates are
coupled through the plugin's own iterator (a plugin that needs uneven rates buffers
internally). Drop tears down the whole chain; backpressure is the host's output pull rate.

The host drives it with `PluginHandle::call_bidi_streaming::<In, Args, Out>(method, items,
&args).await`, which returns a `ChunkStream` of `Out`. All three backends compose their
existing client- (input) and server- (output) streaming machinery: cdylib (producer handle
in → output stream handle out), WASM (the `fidius:stream-pull` import **and** an exported
output resource), and Python (a host-fed iterator argument **and** a returned generator).
The guest writes a plain lazy adapter — e.g. `Stream::from_iter(std::iter::from_fn(move ||
input.next_item().map(transform)))` in Rust, or `for r in rows: yield f(r)` in Python.

> **Stream item types.** An **input** item (client-streaming, and the input side of
> bidirectional) crosses as bincode, so it can be any `Serialize`/`Deserialize` type —
> primitives, `String`, or a record — with no `#[derive(WitType)]` needed (FIDIUS-T-0171).
> An **output** item (server-streaming, and the output side of bidirectional) crosses via a
> WIT resource, so a record output needs `#[derive(WitType)]` like any other WIT-typed
> return (FIDIUS-T-0175). A record may be both an input stream item and a WIT-typed
> arg/return (derive both `WitType` and `Serialize`/`Deserialize`).
>
> **Memory & concurrency.** The host producer is **lazy on all three backends** — each
> input item is encoded/converted only as the plugin pulls it, so an *unbounded* input
> streams with bounded memory (FIDIUS-T-0172 for cdylib/WASM, FIDIUS-T-0174 for Python). A
> truly concurrent two-pump (independent in/out rates without internal buffering) was
> deliberately rejected — see ADR-0010.

## The host side: `ChunkStream`

On the host, a streaming call returns a `ChunkStream` — a
`futures::Stream` of `Result<Value, CallError>` you pull with `.next().await`:

```rust
use futures::StreamExt;

let mut stream = handle.call_streaming::<_, Record>(READ, &(1000u32,)).await?;
while let Some(item) = stream.next().await {
    let record: Record = fidius_core::from_value(item?)?;
    // … process one record …
}
```

Two properties are **structural**, not bolted on:

- **Backpressure.** Items are pulled one at a time through a small bounded
  channel. A slow consumer naturally parks the producer — the next item isn't
  produced until the current one is taken. Memory stays bounded regardless of how
  many items the source *could* produce (a source of ten million items that you
  pull three from and drop costs almost nothing).
- **Cancellation = drop.** Dropping the `ChunkStream` tears the producer down: the
  Python generator is closed, the WASM stream resource's destructor runs, the
  cdylib iterator is dropped. There is no separate "cancel" call to forget.

## Mechanism, not policy

fidius ships **the typed pipe, not the connector runtime** (ADR
*Streaming as Mechanism, Not Protocol*). `ChunkStream` gives you a backpressured,
cancellable sequence of typed values — and stops there. Scheduling, retries,
checkpointing, observability, fan-out, "reverse-ETL" semantics: those are
*your* orchestration, layered on top. fidius deliberately does not grow a
connector protocol, because the right protocol is the adopter's, not the
framework's.

### The composition harness is test-tier

`fidius-test` ships a tiny reference for wiring a producer stream to a consumer —
`stream_of` / `collect` / `pump` / `StreamSink` — the "bash pipe for plugins":

```rust
// reader plugin ──stream──> a sink (here, another plugin's writer)
let stream = reader.call_streaming::<_, Value>(READ, &cfg).await?;
fidius_test::pump(stream, &writer_sink).await?;
```

This is **explicitly not** part of the semver-stable surface — it exists so tests
can compose streams in one readable place. In production you copy the ~10-line
`pump` loop and grow your own retries/observability around it.

## How it works across the three backends

The same `fidius::Stream<T>` contract is implemented natively on each backend; the
host drives all of them through the identical `ChunkStream`:

| Backend | Producer | Cancellation |
| ------- | -------- | ------------ |
| **Python** | a generator, driven on a dedicated GIL thread | `gen.close()` on drop |
| **WASM** | a Component-Model `resource` with a `next()` method, pumped on a thread | resource drop → guest destructor |
| **cdylib** | an iterator-handle FFI ABI (`init`/`next`/`drop`), pumped on a thread | `drop_fn` on drop |

In every case a dedicated pump thread pulls from the synchronous producer and
hands items across a bounded `tokio::mpsc` channel to the async `ChunkStream` — so
a blocking producer never blocks the async consumer, and backpressure flows the
other way.

Because the contract is the WASM **WIT resource**, streaming is *language-neutral*:
the same `ticker` streaming interface is implemented in Rust, JavaScript, Python,
and C guests, and the host streams from all of them with identical code.

### Item encoding

Python and WASM cross items as the self-describing `fidius_core::Value`
currency (the same as unary calls). cdylib crosses items as concrete `bincode`
of the item type and decodes them host-side — byte-identical to the unary cdylib
wire. You don't see this: `ChunkStream` yields `Value` either way.

## What you do *not* get

The same "no built-in timeout / watchdog" caveat as unary calls applies: a
producer that hangs mid-stream hangs the consumer's `.next().await`. Dropping the
stream cancels cooperative producers, but fidius cannot interrupt a truly stuck
one — that needs process isolation, which is the adopter's call. See the `fidius`
crate top-level docs for the rationale.

## See also

- [Capabilities & the WASM Sandbox](wasm-capabilities.md) — a streaming REST
  source combines `read() -> Stream<Record>` with the `http` egress capability.
- [Buffer Strategies](buffer-strategies.md) — the unary wire the cdylib stream
  reuses per item.
