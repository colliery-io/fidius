// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//! Host composition: a real **multi-plugin pipeline**. The host orchestrates two
//! plugins — a streaming *reader* and a *transformer* — wiring the reader's stream
//! into the transformer, one item at a time. This is "plugin A's output → plugin
//! B's input", driven entirely by the host.
//!
//! Run: `cargo run -p fidius-examples --example 04_pipeline`
#![allow(unexpected_cfgs)]

use fidius::{plugin_impl, plugin_interface, PluginHandle, Stream};
use futures::StreamExt;

// ── Plugin A: a streaming source ────────────────────────────────────────────
#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Reader: Send + Sync {
    fn read(&self, count: u32) -> Stream<u64>;
}

pub struct Range;

#[plugin_impl(Reader)]
impl Reader for Range {
    fn read(&self, count: u32) -> Stream<u64> {
        Stream::from_iter(1..=count as u64)
    }
}

// ── Plugin B: a transformer (consumes one record, emits one) ─────────────────
#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Transformer: Send + Sync {
    fn transform(&self, value: u64) -> String;
}

pub struct Labeler;

#[plugin_impl(Transformer)]
impl Transformer for Labeler {
    fn transform(&self, value: u64) -> String {
        format!("record #{value}")
    }
}

fidius::fidius_plugin_registry!();

#[tokio::main]
async fn main() {
    // The host loads BOTH plugins and owns the wiring between them.
    let reader = PluginHandle::from_descriptor(
        PluginHandle::find_in_process_descriptor("Range").expect("Range registered"),
    )
    .expect("load reader");
    let transformer = PluginHandle::from_descriptor(
        PluginHandle::find_in_process_descriptor("Labeler").expect("Labeler registered"),
    )
    .expect("load transformer");

    // Pipe: reader.read() stream → transformer.transform() → collected output.
    // Backpressured + lazy: each record is pulled, transformed, and emitted one at
    // a time (swap the `while` body for a sink/writer plugin for a full A→B→C pipe).
    let mut stream = reader
        .call_streaming::<_, u64>(0, &(3u32,))
        .await
        .expect("reader stream");
    let mut out = Vec::new();
    while let Some(item) = stream.next().await {
        let n = fidius::from_value::<u64>(item.unwrap()).unwrap();
        let labeled: String = transformer.call_method(0, &(n,)).expect("transform");
        out.push(labeled);
    }

    println!("{out:?}");
    assert_eq!(out, vec!["record #1", "record #2", "record #3"]);
}
