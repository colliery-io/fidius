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

//! WASM **client-streaming** end to end (FIDIUS-I-0030 CS2.3): a method whose
//! `Stream<T>` argument is fed by the HOST. The host produces items; the guest
//! pulls them via the `fidius:stream-pull` import (the two `generate!` compose like
//! wasi:http), folds them, and returns. Proves the import + the macro wasm codegen
//! + `WasmHostStream` + `call_client_streaming` over a real component.

#![cfg(all(feature = "wasm", feature = "streaming"))]
#![allow(unexpected_cfgs)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use fidius_host::PluginHost;

// Same signatures as the fixture → same interface hash + export name.
#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Sink: Send + Sync {
    fn load(&self, rows: fidius_core::Stream<u64>) -> u64;
}

fn component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/wasm-fixtures/client-stream");
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("run `cargo build --target wasm32-wasip2` (see T-0094 for the toolchain)");
        assert!(status.success(), "client-stream wasm build failed");
        let art = fixture.join("target/wasm32-wasip2/release/client_stream.wasm");
        std::fs::read(&art).unwrap_or_else(|e| panic!("read {}: {e}", art.display()))
    })
}

fn stage_pkg(root: &std::path::Path) {
    let dir = root.join("client-stream-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        r#"
[package]
name = "client-stream-pkg"
version = "0.1.0"
interface = "sink"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "client_stream.wasm"
"#,
    )
    .unwrap();
    std::fs::write(dir.join("client_stream.wasm"), component()).unwrap();
}

#[test]
fn wasm_consumes_a_host_produced_stream() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm("client-stream-pkg", &__fidius_Sink::Sink_WASM_DESCRIPTOR)
        .expect("load client-stream");

    // The host produces [1..=5]; the guest pulls them via fidius:stream-pull + sums.
    let items: Vec<u64> = vec![1, 2, 3, 4, 5];
    let sum: u64 = handle
        .call_client_streaming(0, items, &())
        .expect("client-streaming call");
    assert_eq!(sum, 15);
}
