# fidius-host::executor::body_tee <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Tee/replay bodies for the egress response hook (FIDIUS-I-0035).

To honor [`ResponseDirective::RetryOnce`](super::wasm::ResponseDirective),
the dispatch path must be able to send the guest's request body a second
time — but an outgoing body streams to the wire once and is gone. The
[`TeeBody`] wrapper solves this without changing wire behavior: frames pass
through to the inner body untouched while their bytes are copied into a
side buffer, capped at [`REPLAY_CAP`]. Whether a retry is possible is
decided *at response time* from the capture's terminal state (a bodiless
GET has no `Content-Length`, so a pre-dispatch size check would exclude the
most common retryable requests).

## Structs

### `fidius-host::executor::body_tee::CaptureInner`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


#### Fields

| Name | Type | Description |
|------|------|-------------|
| `buf` | `BytesMut` |  |
| `state` | `CaptureState` |  |



### `fidius-host::executor::body_tee::CaptureHandle`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">pub(crate)</span>


**Derives:** `Clone`

Shared handle onto a [`TeeBody`]'s capture, held by the dispatch task.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `0` | `Arc < Mutex < CaptureInner > >` |  |

#### Methods

##### `state` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">pub(crate)</span>


```rust
fn state (& self) -> CaptureState
```

The capture's current terminal state (test observability; the dispatch path decides off [`replayable`](Self::replayable) alone).

<details>
<summary>Source</summary>

```rust
    pub(crate) fn state(&self) -> CaptureState {
        self.0.lock().expect("tee capture lock poisoned").state
    }
```

</details>



##### `replayable` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">pub(crate)</span>


```rust
fn replayable (& self) -> Option < Bytes >
```

The bytes to replay, if and only if the body was fully captured under the cap.

<details>
<summary>Source</summary>

```rust
    pub(crate) fn replayable(&self) -> Option<Bytes> {
        let inner = self.0.lock().expect("tee capture lock poisoned");
        (inner.state == CaptureState::Complete).then(|| inner.buf.clone().freeze())
    }
```

</details>





### `fidius-host::executor::body_tee::TeeBody`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">pub(crate)</span>


A pass-through wrapper over a guest's outgoing body that captures its bytes for a possible single replay. Every frame the inner body yields is delivered unchanged; the one deliberate wire difference is that a body whose end was observed at wrap time advertises `is_end_stream()`, so e.g. a bodiless GET goes out with no framing instead of an empty chunked body.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `inner` | `HyperOutgoingBody` |  |
| `capture` | `Arc < Mutex < CaptureInner > >` |  |
| `stash` | `std :: collections :: VecDeque < Result < Frame < Bytes > , ErrorCode > >` | Frames pulled out of `inner` by [`prime`](Self::prime), replayed to
the real consumer before `inner` is polled again. |
| `ended` | `bool` | `inner` returned end-of-stream during priming; it must not be polled
again. |

#### Methods

##### `wrap` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">pub(crate)</span>


```rust
fn wrap (inner : HyperOutgoingBody) -> (HyperOutgoingBody , CaptureHandle)
```

Wrap `inner`, returning the teed body (boxed back to the [`HyperOutgoingBody`] shape dispatch expects) and the capture handle.

<details>
<summary>Source</summary>

```rust
    pub(crate) fn wrap(inner: HyperOutgoingBody) -> (HyperOutgoingBody, CaptureHandle) {
        let capture = Arc::new(Mutex::new(CaptureInner {
            buf: BytesMut::new(),
            state: CaptureState::Incomplete,
        }));
        let handle = CaptureHandle(Arc::clone(&capture));
        let mut tee = TeeBody {
            inner,
            capture,
            stash: std::collections::VecDeque::new(),
            ended: false,
        };
        tee.prime();
        (tee.boxed_unsync(), handle)
    }
```

</details>



