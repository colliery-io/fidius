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

//! Tee/replay bodies for the egress response hook (FIDIUS-I-0035).
//!
//! To honor [`ResponseDirective::RetryOnce`](super::wasm::ResponseDirective),
//! the dispatch path must be able to send the guest's request body a second
//! time — but an outgoing body streams to the wire once and is gone. The
//! [`TeeBody`] wrapper solves this without changing wire behavior: frames pass
//! through to the inner body untouched while their bytes are copied into a
//! side buffer, capped at [`REPLAY_CAP`]. Whether a retry is possible is
//! decided *at response time* from the capture's terminal state (a bodiless
//! GET has no `Content-Length`, so a pre-dispatch size check would exclude the
//! most common retryable requests).

use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use bytes::{Bytes, BytesMut};
use http_body::{Body, Frame, SizeHint};
use http_body_util::BodyExt;
use wasmtime_wasi_http::p2::bindings::http::types::ErrorCode;
use wasmtime_wasi_http::p2::body::HyperOutgoingBody;

/// Cap on the bytes a [`TeeBody`] captures for replay (64 KiB). A body that
/// grows past this is dispatched as usual but is not replayable — a
/// `RetryOnce` for it is ignored and the response forwards to the guest.
pub(crate) const REPLAY_CAP: usize = 64 * 1024;

/// Terminal state of a tee capture, read at response time to decide whether
/// a retry may replay the body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CaptureState {
    /// End-of-stream not (cleanly) observed yet — the request may still be
    /// streaming, or the body errored mid-flight. Not replayable.
    Incomplete,
    /// The whole body was observed and fit under [`REPLAY_CAP`]. Replayable.
    Complete,
    /// The body exceeded [`REPLAY_CAP`]; capture was abandoned. Not replayable.
    Overflowed,
    /// The body carried trailers, which a replay would not reproduce. Not
    /// replayable.
    Trailers,
}

struct CaptureInner {
    buf: BytesMut,
    state: CaptureState,
}

/// Shared handle onto a [`TeeBody`]'s capture, held by the dispatch task.
#[derive(Clone)]
pub(crate) struct CaptureHandle(Arc<Mutex<CaptureInner>>);

impl CaptureHandle {
    /// The capture's current terminal state (test observability; the dispatch
    /// path decides off [`replayable`](Self::replayable) alone).
    #[cfg(test)]
    pub(crate) fn state(&self) -> CaptureState {
        self.0.lock().expect("tee capture lock poisoned").state
    }

    /// The bytes to replay, if and only if the body was fully captured under
    /// the cap.
    pub(crate) fn replayable(&self) -> Option<Bytes> {
        let inner = self.0.lock().expect("tee capture lock poisoned");
        (inner.state == CaptureState::Complete).then(|| inner.buf.clone().freeze())
    }
}

/// A pass-through wrapper over a guest's outgoing body that captures its
/// bytes for a possible single replay. Every frame the inner body yields is
/// delivered unchanged; the one deliberate wire difference is that a body
/// whose end was observed at wrap time advertises `is_end_stream()`, so e.g.
/// a bodiless GET goes out with no framing instead of an empty chunked body.
pub(crate) struct TeeBody {
    inner: HyperOutgoingBody,
    capture: Arc<Mutex<CaptureInner>>,
    /// Frames pulled out of `inner` by [`prime`](Self::prime), replayed to
    /// the real consumer before `inner` is polled again.
    stash: std::collections::VecDeque<Result<Frame<Bytes>, ErrorCode>>,
    /// `inner` returned end-of-stream during priming; it must not be polled
    /// again.
    ended: bool,
}

impl TeeBody {
    /// Wrap `inner`, returning the teed body (boxed back to the
    /// [`HyperOutgoingBody`] shape dispatch expects) and the capture handle.
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

