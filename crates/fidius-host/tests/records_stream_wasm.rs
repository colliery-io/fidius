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

//! PC.2 (FIDIUS-T-0153): server-streaming a user `#[derive(WitType)]` **record**
//! over WASM. The `records-stream` fixture's `rows` returns `Stream<Row>`; the
//! macro emits a `rows-stream` resource whose `next()` yields the record binding.
//! The host collects typed `Row`s, proving records cross the streaming boundary —
//! lifting the prior "streaming items must be primitives/String" restriction.

#![cfg(all(feature = "wasm", feature = "streaming"))]
#![allow(unexpected_cfgs)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use fidius_core::from_value;
use fidius_host::PluginHost;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Row {
    pub id: u32,
    pub label: String,
}

// Same signatures as the fixture → same interface hash + export name.
#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Source: Send + Sync {
    fn rows(&self, count: u32) -> fidius_core::Stream<Row>;
}

fn component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/wasm-fixtures/records-stream");
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("run `cargo build --target wasm32-wasip2` (see T-0094 for the toolchain)");
        assert!(status.success(), "records-stream wasm build failed");
        let art = fixture.join("target/wasm32-wasip2/release/records_stream.wasm");
        std::fs::read(&art).unwrap_or_else(|e| panic!("read {}: {e}", art.display()))
    })
}

fn stage_pkg(root: &std::path::Path) {
    let dir = root.join("records-stream-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        r#"
[package]
name = "records-stream-pkg"
version = "0.1.0"
interface = "source"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "records_stream.wasm"
"#,
    )
    .unwrap();
    std::fs::write(dir.join("records_stream.wasm"), component()).unwrap();
}

fn row(id: u32) -> Row {
    Row {
        id,
        label: format!("row-{id}"),
    }
}

#[tokio::test]
async fn streams_typed_records() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm(
            "records-stream-pkg",
            &__fidius_Source::Source_WASM_DESCRIPTOR,
        )
        .expect("load records-stream");

    let mut stream = handle
        .call_streaming::<_, Row>(0, &(3u32,))
        .await
        .expect("call_streaming");
    let mut got = Vec::new();
    while let Some(item) = stream.next().await {
        got.push(from_value::<Row>(item.expect("item ok")).expect("Row"));
    }
    assert_eq!(got, vec![row(0), row(1), row(2)]);
}

#[tokio::test]
async fn record_stream_is_bounded_and_cancellable() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm(
            "records-stream-pkg",
            &__fidius_Source::Source_WASM_DESCRIPTOR,
        )
        .unwrap();

    // 10M records: pulled lazily + torn down on drop, else this would hang.
    let mut stream = handle
        .call_streaming::<_, Row>(0, &(10_000_000u32,))
        .await
        .unwrap();
    let mut got = Vec::new();
    for _ in 0..3 {
        got.push(from_value::<Row>(stream.next().await.expect("item").expect("ok")).unwrap());
    }
    assert_eq!(got, vec![row(0), row(1), row(2)]);
    drop(stream);
}
