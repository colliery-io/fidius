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

//! Host-orchestrated **multi-plugin pipeline**: the host wires one plugin's stream
//! into a SECOND plugin (a plugin-backed `StreamSink`), via `fidius_test::pump`.
//! Proves plugin-A.stream → host → plugin-B.input end to end — the composition the
//! framework's `pump` helper exists for.

#![cfg(feature = "streaming")]
#![allow(unexpected_cfgs)]

use fidius_core::Value;
use fidius_host::{CallError, PluginHandle};
use fidius_test::StreamSink;

// Plugin A: a streaming source.
#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Reader: Send + Sync {
    fn read(&self, count: u32) -> fidius_core::Stream<u64>;
}
pub struct Range;
#[fidius_macro::plugin_impl(Reader, crate = "fidius_core")]
impl Reader for Range {
    fn read(&self, count: u32) -> fidius_core::Stream<u64> {
        fidius_core::Stream::from_iter(1..=count as u64)
    }
}

// Plugin B: a transformer the pipeline sinks into.
#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Writer: Send + Sync {
    fn label(&self, value: u64) -> String;
}
pub struct Labeler;
#[fidius_macro::plugin_impl(Writer, crate = "fidius_core")]
impl Writer for Labeler {
    fn label(&self, value: u64) -> String {
        format!("record #{value}")
    }
}

fidius_core::fidius_plugin_registry!();

/// A `StreamSink` backed by a real plugin: each streamed item is fed to the
/// writer plugin, and its output collected.
struct PluginSink {
    writer: PluginHandle,
    out: std::sync::Mutex<Vec<String>>,
}

#[async_trait::async_trait]
impl StreamSink for PluginSink {
    async fn accept(&self, item: Value) -> Result<(), CallError> {
        let n: u64 = fidius_core::from_value(item).map_err(|e| CallError::Backend {
            runtime: "pipeline".into(),
            message: e.to_string(),
        })?;
        let labeled: String = self.writer.call_method(0, &(n,))?;
        self.out.lock().unwrap().push(labeled);
        Ok(())
    }
}

#[tokio::test]
async fn host_pipes_reader_stream_into_writer_plugin() {
    let reader =
        PluginHandle::from_descriptor(PluginHandle::find_in_process_descriptor("Range").unwrap())
            .unwrap();
    let writer =
        PluginHandle::from_descriptor(PluginHandle::find_in_process_descriptor("Labeler").unwrap())
            .unwrap();

    let sink = PluginSink {
        writer,
        out: std::sync::Mutex::new(Vec::new()),
    };

    // The host owns the wiring: reader's stream → pump → writer plugin.
    let stream = reader.call_streaming::<_, u64>(0, &(3u32,)).await.unwrap();
    fidius_test::pump(stream, &sink).await.unwrap();

    assert_eq!(
        *sink.out.lock().unwrap(),
        vec!["record #1", "record #2", "record #3"]
    );
}
