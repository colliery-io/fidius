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

//! cdylib server-streaming E2E (FIDIUS-I-0026 CS.1 / Phase 3). The
//! `TickerImpl` plugin in `tests/test-plugin-smoke` implements
//! `tick(count) -> fidius::Stream<u64>`; `#[plugin_impl]` generates the
//! iterator-handle FFI shims (init/next/drop). Loaded as a real `.dylib` and
//! driven through `PluginHandle::call_streaming` — proving cdylib is now a true
//! streaming peer alongside Python and WASM.

#![cfg(feature = "streaming")]

use std::path::PathBuf;

use fidius_core::from_value;
use fidius_host::{PluginHandle, PluginHost};
use fidius_test::dylib_fixture;
use futures::StreamExt;

fn ticker_handle() -> PluginHandle {
    let fixture = dylib_fixture(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/test-plugin-smoke"),
    )
    .build();
    let host = PluginHost::builder()
        .search_path(fixture.dir())
        .build()
        .unwrap();
    // TickerImpl is one of several plugins in the smoke dylib; Ticker has a
    // single method `tick` at index 0.
    PluginHandle::from_loaded(host.load("TickerImpl").unwrap())
}

#[tokio::test]
async fn cdylib_stream_yields_all_items() {
    let h = ticker_handle();
    let mut stream = h
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
async fn cdylib_empty_stream() {
    let h = ticker_handle();
    let mut stream = h
        .call_streaming::<_, u64>(0, &(0u32,))
        .await
        .expect("call_streaming");
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn cdylib_huge_stream_is_bounded_and_cancellable() {
    let h = ticker_handle();
    // 10M items: pulled lazily + torn down on drop (drop_fn runs), else this
    // would take far longer than the test does.
    let mut stream = h
        .call_streaming::<_, u64>(0, &(10_000_000u32,))
        .await
        .expect("call_streaming");
    let mut got = Vec::new();
    for _ in 0..3 {
        got.push(from_value::<u64>(stream.next().await.expect("item").expect("ok")).unwrap());
    }
    assert_eq!(got, vec![0, 1, 2]);
    drop(stream); // → guest drop_fn (cancel)
}
