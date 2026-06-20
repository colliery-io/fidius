// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//! Host composition: consume a server-streaming plugin via `call_streaming`
//! (pull-based, backpressured, drop-to-cancel).
//!
//! Run: `cargo run -p fidius-examples --example 03_streaming`
#![allow(unexpected_cfgs)]

use fidius::{plugin_impl, plugin_interface, PluginHandle, Stream};
use futures::StreamExt;

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Source: Send + Sync {
    fn read(&self, count: u32) -> Stream<u64>;
}

pub struct Counter;

#[plugin_impl(Source)]
impl Source for Counter {
    fn read(&self, count: u32) -> Stream<u64> {
        Stream::from_iter(0..count as u64)
    }
}

fidius::fidius_plugin_registry!();

#[tokio::main]
async fn main() {
    let desc = PluginHandle::find_in_process_descriptor("Counter").expect("registered");
    let handle = PluginHandle::from_descriptor(desc).expect("load");

    // Pull items lazily; only the items you take are produced.
    let mut stream = handle
        .call_streaming::<_, u64>(0, &(5u32,))
        .await
        .expect("stream");
    let mut got = Vec::new();
    while let Some(item) = stream.next().await {
        got.push(fidius::from_value::<u64>(item.unwrap()).unwrap());
    }
    println!("{got:?}");
    assert_eq!(got, vec![0, 1, 2, 3, 4]);
}
