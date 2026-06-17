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

//! CLI wiring for `fidius package pack` on wasm packages (FIDIUS-T-0107).
//!
//! These cover the default (no-`wasm`-feature) build: validation/precompile are
//! skipped with a warning, and `--precompile` errors with a rebuild hint. The
//! actual validate/precompile + load round-trips are covered by the
//! `fidius-host --features wasm` tests (`wasm_executor.rs`).

#![cfg(not(feature = "wasm"))]

use std::fs;

use assert_cmd::Command;

fn stage_wasm_pkg(dir: &std::path::Path) {
    fs::create_dir_all(dir).unwrap();
    fs::write(
        dir.join("package.toml"),
        r#"
[package]
name = "wp"
version = "0.1.0"
interface = "greeter"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "plugin.wasm"
"#,
    )
    .unwrap();
    // A prebuilt component is accepted as-is; the no-wasm build doesn't validate.
    fs::write(dir.join("plugin.wasm"), b"\0asm\x0d\x00\x01\x00dummy").unwrap();
}

#[test]
fn pack_wasm_package_archives_with_a_skip_warning() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path().join("p");
    stage_wasm_pkg(&dir);
    let out = tmp.path().join("wp.fid");

    Command::cargo_bin("fidius")
        .unwrap()
        .args([
            "package",
            "pack",
            dir.to_str().unwrap(),
            "--output",
            out.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicates::str::contains("without wasm support"));

    assert!(out.exists(), "the .fid archive should be produced");
}

#[test]
fn precompile_without_wasm_feature_errors() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path().join("p");
    stage_wasm_pkg(&dir);

    Command::cargo_bin("fidius")
        .unwrap()
        .args(["package", "pack", dir.to_str().unwrap(), "--precompile"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("--features wasm"));
}

#[test]
fn inspect_renders_wasm_fields() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path().join("p");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("package.toml"),
        r#"
[package]
name = "wp"
version = "0.1.0"
interface = "greeter"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "plugin.wasm"
precompiled = "plugin.cwasm"
capabilities = ["clocks", "stdout"]
"#,
    )
    .unwrap();
    fs::write(dir.join("plugin.wasm"), b"x").unwrap();

    Command::cargo_bin("fidius")
        .unwrap()
        .args(["package", "inspect", dir.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("Runtime: wasm"))
        .stdout(predicates::str::contains("Component: plugin.wasm"))
        .stdout(predicates::str::contains(
            "Precompiled (.cwasm): plugin.cwasm",
        ))
        .stdout(predicates::str::contains("Capabilities: clocks, stdout"));
}

#[test]
fn sign_verify_and_tamper_wasm_package() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path().join("p");
    stage_wasm_pkg(&dir);

    let keybase = tmp.path().join("key");
    Command::cargo_bin("fidius")
        .unwrap()
        .args(["keygen", "--out", keybase.to_str().unwrap()])
        .assert()
        .success();
    let secret = format!("{}.secret", keybase.display());
    let public = format!("{}.public", keybase.display());

    // Sign the (artifact-agnostic) package, then verify.
    Command::cargo_bin("fidius")
        .unwrap()
        .args(["package", "sign", "--key", &secret, dir.to_str().unwrap()])
        .assert()
        .success();
    Command::cargo_bin("fidius")
        .unwrap()
        .args(["package", "verify", "--key", &public, dir.to_str().unwrap()])
        .assert()
        .success();

    // Tamper the component → verification must fail.
    fs::write(dir.join("plugin.wasm"), b"tampered component bytes").unwrap();
    Command::cargo_bin("fidius")
        .unwrap()
        .args(["package", "verify", "--key", &public, dir.to_str().unwrap()])
        .assert()
        .failure();
}