##### `prime` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn prime (& mut self)
```

Drain whatever the inner body can yield *right now* (noop waker, no task context) into the stash, recording it in the capture.

This is what makes the common case deterministic: a wasi-http guest
body is a channel (`is_end_stream()` is never true up front, end only
observable by polling), and hyper's conn task may not have polled it
to completion by the time a fast 401 arrives — the capture would still
be `Incomplete` at decision time and the retry silently unavailable.
A guest that finished its body before dispatch (bodiless GETs, small
JSON POSTs — the motivating weir shape) has every frame already in
the channel, so priming reaches `Complete` here, synchronously.
A still-streaming body returns `Pending` immediately (registering the
noop waker — harmless: channel bodies re-register on every poll, and
hyper's dispatcher always polls the body at least once without
needing a wakeup first) and replay eligibility stays timing-dependent,
as documented.

<details>
<summary>Source</summary>

```rust
    fn prime(&mut self) {
        let mut cx = Context::from_waker(std::task::Waker::noop());
        while !self.ended {
            match Pin::new(&mut self.inner).poll_frame(&mut cx) {
                Poll::Pending => break,
                Poll::Ready(None) => {
                    self.record_end();
                    self.ended = true;
                }
                Poll::Ready(Some(item)) => {
                    let broke = item.is_err();
                    if let Ok(frame) = &item {
                        self.record_frame(frame);
                    }
                    self.stash.push_back(item);
                    if broke {
                        // The body errored; hand the error through and stop.
                        break;
                    }
                }
            }
        }
    }
```

</details>



##### `record_frame` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn record_frame (& self , frame : & Frame < Bytes >)
```

<details>
<summary>Source</summary>

```rust
    fn record_frame(&self, frame: &Frame<Bytes>) {
        let mut inner = self.capture.lock().expect("tee capture lock poisoned");
        if let Some(data) = frame.data_ref() {
            // Once the capture is dead (overflow/trailers) stop copying; the
            // body still streams to the wire.
            if inner.state == CaptureState::Incomplete {
                if inner.buf.len() + data.len() > REPLAY_CAP {
                    inner.state = CaptureState::Overflowed;
                    inner.buf = BytesMut::new();
                } else {
                    inner.buf.extend_from_slice(data);
                }
            }
        } else if frame.trailers_ref().is_some() && inner.state == CaptureState::Incomplete {
            inner.state = CaptureState::Trailers;
            inner.buf = BytesMut::new();
        }
    }
```

</details>



##### `record_end` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn record_end (& self)
```

<details>
<summary>Source</summary>

```rust
    fn record_end(&self) {
        // Clean end-of-stream: whatever was captured is the whole body.
        let mut inner = self.capture.lock().expect("tee capture lock poisoned");
        if inner.state == CaptureState::Incomplete {
            inner.state = CaptureState::Complete;
        }
    }
```

</details>





## Enums

### `fidius-host::executor::body_tee::CaptureState` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">pub(crate)</span>


Terminal state of a tee capture, read at response time to decide whether a retry may replay the body.

#### Variants

- **`Incomplete`** - End-of-stream not (cleanly) observed yet — the request may still be
streaming, or the body errored mid-flight. Not replayable.
- **`Complete`** - The whole body was observed and fit under [`REPLAY_CAP`]. Replayable.
- **`Overflowed`** - The body exceeded [`REPLAY_CAP`]; capture was abandoned. Not replayable.
- **`Trailers`** - The body carried trailers, which a replay would not reproduce. Not
replayable.



## Functions

### `fidius-host::executor::body_tee::replay_body`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">pub(crate)</span>


```rust
fn replay_body (bytes : Bytes) -> HyperOutgoingBody
```

A replayable body carrying previously captured bytes, for the retry dispatch.

<details>
<summary>Source</summary>

```rust
pub(crate) fn replay_body(bytes: Bytes) -> HyperOutgoingBody {
    http_body_util::Full::new(bytes)
        .map_err(|never| match never {})
        .boxed_unsync()
}
```

</details>



