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

//! End-to-end, the **full guest→host egress loop** (FIDIUS-I-0028 / GH.2): a
//! connector written entirely with the fidius macros, whose `fetch` calls
//! `fidius_guest::http::get`, built to a WASM component, loaded by the host and
//! brokered through the **shipped** `EgressPolicy` (FIDIUS-I-0027). No
//! hand-written WIT, no raw wasi:http bindings — exactly what an adopter's
//! codegen emits. Proves the macro's export `generate!` and `fidius-guest::http`'s
//! wasi:http `generate!` compose, and that the result rides the two-key gate.

#![cfg(feature = "wasm")]
#![allow(unexpected_cfgs)]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use fidius_host::executor::{EgressDenied, EgressPolicy};
use fidius_host::{LoadError, PluginHost};

// The SAME interface the `macro-fetcher` fixture implements. The macro derives
// the interface hash + export name from the signatures/trait, so this yields a
// descriptor that matches the fixture's component (`crate = "fidius_core"` —
// what this host test crate depends on).
#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Fetcher: Send + Sync {
    fn fetch(&self, url: String) -> String;
}

/// Build the macro-fetcher component once.
fn macro_fetcher_component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/wasm-fixtures/macro-fetcher");
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("run `cargo build --target wasm32-wasip2`");
        assert!(status.success(), "macro-fetcher wasm build failed");
        let art = fixture.join("target/wasm32-wasip2/release/macro_fetcher.wasm");
        std::fs::read(&art).unwrap_or_else(|e| panic!("read {}: {e}", art.display()))
    })
}

/// One-shot loopback mock HTTP server serving a single request with `body`.
fn mock_http_once(body: &'static str) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{addr}/");
    let h = std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 2048];
            let _ = stream.read(&mut buf);
            let resp = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });
    (url, h)
}

struct AllowAll;
impl EgressPolicy for AllowAll {
    fn authorize(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
        Ok(())
    }
}

struct DenyAll;
impl EgressPolicy for DenyAll {
    fn authorize(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
        Err(EgressDenied::new("denied by test policy"))
    }
}

/// Stage the macro-fetcher as a `runtime = "wasm"` package declaring `http`.
fn stage_pkg(root: &std::path::Path) {
    let dir = root.join("macro-fetcher-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        "[package]\nname = \"macro-fetcher-pkg\"\nversion = \"0.1.0\"\ninterface = \"fetcher\"\n\
         interface_version = 1\nruntime = \"wasm\"\n\n[metadata]\ncategory = \"test\"\n\n\
         [wasm]\ncomponent = \"macro_fetcher.wasm\"\ncapabilities = [\"http\"]\n",
    )
    .unwrap();
    std::fs::write(dir.join("macro_fetcher.wasm"), macro_fetcher_component()).unwrap();
}

#[test]
fn macro_connector_egress_allowed() {
    let (url, server) = mock_http_once("hello from a macro connector");
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .egress(AllowAll)
        .build()
        .unwrap();
    let handle = host
        .load_wasm(
            "macro-fetcher-pkg",
            &__fidius_Fetcher::Fetcher_WASM_DESCRIPTOR,
        )
        .expect("load_wasm");
    let body: String = handle.call_method(0, &(url,)).expect("fetch");
    server.join().unwrap();
    assert_eq!(body, "hello from a macro connector");
}

#[test]
fn macro_connector_egress_denied() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path());
    // Per-plugin policy that refuses — the connector sees a transport error.
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm_with_egress(
            "macro-fetcher-pkg",
            &__fidius_Fetcher::Fetcher_WASM_DESCRIPTOR,
            DenyAll,
        )
        .expect("load_wasm_with_egress");
    let body: String = handle
        .call_method(0, &("http://127.0.0.1:1/".to_string(),))
        .expect("fetch");
    assert!(
        body.starts_with("ERROR:"),
        "denied egress should surface as ERROR, got: {body}"
    );
}

#[test]
fn macro_connector_no_policy_fails_closed() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path());
    // `http` declared but no policy → wasi:http unlinked → fails closed at load.
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let res = host.load_wasm(
        "macro-fetcher-pkg",
        &__fidius_Fetcher::Fetcher_WASM_DESCRIPTOR,
    );
    assert!(
        matches!(res, Err(LoadError::WasmLoad(_))),
        "must fail closed without an egress policy"
    );
}
