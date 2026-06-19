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

//! WS.2/WS.4 capstone: a fidius **macro-generated** WASM server-streaming
//! component (`tests/wasm-fixtures/macro-ticker`) loaded through `PluginHost`
//! and driven via `call_streaming`. Proves the `#[plugin_impl]` resource-adapter
//! codegen (FIDIUS-I-0026 WS.2) produces a component the host streams from
//! identically to the hand-authored one — write a trait, get a streaming plugin.
//!
//! Requires the wasm component toolchain (cargo + wasm32-wasip2).

#![cfg(all(feature = "wasm", feature = "streaming"))]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use fidius_core::from_value;
use fidius_host::{PluginHost, PluginRuntimeKind};
use futures::StreamExt;

// The SAME interface the `macro-ticker` fixture implements — the macro derives
// the hash from the signature and the export name from the trait name, so an
// identical definition yields a descriptor that matches the built component.
#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Ticker: Send + Sync {
    fn tick(&self, count: u32) -> fidius_core::Stream<u64>;
}

fn macro_ticker_component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/wasm-fixtures/macro-ticker");
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("run `cargo build --target wasm32-wasip2` (see T-0094 for the toolchain)");
        assert!(status.success(), "macro-ticker wasm build failed");
        let art = fixture.join("target/wasm32-wasip2/release/macro_ticker.wasm");
        std::fs::read(&art).unwrap_or_else(|e| panic!("read {}: {e}", art.display()))
    })
}

fn stage_pkg(root: &std::path::Path) {
    let dir = root.join("macro-ticker-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        r#"
[package]
name = "macro-ticker-pkg"
version = "0.1.0"
interface = "ticker"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "macro_ticker.wasm"
"#,
    )
    .unwrap();
    std::fs::write(dir.join("macro_ticker.wasm"), macro_ticker_component()).unwrap();
}

#[test]
fn macro_descriptor_marks_tick_streaming() {
    let desc = &__fidius_Ticker::Ticker_WASM_DESCRIPTOR;
    assert_eq!(desc.interface_export, "fidius:ticker/ticker@0.1.0");
    assert_eq!(desc.methods.len(), 1);
    assert_eq!(desc.methods[0].name, "tick");
    assert!(
        desc.methods[0].streaming,
        "the macro must mark `tick` as a streaming method"
    );
}

#[tokio::test]
async fn macro_streaming_component_loads_and_streams() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();

    let handle = host
        .load_wasm("macro-ticker-pkg", &__fidius_Ticker::Ticker_WASM_DESCRIPTOR)
        .expect("load_wasm against the macro-generated descriptor");
    assert_eq!(handle.info().runtime, PluginRuntimeKind::Wasm);

    let mut stream = handle
        .call_streaming::<_, u64>(0, &(5u32,))
        .await
        .expect("call_streaming");
    let mut got = Vec::new();
    while let Some(item) = stream.next().await {
        got.push(from_value::<u64>(item.expect("item ok")).expect("u64"));
    }
    assert_eq!(got, vec![0, 1, 2, 3, 4]);
}

#[tokio::test]
async fn macro_streaming_bounded_and_cancellable() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm("macro-ticker-pkg", &__fidius_Ticker::Ticker_WASM_DESCRIPTOR)
        .expect("load_wasm");

    // 10M items: pulled lazily + torn down on drop, else this would hang.
    let mut stream = handle
        .call_streaming::<_, u64>(0, &(10_000_000u32,))
        .await
        .expect("call_streaming");
    let mut got = Vec::new();
    for _ in 0..3 {
        got.push(from_value::<u64>(stream.next().await.expect("item").expect("ok")).unwrap());
    }
    assert_eq!(got, vec![0, 1, 2]);
    drop(stream);
}
