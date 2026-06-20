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

//! Configured **streaming** WASM plugin (FIDIUS-I-0029 / CI.3): config bound once
//! via `fidius-configure`, then a server-streaming method reads it. The stream's
//! store is configured before the stream starts (config crosses once, at stream
//! start), so configured + streaming compose.

#![cfg(all(feature = "wasm", feature = "streaming"))]
#![allow(unexpected_cfgs)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use fidius_core::from_value;
use fidius_host::PluginHost;
use futures::StreamExt;
use serde::Serialize;

#[derive(Serialize)]
struct Cfg {
    base: u64,
}

#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Ticker: Send + Sync {
    fn tick(&self, count: u32) -> fidius_core::Stream<u64>;
}

fn component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/wasm-fixtures/macro-configured-stream");
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("cargo build --target wasm32-wasip2");
        assert!(
            status.success(),
            "macro-configured-stream wasm build failed"
        );
        std::fs::read(fixture.join("target/wasm32-wasip2/release/macro_configured_stream.wasm"))
            .unwrap()
    })
}

fn stage(root: &std::path::Path) {
    let dir = root.join("ticker-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("macro_configured_stream.wasm"), component()).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        "[package]\nname = \"ticker-pkg\"\nversion = \"0.1.0\"\ninterface = \"ticker\"\n\
         interface_version = 1\nruntime = \"wasm\"\n\n[metadata]\ncategory = \"test\"\n\n\
         [wasm]\ncomponent = \"macro_configured_stream.wasm\"\n",
    )
    .unwrap();
}

#[tokio::test]
async fn configured_streaming_reads_bound_config() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();

    let handle = host
        .load_wasm_configured(
            "ticker-pkg",
            &__fidius_Ticker::Ticker_WASM_DESCRIPTOR,
            &Cfg { base: 100 },
        )
        .expect("load_wasm_configured");

    // tick streams base..base+count using the bound `base` (100), not a per-call arg.
    let mut stream = handle
        .call_streaming::<_, u64>(0, &(3u32,))
        .await
        .expect("call_streaming");
    let mut got = Vec::new();
    while let Some(item) = stream.next().await {
        got.push(from_value::<u64>(item.expect("ok")).unwrap());
    }
    assert_eq!(got, vec![100, 101, 102]);
}
