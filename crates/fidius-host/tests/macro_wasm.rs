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

//! End-to-end: a Rust author defines an interface with the fidius macros, builds
//! it to a WASM component (FIDIUS-T-0106 auto-export, unblocked by the I-0022
//! `fidius-guest` split), and the host loads it through `load_wasm` using the
//! **macro-emitted** `Greeter_WASM_DESCRIPTOR` — proving the descriptor, the
//! generated WIT, and the component's `fidius-interface-hash` all agree.
//!
//! Runs only with `--features wasm` and requires the wasm component toolchain
//! (FIDIUS-T-0094). The fixture `tests/wasm-fixtures/macro-greeter` is built
//! here via a separate `cargo build --target wasm32-wasip2` invocation.

#![cfg(feature = "wasm")]
// The macros emit a `#[cfg(feature = "host")]`-gated Client; this test crate has
// no `host` feature (it doesn't need the Client), so silence the cfg-value lint.
#![allow(unexpected_cfgs)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use fidius_host::{PluginHost, PluginRuntimeKind};

// Define the SAME interface the `macro-greeter` fixture implements. The macro
// computes the interface hash from the method signatures (not the trait name)
// and the export name from the trait name, so an identical definition yields a
// descriptor that matches the fixture's component. `crate = "fidius_core"`
// because that's what this test crate depends on (it re-exports fidius-guest).
#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;

    #[wire(raw)]
    fn echo(&self, data: Vec<u8>) -> Vec<u8>;
}

/// Build the macro-greeter component once and return its bytes.
fn macro_greeter_component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/wasm-fixtures/macro-greeter");
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("run `cargo build --target wasm32-wasip2` (see T-0094 for the toolchain)");
        assert!(status.success(), "macro-greeter wasm build failed");
        let art = fixture.join("target/wasm32-wasip2/release/macro_greeter.wasm");
        std::fs::read(&art).unwrap_or_else(|e| panic!("read {}: {e}", art.display()))
    })
}

/// Stage a `runtime = "wasm"` package containing the built component.
fn stage_pkg(root: &std::path::Path) {
    let dir = root.join("macro-greeter-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        r#"
[package]
name = "macro-greeter-pkg"
version = "0.1.0"
interface = "greeter"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "macro_greeter.wasm"
"#,
    )
    .unwrap();
    std::fs::write(dir.join("macro_greeter.wasm"), macro_greeter_component()).unwrap();
}

#[test]
fn macro_built_component_loads_and_calls() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();

    // Load against the MACRO-EMITTED descriptor. load_wasm validates the
    // component's exported `fidius-interface-hash` against the descriptor's
    // interface_hash — both derived by the macro from the same signatures.
    let handle = host
        .load_wasm(
            "macro-greeter-pkg",
            &__fidius_Greeter::Greeter_WASM_DESCRIPTOR,
        )
        .expect("load_wasm against the macro-generated descriptor");

    assert_eq!(handle.info().runtime, PluginRuntimeKind::Wasm);

    // greet is method 0 (typed); echo is method 1 (#[wire(raw)]).
    let greeting: String = handle.call_method(0, &("Ada".to_string(),)).unwrap();
    assert_eq!(greeting, "Hello, Ada!");

    let reversed = handle.call_method_raw(1, b"abcdef").unwrap();
    assert_eq!(reversed, b"fedcba");
}

#[test]
fn macro_descriptor_export_and_hash_are_self_consistent() {
    // The descriptor the macro emits names the export per its convention and
    // carries the same hash the component exports — sanity-check the constants.
    let desc = &__fidius_Greeter::Greeter_WASM_DESCRIPTOR;
    assert_eq!(desc.interface_export, "fidius:greeter/greeter@0.1.0");
    assert_eq!(desc.interface_name, "Greeter");
    assert_eq!(desc.methods.len(), 2);
    assert_eq!(desc.methods[0].name, "greet");
    assert_eq!(desc.methods[1].name, "echo");
    assert!(desc.methods[1].wire_raw);
}
