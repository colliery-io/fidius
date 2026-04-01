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

//! Integration test: load the test-plugin-smoke cdylib via fidius-host API.

use std::path::PathBuf;
use std::process::Command;

use fidius_host::{LoadError, PluginHandle, PluginHost, PluginInfo};
use serde::{Deserialize, Serialize};

/// Build the test plugin and return the directory containing the cdylib.
fn build_test_plugin() -> PathBuf {
    let manifest =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/test-plugin-smoke/Cargo.toml");

    let mut args = vec!["build", "--manifest-path", manifest.to_str().unwrap()];
    if !cfg!(debug_assertions) {
        args.push("--release");
    }

    let output = Command::new("cargo")
        .args(&args)
        .output()
        .expect("failed to run cargo build");

    assert!(
        output.status.success(),
        "failed to build test plugin: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/test-plugin-smoke/target")
        .join(profile)
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

    let input = (AddInput { a: 3, b: 7 },);
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

    // multiply is the optional method (index 3: add=0, add_direct=1, version=2, multiply=3)
    let input = (MulInput { a: 4, b: 5 },);
    let output: MulOutput = handle.call_method(3, &input).unwrap();
    assert_eq!(output, MulOutput { result: 20 });
}

#[test]
fn call_multi_arg_add_direct() {
    let plugin_dir = build_test_plugin();

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    let handle = PluginHandle::from_loaded(loaded);

    // add_direct takes two i64 args directly (index 1)
    let output: i64 = handle.call_method(1, &(100i64, 200i64)).unwrap();
    assert_eq!(output, 300);
}

#[test]
fn call_zero_arg_version() {
    let plugin_dir = build_test_plugin();

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    let handle = PluginHandle::from_loaded(loaded);

    // version takes zero args (index 2)
    let output: String = handle.call_method(2, &()).unwrap();
    assert_eq!(output, "1.0.0");
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
        fidius_core::descriptor::BufferStrategyKind::PluginAllocated
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

#[test]
fn out_of_bounds_vtable_index_returns_error() {
    let plugin_dir = build_test_plugin();

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    let handle = PluginHandle::from_loaded(loaded);

    #[derive(serde::Serialize)]
    struct Dummy;

    // Index 99 is way past the vtable — should return NotImplemented
    let result = handle.call_method::<Dummy, String>(99, &Dummy);
    assert!(
        matches!(result, Err(fidius_host::CallError::NotImplemented { .. })),
        "expected NotImplemented for OOB index, got {:?}",
        result
    );
}

#[test]
fn has_capability_returns_false_for_high_bits() {
    let plugin_dir = build_test_plugin();

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    let handle = PluginHandle::from_loaded(loaded);

    // Bit 63 should return false, not panic
    assert!(!handle.has_capability(63));
    // Bit 64+ should also return false (was a panic before)
    assert!(!handle.has_capability(64));
    assert!(!handle.has_capability(100));
}