    /// Drain whatever the inner body can yield *right now* (noop waker, no
    /// task context) into the stash, recording it in the capture.
    ///
    /// This is what makes the common case deterministic: a wasi-http guest
    /// body is a channel (`is_end_stream()` is never true up front, end only
    /// observable by polling), and hyper's conn task may not have polled it
    /// to completion by the time a fast 401 arrives — the capture would still
    /// be `Incomplete` at decision time and the retry silently unavailable.
    /// A guest that finished its body before dispatch (bodiless GETs, small
    /// JSON POSTs — the motivating weir shape) has every frame already in
    /// the channel, so priming reaches `Complete` here, synchronously.
    ///
    /// A still-streaming body returns `Pending` immediately (registering the
    /// noop waker — harmless: channel bodies re-register on every poll, and
    /// hyper's dispatcher always polls the body at least once without
    /// needing a wakeup first) and replay eligibility stays timing-dependent,
    /// as documented.
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

    fn record_end(&self) {
        // Clean end-of-stream: whatever was captured is the whole body.
        let mut inner = self.capture.lock().expect("tee capture lock poisoned");
        if inner.state == CaptureState::Incomplete {
            inner.state = CaptureState::Complete;
        }
    }
}

impl Body for TeeBody {
    type Data = Bytes;
    type Error = ErrorCode;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        if let Some(item) = self.stash.pop_front() {
            // Already recorded during priming.
            return Poll::Ready(Some(item));
        }
        if self.ended {
            return Poll::Ready(None);
        }
        let poll = Pin::new(&mut self.inner).poll_frame(cx);
        match &poll {
            Poll::Ready(Some(Ok(frame))) => self.record_frame(frame),
            Poll::Ready(None) => self.record_end(),
            // A body error fails the dispatch; leave the capture Incomplete.
            Poll::Ready(Some(Err(_))) | Poll::Pending => {}
        }
        poll
    }

    fn is_end_stream(&self) -> bool {
        if !self.stash.is_empty() {
            return false;
        }
        self.ended || self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        if self.ended {
            let stashed: u64 = self
                .stash
                .iter()
                .filter_map(|i| i.as_ref().ok().and_then(|f| f.data_ref()))
                .map(|d| d.len() as u64)
                .sum();
            return SizeHint::with_exact(stashed);
        }
        self.inner.size_hint()
    }
}

