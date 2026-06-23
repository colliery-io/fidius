// Copyright 2026 Colliery, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Server-streaming dispatch (FIDIUS-I-0026).
//!
//! The unary seam ([`crate::executor`]) carries one value in and one value out.
//! This module adds the *streaming* seam: one call in, an unbounded **pull**
//! handle of values out.
//!
//! ## The pieces (design decisions D1–D3)
//!
//! - [`StreamExecutor`] — the backend-side trait. `async fn call_streaming`
//!   starts a server-streaming method and returns a [`ChunkStream`]. Implemented
//!   by all three backends (Python and WASM via self-describing values; cdylib
//!   via its FFI iterator-handle ABI).
//! - [`ChunkStream`] — the host-side handle: a `futures::Stream` of
//!   `Result<Value, CallError>`. **Native async** (D1) — every consuming host is
//!   a tokio app, so the handle is awaited, not polled on a blocked thread.
//! - **Backpressure & cancel are structural.** The host pulls with `.next().await`;
//!   a backend bridge that fills a bounded channel blocks when the host stops
//!   pulling (REQ-003). Dropping the `ChunkStream` drops the bridge, which tears
//!   the producer down (D3 / REQ-002).
//!
//! ## Wire
//!
//! On the wire a stream is a sequence of [`Frame`]s (see [`fidius_core::frame`]):
//! zero or more `ITEM` frames terminated by exactly one `END` or `ERROR`.
//! [`ChunkStream::from_frame_bytes`] runs the terminal-frame state machine that
//! turns that byte sequence into the item stream every backend bridge feeds.

use std::pin::Pin;
use std::task::{Context, Poll};

use fidius_core::frame::Frame;
use fidius_core::Value;
use futures::stream::{Stream, StreamExt};

use crate::error::CallError;
use crate::executor::PluginExecutor;

/// Host-facing pull handle for a server-streaming plugin call.
///
/// A `futures::Stream` of decoded items. Pull with `.next().await`; the stream
/// ends after the producer's `END` frame, or yields one final `Err` and then
/// ends on an `ERROR`/abort/malformed frame. Dropping the handle cancels the
/// call — the backend bridge observes the drop and tears its producer down.
pub struct ChunkStream {
    inner: Pin<Box<dyn Stream<Item = Result<Value, CallError>> + Send>>,
}

impl ChunkStream {
    /// Wrap any item stream as a [`ChunkStream`]. Backends that already produce
    /// `Result<Value, CallError>` items use this directly.
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Value, CallError>> + Send + 'static,
    {
        Self {
            inner: Box::pin(stream),
        }
    }

    /// Build a [`ChunkStream`] from a stream of raw, length-delimited frame
    /// *bytes* (one encoded [`Frame`] per element — the shape a serialized
    /// backend's `next()` hands back: a WASM `list<u8>`, a cdylib buffer),
    /// applying the terminal-frame state machine:
    ///
    /// - `ITEM` → `decode_item(payload)` → `Ok(value)`, continue.
    /// - `END` → stop (no item).
    /// - `ERROR` → one `Err(CallError::Plugin)`, then stop.
    /// - a malformed/truncated frame → one `Err(CallError::MalformedFrame)`, stop.
    /// - an `ITEM` whose payload fails `decode_item` → that `Err`, then stop.
    /// - the byte source ending **without** a terminal frame → one
    ///   `Err(CallError::StreamAborted)`, stop.
    ///
    /// `decode_item` is caller-supplied because the ITEM payload is "one unary
    /// return value": for cdylib that is concrete-type bincode the typed client
    /// decodes via `wire::deserialize::<T>()` then `to_value`; for a `#[wire(raw)]`
    /// stream it is the bytes themselves. (Vanilla bincode cannot reconstruct a
    /// self-describing [`Value`] — `deserialize_any` is unsupported — so there is
    /// deliberately no fixed "bytes → Value" decode here.) The self-describing
    /// in-process backends (Python) skip framing entirely and use [`Self::new`].
    ///
    /// After any error the stream is fused: subsequent polls yield `None`.
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

    /// Build a [`ChunkStream`] over a fixed, in-memory sequence of [`Frame`]s.
    /// A convenience for tests and serialized-backend fixtures; runs the same
    /// terminal-frame state machine as [`Self::from_frame_bytes`] with the same
    /// caller-supplied `decode_item`.
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
}

