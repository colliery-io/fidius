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

//! WASM **bidirectional** streaming end to end (FIDIUS-I-0032 / ADR-0010): a method
//! that takes `Stream<In>` AND returns `Stream<Out>`. The component imports
//! `fidius:stream-pull` (input) AND exports a streaming resource (output); the host
//! produces the input and pumps the output. Pulling an output item drives the guest,
//! which pulls one input item via the import (the synchronous lazy-pull composition).

#![cfg(all(feature = "wasm", feature = "streaming"))]
#![allow(unexpected_cfgs)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use fidius_core::from_value;
use fidius_host::PluginHost;
use futures::StreamExt;

// Same signatures as the fixture → same interface hash + export name.
#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Transformer: Send + Sync {
    fn transform(&self, input: fidius_core::Stream<u64>) -> fidius_core::Stream<u64>;
}

fn component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/wasm-fixtures/bidi-stream");
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("run `cargo build --target wasm32-wasip2` (see T-0094 for the toolchain)");
        assert!(status.success(), "bidi-stream wasm build failed");
        let art = fixture.join("target/wasm32-wasip2/release/bidi_stream.wasm");
        std::fs::read(&art).unwrap_or_else(|e| panic!("read {}: {e}", art.display()))
    })
}

fn stage_pkg(root: &std::path::Path) {
    let dir = root.join("bidi-stream-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        r#"
[package]
name = "bidi-stream-pkg"
version = "0.1.0"
interface = "transformer"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "bidi_stream.wasm"
"#,
    )
    .unwrap();
    std::fs::write(dir.join("bidi_stream.wasm"), component()).unwrap();
}

#[tokio::test]
async fn wasm_bidi_doubles_a_host_produced_stream() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm(
            "bidi-stream-pkg",
            &__fidius_Transformer::Transformer_WASM_DESCRIPTOR,
        )
        .expect("load bidi-stream");

    // Host produces [1..=5]; each output item is the guest doubling one pulled input.
    let items: Vec<u64> = vec![1, 2, 3, 4, 5];
    let mut stream = handle
        .call_bidi_streaming::<u64, (), u64>(0, items, &())
        .await
        .expect("call_bidi_streaming");

    let mut got = Vec::new();
    while let Some(item) = stream.next().await {
        got.push(from_value::<u64>(item.expect("item ok")).expect("u64"));
    }
    assert_eq!(got, vec![2, 4, 6, 8, 10]);
}
