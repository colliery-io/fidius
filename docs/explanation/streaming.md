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

# Server-Streaming

A normal plugin method is request/response: one call, one return value. Some
plugins instead produce a *sequence* — a connector paginating an API, a query
that returns rows, a tail of a log. Buffering all of it into one `Vec` return is
wasteful (unbounded memory) and high-latency (nothing is usable until everything
is ready). **Server-streaming** lets a method yield items one at a time, with the
consumer pulling them lazily.

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

`Stream<T>` in argument position is rejected — streaming is a property of the
*return*, not the input. (Streaming *input* would be a different feature; today a
plugin that consumes a sequence takes a `Vec<T>` — "chunked unary".)

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
