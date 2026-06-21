// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//! Host composition: a **client-streaming** sink (FIDIUS-I-0030 / ADR-0007). The host
//! produces a stream of items and the plugin *pulls* and reduces them — the inverse of
//! server-streaming. The producer is lazy: items are encoded only as the plugin pulls
//! them, so an unbounded input streams with bounded memory.
//!
//! Run: `cargo run -p fidius-examples --example 07_client_streaming`
#![allow(unexpected_cfgs)]

use fidius::{plugin_impl, plugin_interface, PluginHandle, Stream};

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Sink: Send + Sync {
    /// The `Stream<u64>` argument is host-produced; the plugin pulls + sums it.
    fn sum(&self, rows: Stream<u64>) -> u64;
}

pub struct Adder;

#[plugin_impl(Sink)]
impl Sink for Adder {
    fn sum(&self, mut rows: Stream<u64>) -> u64 {
        let mut total = 0u64;
        while let Some(x) = rows.next_item() {
            total += x;
        }
        total
    }
}

fidius::fidius_plugin_registry!();

fn main() {
    let desc = PluginHandle::find_in_process_descriptor("Adder").expect("registered");
    let handle = PluginHandle::from_descriptor(desc).expect("load");

    // The host produces 1..=100; the plugin pulls each item and sums → 5050.
    let total: u64 = handle
        .call_client_streaming::<u64, (), u64>(0, 1u64..=100, &())
        .expect("client-streaming");
    println!("{total}");
    assert_eq!(total, 5050);
}
