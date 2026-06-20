// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// CS2.3 fixture: a client-streaming plugin. `load` takes a `Stream<u64>` the HOST
// produces; the guest pulls each item via the `fidius:stream-pull` import
// (WasmHostStream, wired by the macro) and folds them. The macro's interface-export
// generate! composes with fidius_guest::client_stream's import generate!.

use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Sink: Send + Sync {
    /// Client-streaming: the host produces `rows`; the plugin pulls + sums them.
    fn load(&self, rows: fidius_guest::Stream<u64>) -> u64;
}

pub struct MySink;

#[plugin_impl(Sink, crate = "fidius_guest")]
impl Sink for MySink {
    fn load(&self, mut rows: fidius_guest::Stream<u64>) -> u64 {
        let mut sum = 0u64;
        while let Some(x) = rows.next_item() {
            sum += x;
        }
        sum
    }
}
