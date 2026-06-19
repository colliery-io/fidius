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

//! The `fidius::Stream<T>` server-streaming return marker (FIDIUS-I-0026, D4).

use core::marker::PhantomData;

/// Marker type a plugin interface uses to declare a **server-streaming** method:
///
/// ```ignore
/// #[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
/// pub trait Source: Send + Sync {
///     fn read(&self, config: String) -> fidius::Stream<Row>;
/// }
/// ```
///
/// `#[plugin_interface]` recognises a return type whose final path segment is
/// `Stream<T>` and, for that method:
///
/// 1. folds a `!stream` marker into the interface hash (so a streaming method
///    can never be confused with a unary `-> T` method of the same name/args —
///    a producer/consumer mismatch is rejected at load), and
/// 2. (Phase 1+) generates a host-side client method returning the runtime
///    pull handle (`fidius_host::ChunkStream`).
///
/// The marker carries no data — the *runtime* representation of a stream is the
/// host-side `ChunkStream`, not this type. It exists so the interface trait
/// type-checks and so the macro has an explicit, unambiguous thing to detect
/// (rather than guessing from `impl Stream`). `T` is the per-item type and
/// follows the same wire/`WitType` rules as a unary return.
///
/// Argument-position `Stream<T>` (client-streaming / bidirectional) is rejected
/// in v1.
///
/// ## Two forms (FIDIUS-I-0026)
///
/// - **Marker form** ([`Stream::new`]): no items. Used purely to *declare* a
///   streaming method in an interface trait — the Python path (Phase 1) never
///   iterates it in Rust (its generator is bridged to a `ChunkStream`
///   host-side).
/// - **Iterator-backed form** ([`Stream::from_iter`]): a Rust **WASM** guest
///   (Phase 2) returns real items. The macro-generated component resource pulls
///   them one at a time via [`Stream::next_item`] to satisfy the WIT contract:
///   ```wit
///   resource <m>-stream { next: func() -> result<option<T>, plugin-error>; }
///   <m>: func(args) -> own<<m>-stream>;
///   ```
///   `some(item)` = item, `none` = clean end, `err` = mid-stream error;
///   dropping the resource runs the guest dtor = cancel (design decision D3).
pub struct Stream<T> {
    /// `None` = the pure marker form; `Some` = an iterator-backed producer.
    iter: Option<Box<dyn Iterator<Item = T> + Send>>,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Stream<T> {
    /// The marker form — declares a streaming method without producing items.
    /// The interface/Python path uses this; it is never iterated in Rust.
    pub fn new() -> Self {
        Self {
            iter: None,
            _marker: PhantomData,
        }
    }

    /// Build a stream from any iterator — how a Rust WASM guest produces its
    /// items. The iterator must be `Send + 'static` so the host can drive the
    /// component resource across its pump thread.
    #[allow(clippy::should_implement_trait)] // inherent ctor; can't add Send+'static via FromIterator
    pub fn from_iter<I>(items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + 'static,
    {
        Self {
            iter: Some(Box::new(items.into_iter())),
            _marker: PhantomData,
        }
    }

    /// Advance the underlying iterator by one item. Driven by the
    /// macro-generated component resource's `next()` (WS.2). Returns `None` for
    /// the marker form and at end of iteration.
    pub fn next_item(&mut self) -> Option<T> {
        self.iter.as_mut().and_then(|it| it.next())
    }
}

impl<T> Default for Stream<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter_yields_then_none() {
        let mut s = Stream::from_iter(vec![1u64, 2, 3]);
        assert_eq!(s.next_item(), Some(1));
        assert_eq!(s.next_item(), Some(2));
        assert_eq!(s.next_item(), Some(3));
        assert_eq!(s.next_item(), None);
        assert_eq!(s.next_item(), None);
    }

    #[test]
    fn from_iter_accepts_a_range() {
        let s = Stream::from_iter(0u64..3);
        let got: Vec<u64> = collect(s);
        assert_eq!(got, vec![0, 1, 2]);
    }

    #[test]
    fn marker_form_is_empty() {
        let mut s: Stream<u64> = Stream::new();
        assert_eq!(s.next_item(), None);
        let mut d: Stream<u64> = Stream::default();
        assert_eq!(d.next_item(), None);
    }

    fn collect<T>(mut s: Stream<T>) -> Vec<T> {
        let mut out = Vec::new();
        while let Some(x) = s.next_item() {
            out.push(x);
        }
        out
    }
}
