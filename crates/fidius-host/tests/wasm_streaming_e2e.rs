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

//! WASM server-streaming E2E (FIDIUS-I-0026 Phase 2). A real Rust component
//! (`tests/wasm-fixtures/ticker`) exports a streaming `resource`; the host drives
//! it through `PluginHandle::call_streaming` → `ChunkStream`. Proves items,
//! bounded memory + backpressure + cancel (huge stream, pull-few-then-drop), all
//! under the sandbox. Requires the component toolchain (cargo-component).

#![cfg(all(feature = "wasm", feature = "streaming"))]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use fidius_core::descriptor::BufferStrategyKind;
use fidius_core::from_value;
use fidius_host::executor::{WasmComponentExecutor, WasmMethod};
use fidius_host::{PluginHandle, PluginInfo, PluginRuntimeKind};
use futures::StreamExt;

const IFACE: &str = "fidius:ticker/ticker@1.0.0";
// fnv1a("tick:u32->u64!stream")
const HASH: u64 = 0xFD15_2C8A_A111_2FC3;

fn ticker_component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/wasm-fixtures/ticker");
        let status = Command::new("cargo")
            .args(["component", "build", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("run `cargo component build` (is cargo-component installed?)");
        assert!(status.success(), "cargo component build failed");
        let art = fixture.join("target/wasm32-wasip1/release/ticker_guest.wasm");
        std::fs::read(&art).unwrap_or_else(|e| panic!("read {}: {e}", art.display()))
    })
}

fn handle() -> PluginHandle {
    let info = PluginInfo {
        name: "ticker".to_string(),
        interface_name: "ticker".to_string(),
        interface_hash: HASH,
        interface_version: 1,
        capabilities: 0,
        buffer_strategy: BufferStrategyKind::PluginAllocated,
        runtime: PluginRuntimeKind::Wasm,
    };
    let methods = vec![WasmMethod {
        name: "tick".to_string(),
        wire_raw: false,
        streaming: true,
    }];
    let exec = WasmComponentExecutor::from_component_bytes(
        ticker_component(),
        IFACE.to_string(),
        methods,
        vec![],
        info,
    )
    .expect("build executor");
    PluginHandle::from_wasm(exec)
}

#[tokio::test]
async fn wasm_stream_yields_all_items() {
    let h = handle();
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
async fn wasm_huge_stream_is_bounded_and_cancellable() {
    let h = handle();
    // 10M items: must be pulled lazily and torn down on drop, else this hangs.
    let mut stream = h
        .call_streaming::<_, u64>(0, &(10_000_000u32,))
        .await
        .expect("call_streaming");
    let mut got = Vec::new();
    for _ in 0..3 {
        got.push(from_value::<u64>(stream.next().await.expect("item").expect("ok")).unwrap());
    }
    assert_eq!(got, vec![0, 1, 2]);
    drop(stream); // resource drop → guest dtor → cancel
}

#[tokio::test]
async fn wasm_empty_stream() {
    let h = handle();
    let mut stream = h
        .call_streaming::<_, u64>(0, &(0u32,))
        .await
        .expect("call_streaming");
    assert!(stream.next().await.is_none());
}

// Composition / "pipes of plugins": the unsupported `fidius-test` `pump` harness
// wires a real WASM streaming plugin into a sink — the same harness that
// composes the Python backend (python_streaming_e2e::composition_pump_into_sink),
// proving it's backend-neutral.
#[tokio::test]
async fn wasm_composition_pump_into_sink() {
    let h = handle();
    let stream = h
        .call_streaming::<_, u64>(0, &(4u32,))
        .await
        .expect("call_streaming");

    let sink = fidius_test::CollectSink::new();
    fidius_test::pump(stream, &sink).await.expect("pump");

    let got: Vec<u64> = sink
        .take()
        .into_iter()
        .map(|v| from_value::<u64>(v).unwrap())
        .collect();
    assert_eq!(got, vec![0, 1, 2, 3]);
}

// ── Polyglot: a non-Rust (JavaScript / jco) guest serving the SAME streaming
// resource (FIDIUS-I-0026 WS.5). Proves the streaming contract is language-
// neutral — the host drives it with identical code. The committed component is
// built by tests/wasm-fixtures/ticker-js/build.sh; the test skips cleanly where
// it isn't present (e.g. CI without jco).

fn ticker_js_component() -> Option<Vec<u8>> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/wasm-fixtures/ticker-js/ticker_js.wasm");
    std::fs::read(p).ok()
}

fn js_handle(bytes: &[u8]) -> PluginHandle {
    let info = PluginInfo {
        name: "ticker-js".to_string(),
        interface_name: "ticker".to_string(),
        interface_hash: HASH,
        interface_version: 1,
        capabilities: 0,
        buffer_strategy: BufferStrategyKind::PluginAllocated,
        runtime: PluginRuntimeKind::Wasm,
    };
    let methods = vec![WasmMethod {
        name: "tick".to_string(),
        wire_raw: false,
        streaming: true,
    }];
    let exec = WasmComponentExecutor::from_component_bytes(
        bytes,
        IFACE.to_string(),
        methods,
        vec![],
        info,
    )
    .expect("build JS executor");
    PluginHandle::from_wasm(exec)
}

