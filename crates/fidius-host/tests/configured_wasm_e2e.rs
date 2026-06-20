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

//! Configured WASM plugin instances (FIDIUS-I-0029 / CI.3, ADR-0006): a
//! macro-authored `#[plugin_impl(.., config = Cfg)]` connector whose config is
//! bound once via the `fidius-configure` export onto a persistent store; methods
//! then run on the configured instance without re-passing config. N differently-
//! configured instances coexist (each its own store).

#![cfg(feature = "wasm")]
#![allow(unexpected_cfgs)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use fidius_host::PluginHost;
use serde::Serialize;

#[derive(Serialize)]
struct Cfg {
    greeting: String,
}

// SAME interface the fixture implements → matching macro-generated descriptor.
#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;
}

fn component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/wasm-fixtures/macro-configured");
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("cargo build --target wasm32-wasip2");
        assert!(status.success(), "macro-configured wasm build failed");
        std::fs::read(fixture.join("target/wasm32-wasip2/release/macro_configured.wasm")).unwrap()
    })
}

fn stage(root: &std::path::Path) {
    let dir = root.join("macro-configured-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("macro_configured.wasm"), component()).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        "[package]\nname = \"macro-configured-pkg\"\nversion = \"0.1.0\"\ninterface = \"greeter\"\n\
         interface_version = 1\nruntime = \"wasm\"\n\n[metadata]\ncategory = \"test\"\n\n\
         [wasm]\ncomponent = \"macro_configured.wasm\"\n",
    )
    .unwrap();
}

#[test]
fn config_bound_once_and_used_in_methods() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();

    let handle = host
        .load_wasm_configured(
            "macro-configured-pkg",
            &__fidius_Greeter::Greeter_WASM_DESCRIPTOR,
            &Cfg {
                greeting: "Hej".into(),
            },
        )
        .expect("load_wasm_configured");

    // greet is method 0; the caller passes only `name`, config is already bound.
    let out: String = handle.call_method(0, &("Ada".to_string(),)).unwrap();
    assert_eq!(out, "Hej, Ada!");
    // Second call on the same configured instance — config persists.
    let out2: String = handle.call_method(0, &("Bo".to_string(),)).unwrap();
    assert_eq!(out2, "Hej, Bo!");
}

#[test]
fn n_differently_configured_instances_coexist() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();

    let a = host
        .load_wasm_configured(
            "macro-configured-pkg",
            &__fidius_Greeter::Greeter_WASM_DESCRIPTOR,
            &Cfg {
                greeting: "Hej".into(),
            },
        )
        .unwrap();
    let b = host
        .load_wasm_configured(
            "macro-configured-pkg",
            &__fidius_Greeter::Greeter_WASM_DESCRIPTOR,
            &Cfg {
                greeting: "Hi".into(),
            },
        )
        .unwrap();
    let oa: String = a.call_method(0, &("Ada".to_string(),)).unwrap();
    let ob: String = b.call_method(0, &("Bo".to_string(),)).unwrap();
    assert_eq!(oa, "Hej, Ada!");
    assert_eq!(ob, "Hi, Bo!");
}
