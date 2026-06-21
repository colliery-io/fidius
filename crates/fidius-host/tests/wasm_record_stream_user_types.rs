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

//! WASM client/bidi streaming **with user types** (FIDIUS-T-0175): a record OUTPUT item
//! (bidi, via the WIT resource), a user-typed WIT non-stream arg alongside a stream, and a
//! record stream ITEM (bincode) that is also a WIT type. Exercises the `has_user` macro
//! branch's new client/bidi codegen + the build.rs WIT (which skips the stream arg).

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
    pub id: u64,
    pub label: String,
}

#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Pipe: Send + Sync {
    fn rows(&self, ids: fidius_core::Stream<u64>) -> fidius_core::Stream<Row>;
    fn count_from(&self, ids: fidius_core::Stream<u64>, base: Row) -> u64;
    fn ingest(&self, rows: fidius_core::Stream<Row>) -> u64;
}

fn component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/wasm-fixtures/record-stream-user-types");
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("run `cargo build --target wasm32-wasip2` (see T-0094 for the toolchain)");
        assert!(
            status.success(),
            "record-stream-user-types wasm build failed"
        );
        let art = fixture.join("target/wasm32-wasip2/release/record_stream_user_types.wasm");
        std::fs::read(&art).unwrap_or_else(|e| panic!("read {}: {e}", art.display()))
    })
}

fn stage_pkg(root: &std::path::Path) {
    let dir = root.join("rsut-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        r#"
[package]
name = "rsut-pkg"
version = "0.1.0"
interface = "pipe"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "record_stream_user_types.wasm"
"#,
    )
    .unwrap();
    std::fs::write(dir.join("record_stream_user_types.wasm"), component()).unwrap();
}

fn load() -> fidius_host::PluginHandle {
    let tmp = Box::leak(Box::new(tempfile::TempDir::new().unwrap()));
    stage_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    host.load_wasm("rsut-pkg", &__fidius_Pipe::Pipe_WASM_DESCRIPTOR)
        .expect("load record-stream-user-types")
}

#[tokio::test]
async fn wasm_bidi_record_output() {
    // bidi: primitive IN, RECORD OUT (the output item crosses via the WIT resource).
    let handle = load();
    let mut stream = handle
        .call_bidi_streaming::<u64, (), Row>(0, vec![1u64, 2, 3], &())
        .await
        .expect("bidi record output");
    let mut got = Vec::new();
    while let Some(item) = stream.next().await {
        got.push(from_value::<Row>(item.unwrap()).unwrap());
    }
    assert_eq!(
        got,
        vec![
            Row {
                id: 1,
                label: "r1".into()
            },
            Row {
                id: 2,
                label: "r2".into()
            },
            Row {
                id: 3,
                label: "r3".into()
            },
        ]
    );
}

#[tokio::test]
async fn wasm_client_with_user_typed_arg() {
    // client: a primitive stream + a user-typed (WIT) non-stream arg.
    let handle = load();
    let base = Row {
        id: 100,
        label: "b".into(),
    };
    let sum: u64 = handle
        .call_client_streaming::<u64, (Row,), u64>(1, vec![1u64, 2, 3], &(base,))
        .expect("client with a user-typed arg");
    assert_eq!(sum, 106); // 100 + 1 + 2 + 3
}

#[tokio::test]
async fn wasm_client_record_stream_item() {
    // client: a RECORD stream ITEM (bincode), where `Row` is also a WIT type elsewhere.
    let handle = load();
    let rows = vec![
        Row {
            id: 1,
            label: "a".into(),
        },
        Row {
            id: 2,
            label: "b".into(),
        },
        Row {
            id: 4,
            label: "c".into(),
        },
    ];
    let sum: u64 = handle
        .call_client_streaming::<Row, (), u64>(2, rows, &())
        .expect("client with a record stream item");
    assert_eq!(sum, 7);
}