#[tokio::test]
async fn polyglot_js_guest_streams() {
    let Some(bytes) = ticker_js_component() else {
        eprintln!(
            "SKIP polyglot_js_guest_streams: ticker_js.wasm not built \
             (run tests/wasm-fixtures/ticker-js/build.sh)"
        );
        return;
    };
    // Same host code as the Rust guest — the contract is language-neutral.
    let h = js_handle(&bytes);
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
async fn polyglot_js_guest_bounded_and_cancellable() {
    let Some(bytes) = ticker_js_component() else {
        eprintln!("SKIP polyglot_js_guest_bounded_and_cancellable: ticker_js.wasm not built");
        return;
    };
    let h = js_handle(&bytes);
    let mut stream = h
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

// ── Polyglot: a Python (componentize-py) guest serving the SAME streaming
// resource. Third language (Rust, JS, Python) through the identical host path.
// Committed component built by tests/wasm-fixtures/ticker-py/build.sh; skips
// cleanly where it isn't present (e.g. CI without componentize-py).

fn ticker_py_component() -> Option<Vec<u8>> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/wasm-fixtures/ticker-py/ticker_py.wasm");
    std::fs::read(p).ok()
}

fn py_wasm_handle(bytes: &[u8]) -> PluginHandle {
    let info = PluginInfo {
        name: "ticker-py".to_string(),
        interface_name: "ticker".to_string(),
        interface_hash: HASH,
        interface_version: 1,
        capabilities: 0,
        buffer_strategy: BufferStrategyKind::PluginAllocated,
        runtime: PluginRuntimeKind::Wasm,
    };
    let methods = vec![WasmMethod {
        name: "tick".to_string(),
        wire_raw: false,
        streaming: true,
    }];
    let exec = WasmComponentExecutor::from_component_bytes(
        bytes,
        IFACE.to_string(),
        methods,
        vec![],
        info,
    )
    .expect("build Python-WASM executor");
    PluginHandle::from_wasm(exec)
}

#[tokio::test]
async fn polyglot_py_wasm_guest_streams() {
    let Some(bytes) = ticker_py_component() else {
        eprintln!(
            "SKIP polyglot_py_wasm_guest_streams: ticker_py.wasm not built \
             (run tests/wasm-fixtures/ticker-py/build.sh)"
        );
        return;
    };
    let h = py_wasm_handle(&bytes);
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
async fn polyglot_py_wasm_guest_bounded_and_cancellable() {
    let Some(bytes) = ticker_py_component() else {
        eprintln!("SKIP polyglot_py_wasm_guest_bounded_and_cancellable: ticker_py.wasm not built");
        return;
    };
    let h = py_wasm_handle(&bytes);
    let mut stream = h
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

// ── Polyglot: a C (wit-bindgen + wasi-sdk) guest serving the SAME streaming
// resource. Fourth language (Rust, JS, Python, C) through the identical host
// path; the C component embeds no runtime (~18 KB). Committed component built by
// tests/wasm-fixtures/ticker-c/build.sh; skips where it isn't present.

fn ticker_c_component() -> Option<Vec<u8>> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/wasm-fixtures/ticker-c/ticker_c.wasm");
    std::fs::read(p).ok()
}

fn c_wasm_handle(bytes: &[u8]) -> PluginHandle {
    let info = PluginInfo {
        name: "ticker-c".to_string(),
        interface_name: "ticker".to_string(),
        interface_hash: HASH,
        interface_version: 1,
        capabilities: 0,
        buffer_strategy: BufferStrategyKind::PluginAllocated,
        runtime: PluginRuntimeKind::Wasm,
    };
    let methods = vec![WasmMethod {
        name: "tick".to_string(),
        wire_raw: false,
        streaming: true,
    }];
    let exec = WasmComponentExecutor::from_component_bytes(
        bytes,
        IFACE.to_string(),
        methods,
        vec![],
        info,
    )
    .expect("build C-WASM executor");
    PluginHandle::from_wasm(exec)
}

#[tokio::test]
async fn polyglot_c_wasm_guest_streams() {
    let Some(bytes) = ticker_c_component() else {
        eprintln!(
            "SKIP polyglot_c_wasm_guest_streams: ticker_c.wasm not built \
             (run tests/wasm-fixtures/ticker-c/build.sh)"
        );
        return;
    };
    let h = c_wasm_handle(&bytes);
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
async fn polyglot_c_wasm_guest_bounded_and_cancellable() {
    let Some(bytes) = ticker_c_component() else {
        eprintln!("SKIP polyglot_c_wasm_guest_bounded_and_cancellable: ticker_c.wasm not built");
        return;
    };
    let h = c_wasm_handle(&bytes);
    let mut stream = h
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
