// Copyright 2026 Colliery, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! cdylib client- and bidi-streaming with a **user-typed (record) stream item**
//! (FIDIUS-T-0171). Stream items cross as bincode, so any `Serialize`/`Deserialize`
//! type works as the item — no `#[derive(WitType)]` needed. cdylib never gated this;
//! this locks it in alongside the WASM fix.

#![cfg(feature = "streaming")]
#![allow(unexpected_cfgs)]

use fidius_host::PluginHandle;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Row {
    pub id: u64,
    pub name: String,
}

#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Rows: Send + Sync {
    // client-streaming: a record stream argument.
    fn sum_ids(&self, rows: fidius_core::Stream<Row>) -> u64;
    // bidirectional: record in, record out.
    fn bump(&self, rows: fidius_core::Stream<Row>) -> fidius_core::Stream<Row>;
}

pub struct Tool;

#[fidius_macro::plugin_impl(Rows, crate = "fidius_core")]
impl Rows for Tool {
    fn sum_ids(&self, mut rows: fidius_core::Stream<Row>) -> u64 {
        let mut sum = 0u64;
        while let Some(r) = rows.next_item() {
            sum += r.id;
        }
        sum
    }

    fn bump(&self, mut rows: fidius_core::Stream<Row>) -> fidius_core::Stream<Row> {
        fidius_core::Stream::from_iter(std::iter::from_fn(move || {
            rows.next_item().map(|r| Row {
                id: r.id + 100,
                name: r.name,
            })
        }))
    }
}

fidius_core::fidius_plugin_registry!();

fn rows() -> Vec<Row> {
    vec![
        Row {
            id: 1,
            name: "a".into(),
        },
        Row {
            id: 2,
            name: "b".into(),
        },
        Row {
            id: 3,
            name: "c".into(),
        },
    ]
}

#[test]
fn cdylib_client_streaming_accepts_a_record_item() {
    let desc = PluginHandle::find_in_process_descriptor("Tool").unwrap();
    let handle = PluginHandle::from_descriptor(desc).unwrap();
    let sum: u64 = handle
        .call_client_streaming::<Row, (), u64>(0, rows(), &())
        .expect("client-streaming with a record item");
    assert_eq!(sum, 6);
}

#[tokio::test]
async fn cdylib_bidi_streaming_transforms_record_items() {
    let desc = PluginHandle::find_in_process_descriptor("Tool").unwrap();
    let handle = PluginHandle::from_descriptor(desc).unwrap();
    let mut stream = handle
        .call_bidi_streaming::<Row, (), Row>(1, rows(), &())
        .await
        .expect("bidi with record items");
    let mut got = Vec::new();
    while let Some(item) = stream.next().await {
        got.push(fidius_core::from_value::<Row>(item.unwrap()).unwrap());
    }
    assert_eq!(
        got.iter().map(|r| r.id).collect::<Vec<_>>(),
        vec![101, 102, 103]
    );
}
