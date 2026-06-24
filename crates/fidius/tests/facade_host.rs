// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// FIDIUS-I-0033 Phase 5: facade tests for the `host` feature surface.
#![cfg(feature = "host")]

use fidius::PluginHost;

#[test]
fn plugin_host_builds_through_facade() {
    // An empty host over a real (plugin-less) search dir builds cleanly — exercises
    // the re-exported builder + loader end-to-end, not just the type name.
    let host = PluginHost::builder()
        .search_path(std::env::temp_dir())
        .build();
    assert!(
        host.is_ok(),
        "facade PluginHost builder should build: {:?}",
        host.err()
    );
}

#[test]
fn host_types_are_reexported() {
    // Guard the host re-export surface consumers name.
    fn assert_exists<T>() {}
    assert_exists::<fidius::CallError>();
    assert_exists::<fidius::LoadError>();
    assert_exists::<fidius::PluginHandle>();
    assert_exists::<fidius::PluginInfo>();
    assert_exists::<fidius::PluginRuntimeKind>();
    assert_exists::<fidius::PluginHostBuilder>();
}
