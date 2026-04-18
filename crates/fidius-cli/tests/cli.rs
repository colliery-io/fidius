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

//! CLI integration tests using assert_cmd.

use assert_cmd::Command;
use fidius_test::dylib_fixture;
use predicates::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn fidius_cmd() -> Command {
    Command::cargo_bin("fidius").unwrap()
}

fn plugin_source_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/test-plugin-smoke")
}

fn plugin_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        dylib_fixture(plugin_source_dir())
            .build()
            .dir()
            .to_path_buf()
    })
}

fn smoke_dylib_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "libtest_plugin_smoke.dylib"
    } else if cfg!(target_os = "windows") {
        "test_plugin_smoke.dll"
    } else {
        "libtest_plugin_smoke.so"
    }
}

#[test]
fn help_works() {
    fidius_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init-interface"))
        .stdout(predicate::str::contains("init-plugin"))
        .stdout(predicate::str::contains("keygen"))
        .stdout(predicate::str::contains("sign"))
        .stdout(predicate::str::contains("verify"))
        .stdout(predicate::str::contains("inspect"));
}

#[test]
fn init_interface_creates_files() {
    let tmp = TempDir::new().unwrap();

    fidius_cmd()
        .args([
            "init-interface",
            "my-api",
            "--trait",
            "MyTrait",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created interface crate"));

    let cargo_toml = tmp.path().join("my-api/Cargo.toml");
    let lib_rs = tmp.path().join("my-api/src/lib.rs");

    assert!(cargo_toml.exists());
    assert!(lib_rs.exists());

    let cargo_content = std::fs::read_to_string(&cargo_toml).unwrap();
    assert!(cargo_content.contains("name = \"my-api\""));
    assert!(cargo_content.contains("fidius"));

    let lib_content = std::fs::read_to_string(&lib_rs).unwrap();
    assert!(lib_content.contains("MyTrait"));
    assert!(lib_content.contains("plugin_interface"));
}

#[test]
fn init_interface_errors_if_exists() {
    let tmp = TempDir::new().unwrap();

    // Create first
    fidius_cmd()
        .args([
            "init-interface",
            "my-api",
            "--trait",
            "MyTrait",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Try again — should fail
    fidius_cmd()
        .args([
            "init-interface",
            "my-api",
            "--trait",
            "MyTrait",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn init_plugin_creates_files() {
    let tmp = TempDir::new().unwrap();

    fidius_cmd()
        .args([
            "init-plugin",
            "my-plugin",
            "--interface",
            "my-api",
            "--trait",
            "MyTrait",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created plugin crate"));

    let cargo_toml = tmp.path().join("my-plugin/Cargo.toml");
    let lib_rs = tmp.path().join("my-plugin/src/lib.rs");

    assert!(cargo_toml.exists());
    assert!(lib_rs.exists());

    let cargo_content = std::fs::read_to_string(&cargo_toml).unwrap();
    assert!(cargo_content.contains("cdylib"));
    assert!(cargo_content.contains("my-api"));

    let lib_content = std::fs::read_to_string(&lib_rs).unwrap();
    assert!(lib_content.contains("MyTrait"));
    assert!(lib_content.contains("plugin_impl"));
    assert!(lib_content.contains("fidius_plugin_registry"));
}

#[test]
fn keygen_sign_verify_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let key_base = tmp.path().join("testkey");
    let dylib = plugin_dir().join(smoke_dylib_name());

    // Keygen
    fidius_cmd()
        .args(["keygen", "--out", key_base.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated keypair"));

    let secret = format!("{}.secret", key_base.display());
    let public = format!("{}.public", key_base.display());
    assert!(PathBuf::from(&secret).exists());
    assert!(PathBuf::from(&public).exists());

    // Sign
    fidius_cmd()
        .args(["sign", "--key", &secret, dylib.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Signed"));

    // Verify
    fidius_cmd()
        .args(["verify", "--key", &public, dylib.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Signature valid"));

    // Cleanup sig
    let sig_ext = format!("{}.sig", dylib.extension().unwrap().to_str().unwrap());
    let _ = std::fs::remove_file(dylib.with_extension(sig_ext));
}

#[test]
fn inspect_shows_plugin_info() {
    let dylib = plugin_dir().join(smoke_dylib_name());

    fidius_cmd()
        .args(["inspect", dylib.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("BasicCalculator"))
        .stdout(predicate::str::contains("Calculator"))
        .stdout(predicate::str::contains("PluginAllocated"));
}
