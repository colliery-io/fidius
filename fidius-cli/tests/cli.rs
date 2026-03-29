//! CLI integration tests using assert_cmd.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use std::process;
use tempfile::TempDir;

fn fidius_cmd() -> Command {
    Command::cargo_bin("fidius").unwrap()
}

fn build_test_plugin() -> PathBuf {
    let manifest =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/test-plugin-smoke/Cargo.toml");

    let output = process::Command::new("cargo")
        .args(["build", "--manifest-path", manifest.to_str().unwrap()])
        .output()
        .expect("failed to run cargo build");

    assert!(output.status.success());

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/test-plugin-smoke/target/debug")
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
    let plugin_dir = build_test_plugin();
    let dylib = plugin_dir.join("libtest_plugin_smoke.dylib");

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
    let _ = std::fs::remove_file(dylib.with_extension("dylib.sig"));
}

#[test]
fn inspect_shows_plugin_info() {
    let plugin_dir = build_test_plugin();
    let dylib = plugin_dir.join("libtest_plugin_smoke.dylib");

    fidius_cmd()
        .args(["inspect", dylib.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("BasicCalculator"))
        .stdout(predicate::str::contains("Calculator"))
        .stdout(predicate::str::contains("PluginAllocated"));
}
