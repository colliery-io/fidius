//! Integration test: load the test-plugin-smoke cdylib via fides-host API.

use std::path::PathBuf;
use std::process::Command;

use fides_host::{LoadError, PluginHandle, PluginHost, PluginInfo};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
struct AddInput {
    a: i64,
    b: i64,
}

#[derive(Deserialize, Debug, PartialEq)]
struct AddOutput {
    result: i64,
}

#[derive(Serialize)]
struct MulInput {
    a: i64,
    b: i64,
}

#[derive(Deserialize, Debug, PartialEq)]
struct MulOutput {
    result: i64,
}

#[test]
fn discover_finds_plugin() {
    let plugin_dir = build_test_plugin();

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .build()
        .unwrap();

    let plugins: Vec<PluginInfo> = host.discover().unwrap();
    let names: Vec<&str> = plugins.iter().map(|p| p.name.as_str()).collect();
    assert!(
        names.contains(&"BasicCalculator"),
        "expected BasicCalculator in {:?}",
        names
    );
}

#[test]
fn load_plugin_by_name() {
    let plugin_dir = build_test_plugin();

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    assert_eq!(loaded.info.name, "BasicCalculator");
    assert_eq!(loaded.info.interface_name, "Calculator");
}

#[test]
fn call_add_method_via_handle() {
    let plugin_dir = build_test_plugin();

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    let handle = PluginHandle::from_loaded(loaded);

    let input = AddInput { a: 3, b: 7 };
    let output: AddOutput = handle.call_method(0, &input).unwrap();
    assert_eq!(output, AddOutput { result: 10 });
}

#[test]
fn call_multiply_method_via_handle() {
    let plugin_dir = build_test_plugin();

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    let handle = PluginHandle::from_loaded(loaded);

    // multiply is the optional method (index 1)
    let input = MulInput { a: 4, b: 5 };
    let output: MulOutput = handle.call_method(1, &input).unwrap();
    assert_eq!(output, MulOutput { result: 20 });
}

#[test]
fn plugin_info_is_correct() {
    let plugin_dir = build_test_plugin();

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    let handle = PluginHandle::from_loaded(loaded);

    assert_eq!(handle.info().interface_name, "Calculator");
    assert_eq!(handle.info().name, "BasicCalculator");
    assert_eq!(handle.info().interface_version, 1);
    assert_eq!(
        handle.info().buffer_strategy,
        fides_core::descriptor::BufferStrategyKind::PluginAllocated
    );
}

#[test]
fn load_nonexistent_plugin_returns_not_found() {
    let plugin_dir = build_test_plugin();

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .build()
        .unwrap();

    let result = host.load("DoesNotExist");
    assert!(matches!(result, Err(LoadError::PluginNotFound { .. })));
}
