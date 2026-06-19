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

//! Composition harness for streaming plugins — **for tests, not production**.
//!
//! `stream_of` / `collect` / `pump` make it trivial to test a streaming plugin
//! in isolation or to wire a producer to a consumer. They are the reference `|`
//! for "pipes of plugins."
//!
//! # Stability
//!
//! **This module is deliberately NOT part of fidius's semver-stable surface.**
//! It exists so tests can compose streams without re-deriving the same loop, and
//! so the canonical composition pattern is readable in one place. Its behaviour
//! is *correct* (pull-paced backpressure, stop-and-surface on the first error)
//! but its API may change at any time. **In production, write your own pump
//! loop** — fidius ships the typed pipe (the [`ChunkStream`] primitive), not an
//! orchestrator; orchestration (scheduling, retries, observability) is yours.
//! See ADR FIDIUS-A-0004 ("Streaming as Mechanism, Not Protocol").

use std::sync::Mutex;

use fidius_core::Value;
use fidius_host::{CallError, ChunkStream};
use futures::StreamExt;

/// The destination side of a pipe: a consumer `pump` hands each item to, in
/// pull order. Deliberately tiny — a real destination plugin presents whatever
/// richer surface it likes; this is only what composition tests need.
#[async_trait::async_trait]
pub trait StreamSink {
    /// Accept one streamed item. Returning `Err` stops the pump and surfaces.
    async fn accept(&self, item: Value) -> Result<(), CallError>;
}

/// An in-memory source over a fixed item sequence. **Test-only by nature** —
/// you would never construct a stream from a `Vec` in production; that it is
/// useful only in tests is exactly why the whole module is test-tier.
///
/// Yields each value as `Ok`, then a clean end of stream.
pub fn stream_of(items: Vec<Value>) -> ChunkStream {
    ChunkStream::new(futures::stream::iter(
        items.into_iter().map(Ok::<Value, CallError>),
    ))
}

/// Drain a stream to a `Vec`, stopping at — and returning — the first error.
/// The single-plugin unit-test idiom: `collect(plugin.process(stream_of(rows)))`.
pub async fn collect(mut s: ChunkStream) -> Result<Vec<Value>, CallError> {
    let mut out = Vec::new();
    while let Some(item) = s.next().await {
        out.push(item?);
    }
    Ok(out)
}

/// The reference pull-loop wiring a producer stream to a [`StreamSink`].
///
/// Pull-paced: exactly one item is awaited at a time, so a slow sink naturally
/// backpressures the producer (the next item is not pulled until the sink has
/// accepted the current one). Stops at the first error from either side —
/// producer error or sink rejection — and returns it; on a clean end of stream
/// returns `Ok(())`. This is the ~10 lines you would copy into production and
/// then grow your own retries/observability around.
pub async fn pump<S>(mut out: ChunkStream, into: &S) -> Result<(), CallError>
where
    S: StreamSink + ?Sized,
{
    while let Some(item) = out.next().await {
        into.accept(item?).await?;
    }
    Ok(())
}

/// A [`StreamSink`] that records everything it accepts — for asserting on the
/// far end of a `pump`.
#[derive(Default)]
pub struct CollectSink {
    items: Mutex<Vec<Value>>,
}

impl CollectSink {
    /// A fresh, empty sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot of everything accepted so far.
    pub fn take(&self) -> Vec<Value> {
        self.items.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl StreamSink for CollectSink {
    async fn accept(&self, item: Value) -> Result<(), CallError> {
        self.items.lock().unwrap().push(item);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fidius_core::{from_value, to_value};

    fn vals(xs: &[i64]) -> Vec<Value> {
        xs.iter().map(|n| to_value(n).unwrap()).collect()
    }

    fn ints(vs: Vec<Value>) -> Vec<i64> {
        vs.into_iter().map(|v| from_value(v).unwrap()).collect()
    }

    #[tokio::test]
    async fn stream_of_then_collect_round_trips() {
        let got = collect(stream_of(vals(&[1, 2, 3]))).await.unwrap();
        assert_eq!(ints(got), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn collect_surfaces_first_error() {
        // A stream that yields one item then an error.
        let s = ChunkStream::new(futures::stream::iter(vec![
            Ok(to_value(&1i64).unwrap()),
            Err(CallError::StreamAborted),
        ]));
        let err = collect(s).await.unwrap_err();
        assert!(matches!(err, CallError::StreamAborted));
    }

    #[tokio::test]
    async fn pump_delivers_all_items_to_sink() {
        let sink = CollectSink::new();
        pump(stream_of(vals(&[10, 20, 30])), &sink).await.unwrap();
        assert_eq!(ints(sink.take()), vec![10, 20, 30]);
    }

    #[tokio::test]
    async fn pump_stops_on_producer_error() {
        let sink = CollectSink::new();
        let s = ChunkStream::new(futures::stream::iter(vec![
            Ok(to_value(&7i64).unwrap()),
            Err(CallError::StreamAborted),
            Ok(to_value(&8i64).unwrap()), // must never reach the sink
        ]));
        let err = pump(s, &sink).await.unwrap_err();
        assert!(matches!(err, CallError::StreamAborted));
        assert_eq!(ints(sink.take()), vec![7]);
    }

    #[tokio::test]
    async fn compose_single_plugin_idiom() {
        // collect(transform(stream_of(..))) — here "transform" doubles each item.
        let doubled = stream_of(vals(&[1, 2, 3])).map(|r| {
            r.map(|v| {
                let n: i64 = from_value(v).unwrap();
                to_value(&(n * 2)).unwrap()
            })
        });
        let got = collect(ChunkStream::new(doubled)).await.unwrap();
        assert_eq!(ints(got), vec![2, 4, 6]);
    }
}