impl Stream for ChunkStream {
    type Item = Result<Value, CallError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

/// Backends whose typed boundary can produce a **server-streaming** result.
///
/// Sits beside [`crate::executor::ValueExecutor`]: that returns one [`Value`],
/// this returns a [`ChunkStream`] of them. Implemented by all three backends
/// (Python and WASM via self-describing values; cdylib via its FFI
/// iterator-handle ABI). `PluginHandle` routes a streaming method to
/// `call_streaming` exactly as it routes a unary one to `call`.
#[async_trait::async_trait]
pub trait StreamExecutor: PluginExecutor {
    /// Start a server-streaming call by method index. `args` crosses as a
    /// self-describing [`Value`] (same as the unary typed path); the result is
    /// a pull handle over the produced items.
    async fn call_streaming(&self, method: usize, args: Value) -> Result<ChunkStream, CallError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use fidius_core::error::PluginError;
    use fidius_core::to_value;

    /// An ITEM frame carrying a concrete `i64` (bincode of a concrete type
    /// round-trips fine — unlike a self-describing `Value`).
    fn item(v: i64) -> Frame {
        Frame::Item(fidius_core::wire::serialize(&v).unwrap())
    }

    /// The matching item decoder: concrete-bincode `i64` → `Value`.
    fn decode_i64(b: &[u8]) -> Result<Value, CallError> {
        fidius_core::wire::deserialize::<i64>(b)
            .map(|n| to_value(&n).unwrap())
            .map_err(|e| CallError::Deserialization(e.to_string()))
    }

    async fn collect(mut s: ChunkStream) -> Vec<Result<Value, CallError>> {
        let mut out = Vec::new();
        while let Some(x) = s.next().await {
            out.push(x);
        }
        out
    }

    #[tokio::test]
    async fn items_then_clean_end() {
        let s = ChunkStream::from_frames(vec![item(1), item(2), item(3), Frame::End], decode_i64);
        let vals: Vec<i64> = collect(s)
            .await
            .into_iter()
            .map(|r| fidius_core::from_value(r.unwrap()).unwrap())
            .collect();
        assert_eq!(vals, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn native_value_stream_via_new() {
        // The Value-rail (Python) path: items are produced as Values natively,
        // no framing involved.
        let items = vec![Ok(to_value(&"a").unwrap()), Ok(to_value(&"b").unwrap())];
        let s = ChunkStream::new(futures::stream::iter(items));
        let got: Vec<String> = collect(s)
            .await
            .into_iter()
            .map(|r| fidius_core::from_value(r.unwrap()).unwrap())
            .collect();
        assert_eq!(got, vec!["a".to_string(), "b".to_string()]);
    }

    #[tokio::test]
    async fn error_frame_terminates_after_one_err() {
        let s = ChunkStream::from_frames(
            vec![
                item(1),
                Frame::Error(PluginError::new("BOOM", "broke")),
                item(2), // must never be observed
            ],
            decode_i64,
        );
        let got = collect(s).await;
        assert_eq!(got.len(), 2);
        assert!(got[0].is_ok());
        assert!(matches!(got[1], Err(CallError::Plugin(_))));
    }

    #[tokio::test]
    async fn missing_terminal_is_abort() {
        // No END/ERROR: the source just stops after one item.
        let s = ChunkStream::from_frames(vec![item(7)], decode_i64);
        let got = collect(s).await;
        assert_eq!(got.len(), 2);
        assert!(got[0].is_ok());
        assert!(matches!(got[1], Err(CallError::StreamAborted)));
    }

    #[tokio::test]
    async fn malformed_frame_surfaces_then_stops() {
        let s = ChunkStream::from_frame_bytes(
            futures::stream::iter(vec![
                item(1).encode().unwrap(),
                vec![99, 0, 0, 0, 0], // unknown tag
                item(2).encode().unwrap(),
            ]),
            decode_i64,
        );
        let got = collect(s).await;
        assert_eq!(got.len(), 2);
        assert!(got[0].is_ok());
        assert!(matches!(got[1], Err(CallError::MalformedFrame(_))));
    }

    #[tokio::test]
    async fn empty_stream_just_ends() {
        let s = ChunkStream::from_frames(vec![Frame::End], decode_i64);
        assert!(collect(s).await.is_empty());
    }
}
