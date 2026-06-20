// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// PC.2 fixture: server-streaming a user record over WASM. `rows` returns a
// `Stream<Row>`; the macro emits a `rows-stream` resource whose `next()` converts
// each user `Row` to its WIT binding via the generated From impls.

use fidius_macro::{plugin_impl, plugin_interface, WitType};

#[derive(WitType, Clone)]
pub struct Row {
    pub id: u32,
    pub label: String,
}

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Source: Send + Sync {
    fn rows(&self, count: u32) -> fidius_guest::Stream<Row>;
}

pub struct MySource;

#[plugin_impl(Source, crate = "fidius_guest")]
impl Source for MySource {
    fn rows(&self, count: u32) -> fidius_guest::Stream<Row> {
        fidius_guest::Stream::from_iter((0..count).map(|i| Row {
            id: i,
            label: format!("row-{i}"),
        }))
    }
}
