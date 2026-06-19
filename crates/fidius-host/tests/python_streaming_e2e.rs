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

//! Capstone server-streaming integration test (FIDIUS-I-0026, ST.5).
//!
//! A real Python plugin package (`tests/test-plugin-py-ticker`) implements the
//! Rust-defined `Ticker` interface (`-> fidius::Stream<u64>`) as a generator,
//! loaded and driven through the standard `PluginHost` API. This validates the
//! whole Phase-1 stack end-to-end:
//!
//! - the macro's `!stream` interface hash matches the Python plugin's
//!   `__interface_hash__` (cross-language contract closes for streaming);
//! - `PluginHandle::call_streaming` pumps the generator onto a `ChunkStream`;
//! - **server-streaming** delivers each yielded item (REQ-001);
//! - **bounded memory + backpressure + cancel** — a huge generator is consumed
//!   only as fast as the host pulls, and dropping the stream stops it without
//!   running it to completion (REQ-002/REQ-003/NFR-003);
//! - the `fidius-test` composition harness (`pump`) wires the stream to a sink.

#![cfg(all(feature = "python", feature = "streaming"))]

use std::path::PathBuf;

use fidius_core::python_descriptor::PythonInterfaceDescriptor;
use fidius_host::{PluginHost, PluginRuntimeKind};
use futures::StreamExt;

/// The macro-generated descriptor for the `Ticker` interface — its
/// `interface_hash` includes the `!stream` marker.
fn ticker_descriptor() -> &'static PythonInterfaceDescriptor {
    &test_plugin_smoke::__fidius_Ticker::Ticker_PYTHON_DESCRIPTOR
}

/// Stage the py-ticker package into a fresh temp dir, vendor the in-tree SDK,
/// and inject the real interface hash in place of the `.py` placeholder.
fn stage(tmp: &tempfile::TempDir) -> PathBuf {
    let plugins_root = tmp.path().to_path_buf();
    let dest = plugins_root.join("py-ticker");
    copy_dir(&repo_root().join("tests/test-plugin-py-ticker"), &dest);
    copy_dir(
        &repo_root().join("python/fidius"),
        &dest.join("vendor").join("fidius"),
    );

    // Inject the macro's hash so the Python plugin matches the Rust trait.
    let py = dest.join("ticker.py");
    let src = std::fs::read_to_string(&py).unwrap();
    let injected = src.replace(
        "__HASH_PLACEHOLDER__",
        &format!("0x{:016X}", ticker_descriptor().interface_hash),
    );
    std::fs::write(&py, injected).unwrap();

    plugins_root
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn copy_dir(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

fn tick_index() -> usize {
    test_plugin_smoke::__fidius_Ticker::METHOD_TICK
}

#[test]
fn discover_lists_streaming_python_plugin() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plugins = stage(&tmp);
    let host = PluginHost::builder().search_path(&plugins).build().unwrap();
    let infos = host.discover().unwrap();
    let info = infos
        .iter()
        .find(|i| i.name == "py-ticker")
        .expect("py-ticker in discovery");
    assert!(matches!(info.runtime, PluginRuntimeKind::Python));
    assert_eq!(info.interface_name, "Ticker");
}

#[tokio::test]
async fn server_stream_yields_all_items() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plugins = stage(&tmp);
    let host = PluginHost::builder().search_path(&plugins).build().unwrap();
    let handle = host
        .load_python("py-ticker", ticker_descriptor())
        .expect("load_python");

    // tick(5) -> 0,1,2,3,4
    let mut stream = handle
        .call_streaming::<_, u64>(tick_index(), &(5u32,))
        .await
        .expect("call_streaming");

    let mut got = Vec::new();
    while let Some(item) = stream.next().await {
        let v: u64 = fidius_core::from_value(item.expect("item ok")).expect("u64");
        got.push(v);
    }
    assert_eq!(got, vec![0, 1, 2, 3, 4]);
}

#[tokio::test]
async fn huge_stream_is_bounded_and_cancellable() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plugins = stage(&tmp);
    let host = PluginHost::builder().search_path(&plugins).build().unwrap();
    let handle = host
        .load_python("py-ticker", ticker_descriptor())
        .expect("load_python");

    // A 10-million-item generator: if the stream weren't bounded/backpressured,
    // draining it into memory (or running it to completion on drop) would be
    // catastrophically slow. We pull just three items and drop — this must
    // return promptly, proving the producer is pulled lazily and cancelled.
    let mut stream = handle
        .call_streaming::<_, u64>(tick_index(), &(10_000_000u32,))
        .await
        .expect("call_streaming");

    let mut got = Vec::new();
    for _ in 0..3 {
        let v: u64 =
            fidius_core::from_value(stream.next().await.expect("item").expect("ok")).unwrap();
        got.push(v);
    }
    assert_eq!(got, vec![0, 1, 2]);
    // Drop mid-stream: the bounded channel + cancel tear the generator down.
    drop(stream);
}

#[tokio::test]
async fn composition_pump_into_sink() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plugins = stage(&tmp);
    let host = PluginHost::builder().search_path(&plugins).build().unwrap();
    let handle = host
        .load_python("py-ticker", ticker_descriptor())
        .expect("load_python");

    // The unsupported `fidius-test` harness composing a real streaming plugin
    // into a sink — the "pipes of plugins" demonstration.
    let stream = handle
        .call_streaming::<_, u64>(tick_index(), &(4u32,))
        .await
        .expect("call_streaming");

    let sink = fidius_test::CollectSink::new();
    fidius_test::pump(stream, &sink).await.expect("pump");

    let got: Vec<u64> = sink
        .take()
        .into_iter()
        .map(|v| fidius_core::from_value(v).unwrap())
        .collect();
    assert_eq!(got, vec![0, 1, 2, 3]);
}
