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
//!
//! Exercises the generated `CalculatorClient` typed proxy for method-call
//! tests and the raw `PluginHandle::call_method` path for out-of-bounds /
//! capability / info assertions where the Client abstracts them away.

use std::path::{Path, PathBuf};

use fidius_host::{LoadError, PluginHandle, PluginHost, PluginInfo};
use fidius_test::dylib_fixture;
use test_plugin_smoke::{
    AddInput, AddOutput, ArenaEchoClient, CalculatorClient, MulInput, MulOutput,
};

fn plugin_source_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/test-plugin-smoke")
}

/// Directory containing the cached-built test plugin cdylib.
fn plugin_dir() -> &'static Path {
    // Process-wide cache: first call builds, subsequent calls return the
    // same path without re-invoking cargo. We leak a PathBuf to get a
    // &'static borrow for tests that want one.
    static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        dylib_fixture(plugin_source_dir())
            .build()
            .dir()
            .to_path_buf()
    })
}

/// Build a client from the built+loaded plugin. Used by most method-call tests.
fn client() -> CalculatorClient {
    let host = PluginHost::builder()
        .search_path(plugin_dir())
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    let handle = PluginHandle::from_loaded(loaded);
    CalculatorClient::from_handle(handle)
}

#[test]
fn discover_finds_plugin() {
    let host = PluginHost::builder()
        .search_path(plugin_dir())
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
    let host = PluginHost::builder()
        .search_path(plugin_dir())
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    assert_eq!(loaded.info.name, "BasicCalculator");
    assert_eq!(loaded.info.interface_name, "Calculator");
}

#[test]
fn call_add_method_via_client() {
    let client = client();
    let output = client.add(&AddInput { a: 3, b: 7 }).unwrap();
    assert_eq!(output, AddOutput { result: 10 });
}

#[test]
fn call_multiply_method_via_client() {
    let client = client();
    // multiply is the optional method — Client checks capability internally
    let output = client.multiply(&MulInput { a: 4, b: 5 }).unwrap();
    assert_eq!(output, MulOutput { result: 20 });
}

#[test]
fn call_multi_arg_add_direct_via_client() {
    let client = client();
    let output = client.add_direct(&100i64, &200i64).unwrap();
    assert_eq!(output, 300);
}

#[test]
fn call_zero_arg_version_via_client() {
    let client = client();
    let output = client.version().unwrap();
    assert_eq!(output, "1.0.0");
}

#[test]
fn plugin_info_is_correct() {
    let host = PluginHost::builder()
        .search_path(plugin_dir())
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
    let host = PluginHost::builder()
        .search_path(plugin_dir())
        .build()
        .unwrap();

    let result = host.load("DoesNotExist");
    assert!(matches!(result, Err(LoadError::PluginNotFound { .. })));
}

#[test]
fn out_of_bounds_vtable_index_returns_error() {
    let host = PluginHost::builder()
        .search_path(plugin_dir())
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    let handle = PluginHandle::from_loaded(loaded);

    #[derive(serde::Serialize)]
    struct Dummy;

    // Index 99 is way past the vtable — should return InvalidMethodIndex
    let result = handle.call_method::<Dummy, String>(99, &Dummy);
    assert!(
        matches!(
            result,
            Err(fidius_host::CallError::InvalidMethodIndex { index: 99, .. })
        ),
        "expected InvalidMethodIndex for OOB index, got {:?}",
        result
    );
}

#[test]
fn arena_plugin_loads_and_round_trips() {
    let host = PluginHost::builder()
        .search_path(plugin_dir())
        .build()
        .unwrap();

    let loaded = host.load("ArenaEchoer").unwrap();
    assert_eq!(loaded.info.interface_name, "ArenaEcho");
    // Arena strategy has no free_buffer (host writes into its own arena).
    assert!(loaded.free_buffer.is_none());

    let handle = PluginHandle::from_loaded(loaded);
    let client = ArenaEchoClient::from_handle(handle);

    let out = client.echo(&"hello".to_string()).unwrap();
    assert_eq!(out, "arena-echo: hello");
}

#[test]
fn arena_plugin_grows_buffer_on_too_small_retry() {
    // Force the retry path by invoking the Arena plugin with an input that
    // produces an output larger than the initial arena. We can't directly
    // shrink the initial arena here (DEFAULT_ARENA_CAPACITY is 4KB and the
    // pool reuses previous buffers), so we construct an input that forces
    // the output past any reasonable initial size.
    let host = PluginHost::builder()
        .search_path(plugin_dir())
        .build()
        .unwrap();

    let loaded = host.load("ArenaEchoer").unwrap();
    let handle = PluginHandle::from_loaded(loaded);
    let client = ArenaEchoClient::from_handle(handle);

    // 10 KB of 'a' — wraps to "arena-echo: aaaa..." which exceeds the
    // 4KB default. The host should grow the arena and retry, returning
    // the full output.
    let big_input = "a".repeat(10_000);
    let out = client.echo(&big_input).unwrap();
    assert_eq!(out.len(), "arena-echo: ".len() + big_input.len());
    assert!(out.starts_with("arena-echo: aaa"));
}

#[test]
fn trait_and_method_metadata_readable_through_handle() {
    let host = PluginHost::builder()
        .search_path(plugin_dir())
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    let method_count = loaded.method_count;
    let handle = PluginHandle::from_loaded(loaded);

    let trait_meta = handle.trait_metadata();
    assert_eq!(
        trait_meta,
        vec![("kind", "calculator"), ("stability", "stable")],
    );

    // add (index 0), add_direct (index 1), multiply (index 3) have effect=pure;
    // version (index 2) has no metadata.
    assert_eq!(handle.method_metadata(0), vec![("effect", "pure")]);
    assert_eq!(handle.method_metadata(1), vec![("effect", "pure")]);
    assert_eq!(handle.method_metadata(2), Vec::<(&str, &str)>::new());
    assert_eq!(handle.method_metadata(3), vec![("effect", "pure")]);

    // Out-of-range index returns empty vec, not panic.
    assert!(handle.method_metadata(method_count).is_empty());
    assert!(handle.method_metadata(999).is_empty());
}

#[test]
fn has_capability_returns_false_for_high_bits() {
    let host = PluginHost::builder()
        .search_path(plugin_dir())
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
