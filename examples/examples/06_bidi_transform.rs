// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//! Host composition: a **bidirectional** streaming transform (FIDIUS-I-0032 / ADR-0010).
//! The host produces an input stream; the plugin consumes it and produces a transformed
//! output stream, pulled lazily — each output item pulls exactly one input item, on the
//! same call stack (the synchronous lazy-pull composition). The plugin owns the
//! transform end to end (here: doubling).
//!
//! Run: `cargo run -p fidius-examples --example 06_bidi_transform`
#![allow(unexpected_cfgs)]

use fidius::{plugin_impl, plugin_interface, PluginHandle, Stream};
use futures::StreamExt;

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Transformer: Send + Sync {
    /// Consume a `Stream<u64>` (host-produced) and produce a `Stream<u64>`.
    fn transform(&self, input: Stream<u64>) -> Stream<u64>;
}

pub struct Doubler;

#[plugin_impl(Transformer)]
impl Transformer for Doubler {
    fn transform(&self, mut input: Stream<u64>) -> Stream<u64> {
        // Lazy: each output pull pulls exactly one input item (re-entering the host)
        // and doubles it — no input is drained up front.
        Stream::from_iter(std::iter::from_fn(move || input.next_item().map(|x| x * 2)))
    }
}

fidius::fidius_plugin_registry!();

#[tokio::main]
async fn main() {
    let desc = PluginHandle::find_in_process_descriptor("Doubler").expect("registered");
    let handle = PluginHandle::from_descriptor(desc).expect("load");

    // The host produces [1..=5]; the plugin doubles each as the output is pulled.
    let items: Vec<u64> = vec![1, 2, 3, 4, 5];
    let mut stream = handle
        .call_bidi_streaming::<u64, (), u64>(0, items, &())
        .await
        .expect("bidi stream");

    let mut got = Vec::new();
    while let Some(item) = stream.next().await {
        got.push(fidius::from_value::<u64>(item.unwrap()).unwrap());
    }
    println!("{got:?}");
    assert_eq!(got, vec![2, 4, 6, 8, 10]);
}
