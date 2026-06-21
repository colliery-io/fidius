// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// FIDIUS-T-0175 fixture: client/bidi streaming with user types on WASM.
// `Row` derives both WitType (it crosses WIT positions) and Serialize/Deserialize (it is
// also a bincode stream item in `ingest`).

use fidius_macro::{plugin_impl, plugin_interface, WitType};
use serde::{Deserialize, Serialize};

#[derive(WitType, Clone, Serialize, Deserialize)]
pub struct Row {
    pub id: u64,
    pub label: String,
}

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Pipe: Send + Sync {
    // bidi: primitive IN, RECORD OUT (the output item crosses via the WIT resource).
    fn rows(&self, ids: fidius_guest::Stream<u64>) -> fidius_guest::Stream<Row>;
    // client: a primitive stream + a user-typed (WIT) non-stream arg.
    fn count_from(&self, ids: fidius_guest::Stream<u64>, base: Row) -> u64;
    // client: a RECORD stream ITEM (bincode) where `Row` is also a WIT type elsewhere.
    fn ingest(&self, rows: fidius_guest::Stream<Row>) -> u64;
}

pub struct MyPipe;

#[plugin_impl(Pipe, crate = "fidius_guest")]
impl Pipe for MyPipe {
    fn rows(&self, mut ids: fidius_guest::Stream<u64>) -> fidius_guest::Stream<Row> {
        fidius_guest::Stream::from_iter(std::iter::from_fn(move || {
            ids.next_item().map(|id| Row {
                id,
                label: format!("r{id}"),
            })
        }))
    }

    fn count_from(&self, mut ids: fidius_guest::Stream<u64>, base: Row) -> u64 {
        let mut sum = base.id;
        while let Some(x) = ids.next_item() {
            sum += x;
        }
        sum
    }

    fn ingest(&self, mut rows: fidius_guest::Stream<Row>) -> u64 {
        let mut sum = 0u64;
        while let Some(r) = rows.next_item() {
            sum += r.id;
        }
        sum
    }
}
