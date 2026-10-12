---
title: "fidius-test::stream"
description: "Composition harness for streaming plugins — **for tests, not production**."
editUrl: false
sidebar:
  order: 54
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Composition harness for streaming plugins — **for tests, not production**.

`stream_of` / `collect` / `pump` make it trivial to test a streaming plugin
in isolation or to wire a producer to a consumer. They are the reference `|`
for "pipes of plugins."

**Stability**

**This module is deliberately NOT part of fidius's semver-stable surface.**
It exists so tests can compose streams without re-deriving the same loop, and
so the canonical composition pattern is readable in one place. Its behaviour
is *correct* (pull-paced backpressure, stop-and-surface on the first error)
but its API may change at any time. **In production, write your own pump
loop** — fidius ships the typed pipe (the `ChunkStream` primitive), not an
orchestrator; orchestration (scheduling, retries, observability) is yours.
See ADR FIDIUS-A-0004 ("Streaming as Mechanism, Not Protocol").

## Structs

### <span id="struct.CollectSink"></span><code style="font-size: 0.85em">struct CollectSink</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

**Derives:** `Default`

A [`StreamSink`](../../../rust/fidius-test/stream/#trait.StreamSink) that records everything it accepts — for asserting on the far end of a `pump`.

#### Methods

##### <span id="CollectSink.new"></span><code>new</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn new() -> Self
```

A fresh, empty sink.

<details>
<summary>Source</summary>

```rust
    pub fn new() -> Self {
        Self::default()
    }
```

</details>

##### <span id="CollectSink.take"></span><code>take</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn take(&self) -> Vec<Value>
```

Snapshot of everything accepted so far.

<details>
<summary>Source</summary>

```rust
    pub fn take(&self) -> Vec<Value> {
        self.items.lock().unwrap().clone()
    }
```

</details>

## Traits

### <span id="trait.StreamSink"></span><code style="font-size: 0.85em">trait StreamSink</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

The destination side of a pipe: a consumer `pump` hands each item to, in pull order. Deliberately tiny — a real destination plugin presents whatever richer surface it likes; this is only what composition tests need.

#### Methods

##### <span id="StreamSink.accept"></span><code>accept</code>

<span class="plissken-badge plissken-badge-async" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-blue-low); color: var(--sl-color-blue-high);">async</span>

```rust
async fn accept(&self, item: Value) -> Result<(), CallError>
```

Accept one streamed item. Returning `Err` stops the pump and surfaces.

<details>
<summary>Source</summary>

```rust
    async fn accept(&self, item: Value) -> Result<(), CallError>;
```

</details>

## Functions

### <span id="fn.stream_of"></span><code style="font-size: 0.85em">fn stream_​of</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn stream_of(items: Vec<Value>) -> ChunkStream
```

An in-memory source over a fixed item sequence. **Test-only by nature** — you would never construct a stream from a `Vec` in production; that it is useful only in tests is exactly why the whole module is test-tier.

Yields each value as `Ok`, then a clean end of stream.

<details>
<summary>Source</summary>

```rust
pub fn stream_of(items: Vec<Value>) -> ChunkStream {
    ChunkStream::new(futures::stream::iter(
        items.into_iter().map(Ok::<Value, CallError>),
    ))
}
```

</details>

### <span id="fn.collect"></span><code style="font-size: 0.85em">fn collect</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
async fn collect(mut s: ChunkStream) -> Result<Vec<Value>, CallError>
```

Drain a stream to a `Vec`, stopping at — and returning — the first error. The single-plugin unit-test idiom: `collect(plugin.process(stream_of(rows)))`.

<details>
<summary>Source</summary>

```rust
pub async fn collect(mut s: ChunkStream) -> Result<Vec<Value>, CallError> {
    let mut out = Vec::new();
    while let Some(item) = s.next().await {
        out.push(item?);
    }
    Ok(out)
}
```

</details>

### <span id="fn.pump"></span><code style="font-size: 0.85em">fn pump</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
async fn pump<S>(mut out: ChunkStream, into: &S) -> Result<(), CallError>
where
    S: StreamSink + ?Sized,
```

The reference pull-loop wiring a producer stream to a [`StreamSink`](../../../rust/fidius-test/stream/#trait.StreamSink).

Pull-paced: exactly one item is awaited at a time, so a slow sink naturally
backpressures the producer (the next item is not pulled until the sink has
accepted the current one). Stops at the first error from either side —
producer error or sink rejection — and returns it; on a clean end of stream
returns `Ok(())`. This is the ~10 lines you would copy into production and
then grow your own retries/observability around.

<details>
<summary>Source</summary>

```rust
pub async fn pump<S>(mut out: ChunkStream, into: &S) -> Result<(), CallError>
where
    S: StreamSink + ?Sized,
{
    while let Some(item) = out.next().await {
        into.accept(item?).await?;
    }
    Ok(())
}
```

</details>
