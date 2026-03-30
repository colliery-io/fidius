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

//! True end-to-end test: scaffold → package → build → sign → load → call.
//!
//! Everything is generated from scratch by the CLI. No pre-written fixtures.

use assert_cmd::Command;
use std::path::PathBuf;
use tempfile::TempDir;

fn fides_cmd() -> Command {
    Command::cargo_bin("fidius").unwrap()
}

/// Path to the workspace root's `fidius` facade crate (for local dep resolution).
fn workspace_fidius_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fidius")
}

#[test]
fn full_pipeline_scaffold_package_build_sign_load_call() {
    let tmp = TempDir::new().unwrap();
    let work_dir = tmp.path();

    eprintln!("\n=== FULL PIPELINE E2E TEST ===");
    eprintln!("Work dir: {}\n", work_dir.display());

    // ── Step 1: Scaffold interface crate ──────────────────────────────────
    eprintln!("Step 1: fidius init-interface test-api --trait Processor");
    let fidius_path = workspace_fidius_path();

    fides_cmd()
        .args([
            "init-interface",
            "test-api",
            "--trait",
            "Processor",
            "--path",
            work_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Overwrite the interface Cargo.toml to use local workspace paths
    let iface_cargo = work_dir.join("test-api/Cargo.toml");
    std::fs::write(
        &iface_cargo,
        format!(
            r#"[package]
name = "test-api"
version = "0.1.0"
edition = "2021"

[dependencies]
fidius = {{ path = "{}" }}
"#,
            fidius_path.display(),
        ),
    )
    .unwrap();

    eprintln!("  ✓ Interface crate scaffolded + patched for local deps\n");

    // ── Step 2: Scaffold plugin crate ─────────────────────────────────────
    eprintln!("Step 2: fidius init-plugin test-plugin --interface ./test-api --trait Processor");
    let iface_dir = work_dir.join("test-api");

    fides_cmd()
        .args([
            "init-plugin",
            "test-plugin",
            "--interface",
            iface_dir.to_str().unwrap(),
            "--trait",
            "Processor",
            "--path",
            work_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Overwrite the plugin Cargo.toml to use local workspace paths
    let plugin_cargo = work_dir.join("test-plugin/Cargo.toml");
    std::fs::write(
        &plugin_cargo,
        format!(
            r#"[package]
name = "test-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
test-api = {{ path = "{}" }}
fidius = {{ path = "{}" }}
"#,
            iface_dir.display(),
            fidius_path.display(),
        ),
    )
    .unwrap();

    eprintln!("  ✓ Plugin crate scaffolded + patched for local deps\n");

    // ── Step 3: Write package.toml ────────────────────────────────────────
    eprintln!("Step 3: Write package.toml");
    let package_toml = work_dir.join("test-plugin/package.toml");
    std::fs::write(
        &package_toml,
        r#"[package]
name = "test-processor"
version = "0.1.0"
interface = "test-api"
interface_version = 1

[metadata]
category = "testing"
description = "E2E test plugin"
"#,
    )
    .unwrap();

    eprintln!("  ✓ package.toml written\n");

    // ── Step 4: Generate keypair ──────────────────────────────────────────
    eprintln!("Step 4: fidius keygen --out testkey");
    let key_base = work_dir.join("testkey");

    fides_cmd()
        .args(["keygen", "--out", key_base.to_str().unwrap()])
        .assert()
        .success();

    let secret_key = format!("{}.secret", key_base.display());
    let public_key = format!("{}.public", key_base.display());

    eprintln!("  ✓ Keypair generated\n");

    // ── Step 5: Validate the package ──────────────────────────────────────
    eprintln!("Step 5: fidius package validate");
    let plugin_dir = work_dir.join("test-plugin");
    fides_cmd()
        .args(["package", "validate", plugin_dir.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("test-processor"));

    eprintln!("  ✓ Package validated\n");

    // ── Step 6: Build the package ─────────────────────────────────────────
    // Build before signing — cargo build may create/update Cargo.lock which
    // is part of the signed digest.
    eprintln!("Step 6: fidius package build --debug");
    fides_cmd()
        .args(["package", "build", plugin_dir.to_str().unwrap(), "--debug"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Build successful"));

    eprintln!("  ✓ Package built\n");

    // ── Step 7: Sign the package ─────────────────────────────────────────
    // Signs a digest of all source files (excluding target/ and .sig files).
    eprintln!("Step 7: fidius package sign");
    fides_cmd()
        .args([
            "package",
            "sign",
            "--key",
            &secret_key,
            plugin_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    eprintln!("  ✓ Package signed\n");

    // ── Step 8: Verify the signature ──────────────────────────────────────
    eprintln!("Step 8: fidius package verify");
    fides_cmd()
        .args([
            "package",
            "verify",
            "--key",
            &public_key,
            plugin_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    eprintln!("  ✓ Package signature verified\n");

    // ── Step 9: Load via PluginHost and call a method ─────────────────────
    eprintln!("Step 9: Sign dylib + load via PluginHost + call method");
    let dylib_dir = plugin_dir.join("target/debug");

    // Read the public key for PluginHost
    let key_bytes: [u8; 32] = std::fs::read(&public_key).unwrap().try_into().unwrap();
    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&key_bytes).unwrap();

    // Sign the compiled dylib too (PluginHost checks dylib signatures, not manifests)
    let dylib_name = if cfg!(target_os = "macos") {
        "libtest_plugin.dylib"
    } else if cfg!(target_os = "windows") {
        "test_plugin.dll"
    } else {
        "libtest_plugin.so"
    };
    let dylib_path = dylib_dir.join(dylib_name);

    // Sign the dylib for host loading
    fides_cmd()
        .args(["sign", "--key", &secret_key, dylib_path.to_str().unwrap()])
        .assert()
        .success();

    let host = fidius_host::PluginHost::builder()
        .search_path(&dylib_dir)
        .require_signature(true)
        .trusted_keys(&[verifying_key])
        .build()
        .unwrap();

    let loaded = host.load("MyProcessor").unwrap();
    assert_eq!(loaded.info.name, "MyProcessor");
    assert_eq!(loaded.info.interface_name, "Processor");
    let loaded_name = loaded.info.name.clone();
    let loaded_iface = loaded.info.interface_name.clone();

    let handle = fidius_host::PluginHandle::from_loaded(loaded);

    let input = "hello".to_string();
    let _input_bytes = fidius_core::wire::serialize(&input).unwrap();

    // call_method(0, ...) — the scaffolded process method
    let result: String = handle.call_method(0, &input).unwrap();
    assert_eq!(result, "processed: hello");

    eprintln!(
        "  ✓ Plugin loaded: {} (interface: {})",
        loaded_name, loaded_iface
    );
    eprintln!("  ✓ call_method(0, \"hello\") returned: {:?}", result);
    eprintln!("\n=== ALL STEPS PASSED ===\n");
}
