// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//! Host composition: consume a stream of **rich-typed records** — a record whose
//! field is a `HashMap` (PC.1 rich WIT types) streamed item-by-item (PC.2
//! record streaming). The same shape a production connector emits: typed events,
//! pulled lazily.
//!
//! (In-process for runnability; a WASM connector authors the identical interface
//! and additionally makes time-boxed HTTP calls via `fidius_guest::http` — see
//! the `records-stream` and `macro-fetcher` fixtures + the connectors how-to.)
//!
//! Run: `cargo run -p fidius-examples --example 05_record_stream`
#![allow(unexpected_cfgs)]

use std::collections::HashMap;

use fidius::{plugin_impl, plugin_interface, PluginHandle, Stream};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Event {
    pub id: u32,
    /// A map field — rich types ride inside records (PC.1).
    pub tags: HashMap<String, String>,
}

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Source: Send + Sync {
    fn events(&self, count: u32) -> Stream<Event>;
}

pub struct Feed;

#[plugin_impl(Source)]
impl Source for Feed {
    fn events(&self, count: u32) -> Stream<Event> {
        Stream::from_iter((0..count).map(|i| {
            let mut tags = HashMap::new();
            tags.insert("kind".into(), "demo".into());
            tags.insert("seq".into(), i.to_string());
            Event { id: i, tags }
        }))
    }
}

fidius::fidius_plugin_registry!();

#[tokio::main]
async fn main() {
    let handle = PluginHandle::from_descriptor(
        PluginHandle::find_in_process_descriptor("Feed").expect("registered"),
    )
    .expect("load");

    // Pull typed records lazily; each carries a HashMap field.
    let mut stream = handle
        .call_streaming::<_, Event>(0, &(3u32,))
        .await
        .expect("stream");
    let mut got = Vec::new();
    while let Some(item) = stream.next().await {
        let ev = fidius::from_value::<Event>(item.unwrap()).unwrap();
        println!("event {} tags={:?}", ev.id, ev.tags);
        got.push(ev);
    }

    assert_eq!(got.len(), 3);
    assert_eq!(got[2].id, 2);
    assert_eq!(got[2].tags.get("seq"), Some(&"2".to_string()));
}
