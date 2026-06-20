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

//! Path-scoped filesystem capability, end to end (FIDIUS-A-0008). A real WASM guest
//! does `std::fs` I/O; the host grants exactly one directory via `fs:ro:`/`fs:rw:`.
//! Proves: rw grant round-trips a write+read, no grant denies all I/O, ro grant
//! permits reads but denies writes.

#![cfg(feature = "wasm")]
#![allow(unexpected_cfgs)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use fidius_host::PluginHost;

// Host-side descriptor mirror — same signatures as the fixture's `Fs` trait, so the
// macro derives the same interface hash. (Built for the host; not compiled to wasm.)
#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Fs: Send + Sync {
    fn read_file(&self, path: String) -> String;
    fn write_file(&self, path: String, contents: String) -> bool;
}

const READ_FILE: usize = 0;
const WRITE_FILE: usize = 1;

fn fs_component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/wasm-fixtures/macro-fs");
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("run `cargo build --target wasm32-wasip2` (see T-0094 for the toolchain)");
        assert!(status.success(), "macro-fs wasm build failed");
        let art = fixture.join("target/wasm32-wasip2/release/macro_fs.wasm");
        std::fs::read(&art).unwrap_or_else(|e| panic!("read {}: {e}", art.display()))
    })
}

/// Stage a `runtime = "wasm"` package with the given fs capability grants.
fn stage_pkg(root: &Path, caps: &[String]) {
    let dir = root.join("macro-fs-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    let caps_line = if caps.is_empty() {
        String::new()
    } else {
        let list = caps
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
name = "macro-fs-pkg"
version = "0.1.0"
interface = "fs"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "macro_fs.wasm"
{caps_line}"#
        ),
    )
    .unwrap();
    std::fs::write(dir.join("macro_fs.wasm"), fs_component()).unwrap();
}

fn host_for(caps: &[String]) -> (tempfile::TempDir, PluginHost) {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path(), caps);
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    (tmp, host)
}

#[test]
fn rw_grant_round_trips_a_write_and_read() {
    // The data dir is separate from the package dir so the grant is exactly it.
    let data = tempfile::TempDir::new().unwrap();
    let grant = format!("fs:rw:{}", data.path().display());
    let (_pkg, host) = host_for(&[grant]);
    let handle = host
        .load_wasm("macro-fs-pkg", &__fidius_Fs::Fs_WASM_DESCRIPTOR)
        .unwrap();

    let file = data.path().join("out.txt");
    let path = file.display().to_string();

    let wrote: bool = handle
        .call_method(WRITE_FILE, &(path.clone(), "hello".to_string()))
        .unwrap();
    assert!(wrote, "rw grant must allow the guest to write");
    // The host sees the guest's write on disk.
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "hello");

    let got: String = handle.call_method(READ_FILE, &(path,)).unwrap();
    assert_eq!(got, "hello", "guest must read back what it wrote");
}

#[test]
fn no_grant_denies_all_io() {
    let data = tempfile::TempDir::new().unwrap();
    let (_pkg, host) = host_for(&[]); // deny-all
    let handle = host
        .load_wasm("macro-fs-pkg", &__fidius_Fs::Fs_WASM_DESCRIPTOR)
        .unwrap();

    let path = data.path().join("x.txt").display().to_string();
    let wrote: bool = handle
        .call_method(WRITE_FILE, &(path.clone(), "x".to_string()))
        .unwrap();
    assert!(!wrote, "no grant ⇒ write must be denied");
    let got: String = handle.call_method(READ_FILE, &(path,)).unwrap();
    assert_eq!(got, "", "no grant ⇒ read must be denied (empty)");
}

#[test]
fn ro_grant_allows_read_but_denies_write() {
    let data = tempfile::TempDir::new().unwrap();
    let seed = data.path().join("seed.txt");
    std::fs::write(&seed, "seed").unwrap();
    let grant = format!("fs:ro:{}", data.path().display());
    let (_pkg, host) = host_for(&[grant]);
    let handle = host
        .load_wasm("macro-fs-pkg", &__fidius_Fs::Fs_WASM_DESCRIPTOR)
        .unwrap();

    let got: String = handle
        .call_method(READ_FILE, &(seed.display().to_string(),))
        .unwrap();
    assert_eq!(got, "seed", "ro grant must allow reads");

    let new = data.path().join("new.txt").display().to_string();
    let wrote: bool = handle
        .call_method(WRITE_FILE, &(new, "nope".to_string()))
        .unwrap();
    assert!(!wrote, "ro grant must deny writes");
}
