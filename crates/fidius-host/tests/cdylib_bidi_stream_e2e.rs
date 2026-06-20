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

//! cdylib **bidirectional** streaming end to end (FIDIUS-I-0032 / ADR-0010): a method
//! that takes `Stream<In>` AND returns `Stream<Out>`. The host produces the input and
//! consumes the output; pulling an output item drives the plugin, which pulls one input
//! item **on demand** (re-entering the host producer) — the synchronous lazy-pull
//! composition. Proves the `BidiStreamFn` shim (input pull → output stream handle) +
//! `call_bidi_streaming` + drop-cancel.

#![cfg(feature = "streaming")]
#![allow(unexpected_cfgs)]

use fidius_core::from_value;
use fidius_host::PluginHandle;
use futures::StreamExt;

#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Transformer: Send + Sync {
    /// Bidirectional: consume a `Stream<u64>`, produce a `Stream<u64>`.
    fn transform(&self, input: fidius_core::Stream<u64>) -> fidius_core::Stream<u64>;
}

pub struct Doubler;

#[fidius_macro::plugin_impl(Transformer, crate = "fidius_core")]
impl Transformer for Doubler {
    fn transform(&self, mut input: fidius_core::Stream<u64>) -> fidius_core::Stream<u64> {
        // Lazy: each OUTPUT pull pulls exactly ONE input item (re-entering the host
        // producer) and doubles it. No input is drained up front — `from_fn` only runs
        // when the host pulls the next output, which is the re-entrancy this exercises.
        fidius_core::Stream::from_iter(std::iter::from_fn(move || input.next_item().map(|x| x * 2)))
    }
}

fidius_core::fidius_plugin_registry!();

#[tokio::test]
async fn cdylib_bidi_doubles_a_host_produced_stream() {
    let desc = PluginHandle::find_in_process_descriptor("Doubler").unwrap();
    let handle = PluginHandle::from_descriptor(desc).unwrap();

    // Host produces [1..=5]; each output item is computed by pulling one input item.
    let items: Vec<u64> = vec![1, 2, 3, 4, 5];
    let mut stream = handle
        .call_bidi_streaming::<u64, (), u64>(0, items, &())
        .await
        .expect("call_bidi_streaming");

    let mut got = Vec::new();
    while let Some(item) = stream.next().await {
        got.push(from_value::<u64>(item.expect("item ok")).expect("u64"));
    }
    assert_eq!(got, vec![2, 4, 6, 8, 10]);
}

#[tokio::test]
async fn cdylib_bidi_drop_cancels_the_chain() {
    let desc = PluginHandle::find_in_process_descriptor("Doubler").unwrap();
    let handle = PluginHandle::from_descriptor(desc).unwrap();

    // Pull only the first two of a longer stream, then drop it. Dropping the output
    // ChunkStream runs the output handle's drop_fn, which frees the input HostStream —
    // the whole chain tears down without consuming the rest. No hang, no leak.
    let items: Vec<u64> = (1..=100).collect();
    let mut stream = handle
        .call_bidi_streaming::<u64, (), u64>(0, items, &())
        .await
        .expect("call_bidi_streaming");

    let first = from_value::<u64>(stream.next().await.expect("item").expect("ok")).unwrap();
    let second = from_value::<u64>(stream.next().await.expect("item").expect("ok")).unwrap();
    assert_eq!((first, second), (2, 4));
    drop(stream); // cancels — exercises the output handle's drop_fn → frees the input
}
