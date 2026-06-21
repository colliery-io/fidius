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

//! End-to-end for WIT reserved keywords as author identifiers (FIDIUS-T-0177):
//! the `keyword-fields` fixture has a `record` whose fields are `record`,
//! `stream`, `from`, `r#type`, a `variant` whose cases are `Stream`/`Record`/
//! `List`, and a method `record`. The fixture's build.rs (`emit_wit`) must
//! `%`-escape every one of these or the generated wit/ won't parse — so a
//! successful build proves the escaping, and loading + calling here proves the
//! load-time interface-hash match (host descriptor ↔ guest) and that the
//! generated conversions marshal a keyword-field record both ways.

#![cfg(feature = "wasm")]
#![allow(unexpected_cfgs)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use fidius_host::PluginHost;

// Host-side mirror of the fixture's interface — same signatures → same interface
// hash and export names as the guest. Fields/cases deliberately collide with WIT
// keywords; serde uses the bare names (`type`, not `r#type`), matching WIT.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DeadLetter {
    pub record: String,
    pub stream: u64,
    pub from: bool,
    pub r#type: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Outcome {
    Stream,
    Record(u32),
    List { from: u8 },
}

#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Sink: Send + Sync {
    fn record(&self, list: DeadLetter, option: Outcome) -> DeadLetter;
}

fn keyword_fields_component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/wasm-fixtures/keyword-fields");
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("run `cargo build --target wasm32-wasip2` (see T-0094 for the toolchain)");
        assert!(status.success(), "keyword-fields wasm build failed");
        let art = fixture.join("target/wasm32-wasip2/release/keyword_fields.wasm");
        std::fs::read(&art).unwrap_or_else(|e| panic!("read {}: {e}", art.display()))
    })
}

fn stage_pkg(root: &std::path::Path) {
    let dir = root.join("keyword-fields-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        r#"
[package]
name = "keyword-fields-pkg"
version = "0.1.0"
interface = "sink"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "keyword_fields.wasm"
"#,
    )
    .unwrap();
    std::fs::write(dir.join("keyword_fields.wasm"), keyword_fields_component()).unwrap();
}

#[test]
fn keyword_field_record_round_trips() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    // The load itself checks the descriptor's interface hash against the
    // component — host (derive) ↔ guest (emit_wit) must agree.
    let handle = host
        .load_wasm("keyword-fields-pkg", &__fidius_Sink::Sink_WASM_DESCRIPTOR)
        .expect("load keyword-fields against the macro descriptor");

    let input = DeadLetter {
        record: "dead".to_string(),
        stream: 10,
        from: true,
        r#type: 7,
    };
    // record (method 0): bumps `stream` by the Outcome payload. `Record(5)` → +5.
    let out: DeadLetter = handle
        .call_method(0, &(input.clone(), Outcome::Record(5)))
        .unwrap();
    assert_eq!(
        out,
        DeadLetter {
            stream: 15,
            ..input.clone()
        }
    );

    // Struct-variant case `List { from }` → synthetic record payload; +3.
    let out2: DeadLetter = handle
        .call_method(0, &(input.clone(), Outcome::List { from: 3 }))
        .unwrap();
    assert_eq!(out2.stream, 13);

    // Unit case `Stream` → +0.
    let out3: DeadLetter = handle
        .call_method(0, &(input.clone(), Outcome::Stream))
        .unwrap();
    assert_eq!(out3.stream, 10);
    assert_eq!(out3.r#type, 7);
}
