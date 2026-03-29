//! End-to-end validation tests: signing, negative cases.

use std::path::PathBuf;
use std::process::Command;

use ed25519_dalek::{Signer, SigningKey};
use fides_host::{LoadError, PluginHandle, PluginHost};

/// Build the test plugin and return the directory containing the cdylib.
fn build_test_plugin() -> PathBuf {
    let manifest =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/test-plugin-smoke/Cargo.toml");

    let output = Command::new("cargo")
        .args(["build", "--manifest-path", manifest.to_str().unwrap()])
        .output()
        .expect("failed to run cargo build");

    assert!(
        output.status.success(),
        "failed to build test plugin: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/test-plugin-smoke/target/debug")
}

fn dylib_path(dir: &PathBuf) -> PathBuf {
    dir.join("libtest_plugin_smoke.dylib")
}

fn sign_dylib(dylib: &PathBuf, key: &SigningKey) {
    let content = std::fs::read(dylib).unwrap();
    let signature = key.sign(&content);
    let sig_path = dylib.with_extension("dylib.sig");
    std::fs::write(sig_path, signature.to_bytes()).unwrap();
}

fn cleanup_sig(dylib: &PathBuf) {
    let sig_path = dylib.with_extension("dylib.sig");
    let _ = std::fs::remove_file(sig_path);
}

#[test]
fn signed_plugin_loads_with_correct_key() {
    let plugin_dir = build_test_plugin();
    let dylib = dylib_path(&plugin_dir);

    let signing_key = SigningKey::from_bytes(&[10u8; 32]);
    let verifying_key = signing_key.verifying_key();

    sign_dylib(&dylib, &signing_key);

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .require_signature(true)
        .trusted_keys(&[verifying_key])
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    assert_eq!(loaded.info.name, "BasicCalculator");

    cleanup_sig(&dylib);
}

#[test]
fn signed_plugin_fails_with_wrong_key() {
    let plugin_dir = build_test_plugin();
    let dylib = dylib_path(&plugin_dir);

    let signing_key = SigningKey::from_bytes(&[11u8; 32]);
    let wrong_key = SigningKey::from_bytes(&[12u8; 32]).verifying_key();

    sign_dylib(&dylib, &signing_key);

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .require_signature(true)
        .trusted_keys(&[wrong_key])
        .build()
        .unwrap();

    let result = host.load("BasicCalculator");
    assert!(
        matches!(result, Err(LoadError::SignatureInvalid { .. })),
        "expected SignatureInvalid, got {:?}",
        result
    );

    cleanup_sig(&dylib);
}

#[test]
fn unsigned_plugin_fails_when_signature_required() {
    let plugin_dir = build_test_plugin();
    let dylib = dylib_path(&plugin_dir);

    // Make sure there's no .sig file
    cleanup_sig(&dylib);

    let key = SigningKey::from_bytes(&[13u8; 32]).verifying_key();

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .require_signature(true)
        .trusted_keys(&[key])
        .build()
        .unwrap();

    let result = host.load("BasicCalculator");
    assert!(
        matches!(result, Err(LoadError::SignatureRequired { .. })),
        "expected SignatureRequired, got {:?}",
        result
    );
}

#[test]
fn unsigned_plugin_loads_without_signature_requirement() {
    let plugin_dir = build_test_plugin();
    let dylib = dylib_path(&plugin_dir);
    cleanup_sig(&dylib);

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    let handle = PluginHandle::from_loaded(loaded);

    #[derive(serde::Serialize)]
    struct AddInput { a: i64, b: i64 }
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct AddOutput { result: i64 }

    let output: AddOutput = handle.call_method(0, &AddInput { a: 100, b: 200 }).unwrap();
    assert_eq!(output, AddOutput { result: 300 });
}
