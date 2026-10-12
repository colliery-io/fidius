---
title: "fidius-host::stream"
description: "Server-streaming dispatch (FIDIUS-I-0026)."
editUrl: false
sidebar:
  order: 43
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Server-streaming dispatch (FIDIUS-I-0026).

The unary seam ([`crate::executor`](../../../rust/fidius-host/executor/)) carries one value in and one value out.
This module adds the *streaming* seam: one call in, an unbounded **pull**
handle of values out.

**The pieces (design decisions D1–D3)**

- [`StreamExecutor`](../../../rust/fidius-host/stream/#trait.StreamExecutor) — the backend-side trait. `async fn call_streaming`
  starts a server-streaming method and returns a [`ChunkStream`](../../../rust/fidius-host/stream/#struct.ChunkStream). Implemented
  by all three backends (Python and WASM via self-describing values; cdylib
  via its FFI iterator-handle ABI).
- [`ChunkStream`](../../../rust/fidius-host/stream/#struct.ChunkStream) — the host-side handle: a `futures::Stream` of
  `Result<Value, CallError>`. **Native async** (D1) — every consuming host is
  a tokio app, so the handle is awaited, not polled on a blocked thread.
- **Backpressure & cancel are structural.** The host pulls with `.next().await`;
  a backend bridge that fills a bounded channel blocks when the host stops
  pulling (REQ-003). Dropping the `ChunkStream` drops the bridge, which tears
  the producer down (D3 / REQ-002).

**Wire**

On the wire a stream is a sequence of `Frame`s (see [`fidius_core::frame`](../../../rust/fidius-guest/frame/)):
zero or more `ITEM` frames terminated by exactly one `END` or `ERROR`.
[`ChunkStream::from_frame_bytes`](../../../rust/fidius-host/stream/#ChunkStream.from_frame_bytes) runs the terminal-frame state machine that
turns that byte sequence into the item stream every backend bridge feeds.

## Structs

### <span id="struct.ChunkStream"></span><code style="font-size: 0.85em">struct ChunkStream</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

Host-facing pull handle for a server-streaming plugin call.

A `futures::Stream` of decoded items. Pull with `.next().await`; the stream
ends after the producer's `END` frame, or yields one final `Err` and then
ends on an `ERROR`/abort/malformed frame. Dropping the handle cancels the
call — the backend bridge observes the drop and tears its producer down.

#### Methods

##### <span id="ChunkStream.new"></span><code>new</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn new<S>(stream: S) -> Self
where
    S: Stream<Item = Result<Value, CallError>> + Send + 'static,
```

Wrap any item stream as a [`ChunkStream`](../../../rust/fidius-host/stream/#struct.ChunkStream). Backends that already produce `Result<Value, CallError>` items use this directly.

<details>
<summary>Source</summary>

```rust
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Value, CallError>> + Send + 'static,
    {
        Self {
            inner: Box::pin(stream),
        }
    }
```

</details>

##### <span id="ChunkStream.from_frame_bytes"></span><code>from_​frame_​bytes</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn from_frame_bytes<S, D>(frames: S, decode_item: D) -> Self
where
    S: Stream<Item = Vec<u8>> + Send + 'static,
    D: Fn(&[u8]) -> Result<Value, CallError> + Send + 'static,
```

Build a [`ChunkStream`](../../../rust/fidius-host/stream/#struct.ChunkStream) from a stream of raw, length-delimited frame *bytes* (one encoded `Frame` per element — the shape a serialized backend's `next()` hands back: a WASM `list<u8>`, a cdylib buffer), applying the terminal-frame state machine:

- `ITEM` → `decode_item(payload)` → `Ok(value)`, continue.
- `END` → stop (no item).
- `ERROR` → one `Err(CallError::Plugin)`, then stop.
- a malformed/truncated frame → one `Err(CallError::MalformedFrame)`, stop.
- an `ITEM` whose payload fails `decode_item` → that `Err`, then stop.
- the byte source ending **without** a terminal frame → one
  `Err(CallError::StreamAborted)`, stop.

`decode_item` is caller-supplied because the ITEM payload is "one unary
return value": for cdylib that is concrete-type bincode the typed client
decodes via `wire::deserialize::<T>()` then `to_value`; for a `#[wire(raw)]`
stream it is the bytes themselves. (Vanilla bincode cannot reconstruct a
self-describing `Value` — `deserialize_any` is unsupported — so there is
deliberately no fixed "bytes → Value" decode here.) The self-describing
in-process backends (Python) skip framing entirely and use `Self::new`.

After any error the stream is fused: subsequent polls yield `None`.

<details>
<summary>Source</summary>

```rust
    pub fn from_frame_bytes<S, D>(frames: S, decode_item: D) -> Self
    where
        S: Stream<Item = Vec<u8>> + Send + 'static,
        D: Fn(&[u8]) -> Result<Value, CallError> + Send + 'static,
    {
        let stream = futures::stream::unfold(
            (frames.boxed(), decode_item, false),
            |(mut src, decode_item, done)| async move {
                if done {
                    return None;
                }
                match src.next().await {
                    // Source dried up before a terminal frame: the producer vanished.
                    None => Some((Err(CallError::StreamAborted), (src, decode_item, true))),
                    Some(bytes) => match Frame::decode(&bytes) {
                        Err(e) => Some((
                            Err(CallError::MalformedFrame(e.to_string())),
                            (src, decode_item, true),
                        )),
                        Ok(Frame::Item(payload)) => match decode_item(&payload) {
                            Ok(v) => Some((Ok(v), (src, decode_item, false))),
                            Err(e) => Some((Err(e), (src, decode_item, true))),
                        },
                        Ok(Frame::End) => None,
                        Ok(Frame::Error(pe)) => {
                            Some((Err(CallError::Plugin(pe)), (src, decode_item, true)))
                        }
                    },
                }
            },
        );
        Self::new(stream)
    }
```

</details>

##### <span id="ChunkStream.from_frames"></span><code>from_​frames</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn from_frames<D>(frames: Vec<Frame>, decode_item: D) -> Self
where
    D: Fn(&[u8]) -> Result<Value, CallError> + Send + 'static,
```

Build a [`ChunkStream`](../../../rust/fidius-host/stream/#struct.ChunkStream) over a fixed, in-memory sequence of `Frame`s. A convenience for tests and serialized-backend fixtures; runs the same terminal-frame state machine as `Self::from_frame_bytes` with the same caller-supplied `decode_item`.

<details>
<summary>Source</summary>

```rust
    pub fn from_frames<D>(frames: Vec<Frame>, decode_item: D) -> Self
    where
        D: Fn(&[u8]) -> Result<Value, CallError> + Send + 'static,
    {
        let bytes: Vec<Vec<u8>> = frames
            .iter()
            .map(|f| f.encode().expect("frame encodes"))
            .collect();
        Self::from_frame_bytes(futures::stream::iter(bytes), decode_item)
    }
```

</details>

## Traits

### <span id="trait.StreamExecutor"></span><code style="font-size: 0.85em">trait StreamExecutor</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

**Supertraits:** `PluginExecutor`

Backends whose typed boundary can produce a **server-streaming** result.

Sits beside [`crate::executor::ValueExecutor`](../../../rust/fidius-host/executor/#trait.ValueExecutor): that returns one `Value`,
this returns a [`ChunkStream`](../../../rust/fidius-host/stream/#struct.ChunkStream) of them. Implemented by all three backends
(Python and WASM via self-describing values; cdylib via its FFI
iterator-handle ABI). `PluginHandle` routes a streaming method to
`call_streaming` exactly as it routes a unary one to `call`.

#### Methods

##### <span id="StreamExecutor.call_streaming"></span><code>call_​streaming</code>

<span class="plissken-badge plissken-badge-async" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-blue-low); color: var(--sl-color-blue-high);">async</span>

```rust
async fn call_streaming(
    &self,
    method: usize,
    args: Value,
) -> Result<ChunkStream, CallError>
```

Start a server-streaming call by method index. `args` crosses as a self-describing `Value` (same as the unary typed path); the result is a pull handle over the produced items.

<details>
<summary>Source</summary>

```rust
    async fn call_streaming(&self, method: usize, args: Value) -> Result<ChunkStream, CallError>;
```

</details>
