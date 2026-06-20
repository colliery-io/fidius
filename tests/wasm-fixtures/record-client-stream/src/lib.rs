// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// FIDIUS-T-0171 fixture: a RECORD stream item on the INPUT side. `Row` crosses as
// bincode via the fidius:stream-pull import (not a WIT type), so it derives
// Serialize/Deserialize — no #[derive(WitType)]. Covers client-streaming (record in,
// primitive out) and bidirectional (record in, primitive out).

use fidius_macro::{plugin_impl, plugin_interface};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Row {
    pub id: u64,
    pub name: String,
}

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Rows: Send + Sync {
    fn sum_ids(&self, rows: fidius_guest::Stream<Row>) -> u64;
    fn big_ids(&self, rows: fidius_guest::Stream<Row>) -> fidius_guest::Stream<u64>;
}

pub struct Tool;

#[plugin_impl(Rows, crate = "fidius_guest")]
impl Rows for Tool {
    fn sum_ids(&self, mut rows: fidius_guest::Stream<Row>) -> u64 {
        let mut sum = 0u64;
        while let Some(r) = rows.next_item() {
            sum += r.id;
        }
        sum
    }

    fn big_ids(&self, mut rows: fidius_guest::Stream<Row>) -> fidius_guest::Stream<u64> {
        // bidi: record in → primitive out (the output item is a WIT-typed resource value).
        fidius_guest::Stream::from_iter(std::iter::from_fn(move || {
            rows.next_item().map(|r| r.id * 10)
        }))
    }
}
