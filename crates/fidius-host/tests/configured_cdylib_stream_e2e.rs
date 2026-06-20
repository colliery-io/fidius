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

//! Configured + streaming on the **cdylib** backend (FIDIUS-I-0029): a configured
//! instance whose server-streaming method reads the bound config. Proves the
//! construct/destroy instance pointer (CI.1) flows into the stream-init shim, so a
//! configured cdylib plugin streams from its bound config.

#![cfg(feature = "streaming")]
#![allow(unexpected_cfgs)]

use fidius_core::from_value;
use fidius_host::PluginHandle;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Cfg {
    base: u64,
}

#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Ticker: Send + Sync {
    fn tick(&self, count: u32) -> fidius_core::Stream<u64>;
}

pub struct ConfTicker {
    cfg: Cfg,
}

#[fidius_macro::plugin_impl(Ticker, crate = "fidius_core", config = Cfg)]
impl Ticker for ConfTicker {
    fn tick(&self, count: u32) -> fidius_core::Stream<u64> {
        let base = self.cfg.base;
        fidius_core::Stream::from_iter((0..count as u64).map(move |i| base + i))
    }
}

impl ConfTicker {
    fn configure(cfg: Cfg) -> Self {
        Self { cfg }
    }
}

fidius_core::fidius_plugin_registry!();

#[tokio::test]
async fn configured_cdylib_streaming_reads_bound_config() {
    let desc = PluginHandle::find_in_process_descriptor("ConfTicker").unwrap();
    let handle = PluginHandle::configure_in_process(desc, &Cfg { base: 100 }).unwrap();

    let mut stream = handle.call_streaming::<_, u64>(0, &(3u32,)).await.unwrap();
    let mut got = Vec::new();
    while let Some(item) = stream.next().await {
        got.push(from_value::<u64>(item.unwrap()).unwrap());
    }
    assert_eq!(got, vec![100, 101, 102]);
}