/// A replayable body carrying previously captured bytes, for the retry
/// dispatch.
pub(crate) fn replay_body(bytes: Bytes) -> HyperOutgoingBody {
    http_body_util::Full::new(bytes)
        .map_err(|never| match never {})
        .boxed_unsync()
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderMap;

    /// Test body yielding a scripted sequence of frames.
    struct FrameBody(std::collections::VecDeque<Frame<Bytes>>);

    impl Body for FrameBody {
        type Data = Bytes;
        type Error = ErrorCode;

        fn poll_frame(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
            Poll::Ready(self.0.pop_front().map(Ok))
        }
    }

    fn scripted(frames: Vec<Frame<Bytes>>) -> HyperOutgoingBody {
        FrameBody(frames.into()).boxed_unsync()
    }

    /// Drain a body to end-of-stream, returning the concatenated data frames.
    async fn drain(mut body: HyperOutgoingBody) -> Vec<u8> {
        let mut out = Vec::new();
        while let Some(frame) = body.frame().await {
            if let Some(data) = frame.expect("scripted bodies never error").data_ref() {
                out.extend_from_slice(data);
            }
        }
        out
    }

    /// A body wrapping only after its first poll would be Pending — the
    /// still-streaming shape (guest called `handle` before finishing its
    /// body). Yields Pending once, then its frames.
    struct PendingFirst {
        pending_polls: usize,
        rest: std::collections::VecDeque<Frame<Bytes>>,
    }

    impl Body for PendingFirst {
        type Data = Bytes;
        type Error = ErrorCode;

        fn poll_frame(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
            if self.pending_polls > 0 {
                self.pending_polls -= 1;
                return Poll::Pending;
            }
            Poll::Ready(self.rest.pop_front().map(Ok))
        }
    }

    /// Priming makes a body fully written before dispatch — the wasi-http
    /// guest shape: end-of-stream only observable by polling — Complete at
    /// wrap time, before hyper ever polls. This is what keeps `RetryOnce`
    /// deterministic for bodiless GETs against fast-401 servers.
    #[test]
    fn finished_body_is_complete_at_wrap_time() {
        let (_body, capture) = TeeBody::wrap(scripted(vec![]));
        assert_eq!(capture.state(), CaptureState::Complete);
        assert_eq!(capture.replayable(), Some(Bytes::new()));

        let (_body, capture) = TeeBody::wrap(scripted(vec![Frame::data(Bytes::from_static(
            b"small json",
        ))]));
        assert_eq!(capture.state(), CaptureState::Complete);
        assert_eq!(
            capture.replayable(),
            Some(Bytes::from_static(b"small json"))
        );

        // The already-ended shape (e.g. our own replay body) too.
        let (_body, capture) = TeeBody::wrap(replay_body(Bytes::new()));
        assert_eq!(capture.state(), CaptureState::Complete);
        assert_eq!(capture.replayable(), Some(Bytes::new()));
    }

    #[tokio::test]
    async fn small_body_passes_through_and_captures() {
        let (body, capture) = TeeBody::wrap(scripted(vec![
            Frame::data(Bytes::from_static(b"hello ")),
            Frame::data(Bytes::from_static(b"world")),
        ]));
        // Primed frames must still reach the wire unchanged.
        assert_eq!(drain(body).await, b"hello world");
        assert_eq!(capture.state(), CaptureState::Complete);
        assert_eq!(
            capture.replayable(),
            Some(Bytes::from_static(b"hello world"))
        );
    }

    #[tokio::test]
    async fn streaming_body_passes_through_and_completes_on_drain() {
        // Pending at wrap → nothing primed; the capture completes only once
        // the consumer (hyper) drains the body.
        let (body, capture) = TeeBody::wrap(
            PendingFirst {
                pending_polls: 1,
                rest: [Frame::data(Bytes::from_static(b"late"))].into(),
            }
            .boxed_unsync(),
        );
        assert_eq!(capture.state(), CaptureState::Incomplete);
        assert_eq!(drain(body).await, b"late");
        assert_eq!(capture.state(), CaptureState::Complete);
        assert_eq!(capture.replayable(), Some(Bytes::from_static(b"late")));
    }

    #[tokio::test]
    async fn oversized_body_overflows_but_still_streams() {
        let chunk = Bytes::from(vec![0u8; REPLAY_CAP / 2 + 1]);
        let (body, capture) = TeeBody::wrap(scripted(vec![
            Frame::data(chunk.clone()),
            Frame::data(chunk.clone()),
        ]));
        // The wire still sees every byte…
        assert_eq!(drain(body).await.len(), 2 * chunk.len());
        // …but the capture is dead.
        assert_eq!(capture.state(), CaptureState::Overflowed);
        assert_eq!(capture.replayable(), None);
    }

    #[tokio::test]
    async fn trailers_kill_the_capture() {
        let (body, capture) = TeeBody::wrap(scripted(vec![
            Frame::data(Bytes::from_static(b"data")),
            Frame::trailers(HeaderMap::new()),
        ]));
        assert_eq!(drain(body).await, b"data");
        assert_eq!(capture.state(), CaptureState::Trailers);
        assert_eq!(capture.replayable(), None);
    }

    #[tokio::test]
    async fn undrained_streaming_body_stays_incomplete() {
        let (body, capture) = TeeBody::wrap(
            PendingFirst {
                pending_polls: usize::MAX,
                rest: [].into(),
            }
            .boxed_unsync(),
        );
        drop(body); // dispatch abandoned mid-stream
        assert_eq!(capture.state(), CaptureState::Incomplete);
        assert_eq!(capture.replayable(), None);
    }

    #[tokio::test]
    async fn replay_body_round_trips() {
        let replay = replay_body(Bytes::from_static(b"again"));
        assert_eq!(drain(replay).await, b"again");
    }
}
