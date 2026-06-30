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

//! `load_wasm_configured_with_grants`: the configured load path with a
//! caller-supplied capability allow-list that **overrides** the package
//! manifest's `[wasm].capabilities`. This is the primitive cloacina uses for
//! tenant-granted constructor capabilities — the deploying tenant authorizes the
//! filesystem path (etc.) at load time, not the plugin author at package time.
//!
//! Proves, against a real configured WASM guest doing `std::fs` I/O:
//! - a load-time `fs:ro:` grant enables a read the empty manifest would deny
//!   (override applies) AND the bound config (suffix) is used (configured path);
//! - an empty grant list denies all I/O regardless of what the manifest said
//!   (default-closed; the override REPLACES the manifest, it does not merge);
//! - a coarse/invalid grant (`env`) is rejected at load by the same validation
//!   the manifest caps go through (the override is not a trust bypass).

#![cfg(feature = "wasm")]
#![allow(unexpected_cfgs)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use fidius_host::PluginHost;
use serde::Serialize;

#[derive(Serialize)]
struct Cfg {
    suffix: String,
}

// Host-side descriptor mirror — same signatures as the fixture's `ConfiguredFs`
// trait, so the macro derives the same interface hash.
#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait ConfiguredFs: Send + Sync {
    fn read_file(&self, path: String) -> String;
}

const READ_FILE: usize = 0;

fn component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/wasm-fixtures/macro-configured-fs");
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("run `cargo build --target wasm32-wasip2` (see T-0094 for the toolchain)");
        assert!(status.success(), "macro-configured-fs wasm build failed");
        let art = fixture.join("target/wasm32-wasip2/release/macro_configured_fs.wasm");
        std::fs::read(&art).unwrap_or_else(|e| panic!("read {}: {e}", art.display()))
    })
}

/// Stage a `runtime = "wasm"` package whose manifest carries `manifest_caps`.
/// The load-time grants in each test override these — `manifest_caps` is set to
/// the *opposite* of the grant under test to prove the override wins.
fn stage_pkg(root: &Path, manifest_caps: &[String]) {
    let dir = root.join("macro-configured-fs-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    let caps_line = if manifest_caps.is_empty() {
        String::new()
    } else {
        let list = manifest_caps
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!("capabilities = [{list}]\n")
    };
    std::fs::write(
        dir.join("package.toml"),
        format!(
            r#"
[package]
name = "macro-configured-fs-pkg"
version = "0.1.0"
interface = "configuredfs"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "macro_configured_fs.wasm"
{caps_line}"#
        ),
    )
    .unwrap();
    std::fs::write(dir.join("macro_configured_fs.wasm"), component()).unwrap();
}

fn host_for(manifest_caps: &[String]) -> (tempfile::TempDir, PluginHost) {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path(), manifest_caps);
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    (tmp, host)
}

#[test]
fn load_time_grant_overrides_empty_manifest_and_config_is_bound() {
    // Manifest declares NO caps; the tenant grants `fs:ro:<dir>` at load.
    let (_pkg, host) = host_for(&[]);
    let data = tempfile::TempDir::new().unwrap();
    let seed = data.path().join("seed.txt");
    std::fs::write(&seed, "body").unwrap();

    let grant = format!("fs:ro:{}", data.path().display());
    let handle = host
        .load_wasm_configured_with_grants(
            "macro-configured-fs-pkg",
            &__fidius_ConfiguredFs::ConfiguredFs_WASM_DESCRIPTOR,
            &Cfg { suffix: "!".into() },
            vec![grant],
            None,
        )
        .expect("load with fs grant");

    let got: String = handle
        .call_method(READ_FILE, &(seed.display().to_string(),))
        .unwrap();
    // Read succeeded (grant overrode the empty manifest) AND the bound config
    // suffix was appended (the configured store is live).
    assert_eq!(got, "body!");
}

#[test]
fn empty_grant_denies_io_even_when_manifest_would_allow() {
    // Manifest GRANTS fs; the tenant grants NOTHING at load. The override
    // replaces (not merges) the manifest, so I/O is denied — default-closed.
    let data = tempfile::TempDir::new().unwrap();
    let seed = data.path().join("seed.txt");
    std::fs::write(&seed, "body").unwrap();
    let manifest_grant = format!("fs:ro:{}", data.path().display());
    let (_pkg, host) = host_for(&[manifest_grant]);

    let handle = host
        .load_wasm_configured_with_grants(
            "macro-configured-fs-pkg",
            &__fidius_ConfiguredFs::ConfiguredFs_WASM_DESCRIPTOR,
            &Cfg { suffix: "!".into() },
            vec![], // tenant granted nothing
            None,
        )
        .expect("load with no grant");

    let got: String = handle
        .call_method(READ_FILE, &(seed.display().to_string(),))
        .unwrap();
    // Read denied (empty file body) — only the suffix comes back.
    assert_eq!(got, "!");
}

#[test]
fn coarse_grant_is_rejected_at_load() {
    let (_pkg, host) = host_for(&[]);
    // Bare `env` (every host secret) must fail the load — the override goes
    // through the same `validate_capabilities` as manifest caps.
    let res = host.load_wasm_configured_with_grants(
        "macro-configured-fs-pkg",
        &__fidius_ConfiguredFs::ConfiguredFs_WASM_DESCRIPTOR,
        &Cfg { suffix: "!".into() },
        vec!["env".into()],
        None,
    );
    let err = match res {
        Ok(_) => panic!("bare env must be rejected at load"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(
        msg.contains("env"),
        "error should explain the rejected env grant, got: {msg}"
    );
}
