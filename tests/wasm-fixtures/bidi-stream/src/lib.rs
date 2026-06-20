// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// BD.3 fixture: a BIDIRECTIONAL plugin. `transform` consumes a host-produced
// Stream<u64> (pulled via the fidius:stream-pull import) and produces a Stream<u64>
// (exported as a resource the host pumps). Doubles each item, lazily: each output
// pull pulls exactly one input item (re-entering the host through the import).

use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Transformer: Send + Sync {
    fn transform(&self, input: fidius_guest::Stream<u64>) -> fidius_guest::Stream<u64>;
}

pub struct Doubler;

#[plugin_impl(Transformer, crate = "fidius_guest")]
impl Transformer for Doubler {
    fn transform(&self, mut input: fidius_guest::Stream<u64>) -> fidius_guest::Stream<u64> {
        fidius_guest::Stream::from_iter(std::iter::from_fn(move || {
            input.next_item().map(|x| x * 2)
        }))
    }
}
