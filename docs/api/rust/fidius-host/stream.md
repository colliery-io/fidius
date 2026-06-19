# fidius-host::stream <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Server-streaming dispatch (FIDIUS-I-0026).

The unary seam ([`crate::executor`]) carries one value in and one value out.
This module adds the *streaming* seam: one call in, an unbounded **pull**
handle of values out.

## Structs

### `fidius-host::stream::ChunkStream`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Host-facing pull handle for a server-streaming plugin call.

A `futures::Stream` of decoded items. Pull with `.next().await`; the stream
ends after the producer's `END` frame, or yields one final `Err` and then
ends on an `ERROR`/abort/malformed frame. Dropping the handle cancels the
call — the backend bridge observes the drop and tears its producer down.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `inner` | `Pin < Box < dyn Stream < Item = Result < Value , CallError > > + Send > >` |  |

#### Methods

##### `new` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn new < S > (stream : S) -> Self where S : Stream < Item = Result < Value , CallError > > + Send + 'static ,
```

Wrap any item stream as a [`ChunkStream`]. Backends that already produce `Result<Value, CallError>` items use this directly.

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



##### `from_frame_bytes` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn from_frame_bytes < S , D > (frames : S , decode_item : D) -> Self where S : Stream < Item = Vec < u8 > > + Send + 'static , D : Fn (& [u8]) -> Result < Value , CallError > + Send + 'static ,
```

Build a [`ChunkStream`] from a stream of raw, length-delimited frame *bytes* (one encoded [`Frame`] per element — the shape a serialized backend's `next()` hands back: a WASM `list<u8>`, a cdylib buffer), applying the terminal-frame state machine:

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
self-describing [`Value`] — `deserialize_any` is unsupported — so there is
deliberately no fixed "bytes → Value" decode here.) The self-describing
in-process backends (Python) skip framing entirely and use [`Self::new`].
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



##### `from_frames` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn from_frames < D > (frames : Vec < Frame > , decode_item : D) -> Self where D : Fn (& [u8]) -> Result < Value , CallError > + Send + 'static ,
```

Build a [`ChunkStream`] over a fixed, in-memory sequence of [`Frame`]s. A convenience for tests and serialized-backend fixtures; runs the same terminal-frame state machine as [`Self::from_frame_bytes`] with the same caller-supplied `decode_item`.

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





