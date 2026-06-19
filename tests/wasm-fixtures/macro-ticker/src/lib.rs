// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// WASM author fixture (FIDIUS-I-0026): a fidius *server-streaming* plugin
// written entirely with the fidius macros. `#[plugin_impl]` auto-exports a WIT
// component whose `tick` returns a `tick-stream` resource; the macro's generated
// `GuestTickStream::next` drives the `fidius::Stream<u64>` iterator the method
// returns. Built for wasm32-wasip2; loaded via the macro-emitted
// `Ticker_WASM_DESCRIPTOR`.

use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Ticker: Send + Sync {
    /// Server-streaming: yield `0..count`.
    fn tick(&self, count: u32) -> fidius_guest::Stream<u64>;
}

pub struct MyTicker;

#[plugin_impl(Ticker, crate = "fidius_guest")]
impl Ticker for MyTicker {
    fn tick(&self, count: u32) -> fidius_guest::Stream<u64> {
        fidius_guest::Stream::from_iter(0..count as u64)
    }
}
