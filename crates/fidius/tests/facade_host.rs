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
    // Host-function (plugin → host callback) channel surface.
    assert_exists::<fidius::HostImportInfo>();
    assert_exists::<fidius::host_import::HostImportInfo>();
}

#[test]
fn host_function_bind_gate_is_reachable_through_facade() {
    // `host_import::bind_host_interface` / `list_host_imports` are what the
    // generated `<Trait>Binding::bind` resolves through — guard that the
    // module path generated code names stays re-exported.
    #[allow(unused_imports)]
    use fidius::host_import::{bind_host_interface, list_host_imports};
    // The new LoadError variants are constructible and render.
    let err = fidius::LoadError::HostInterfaceVersionMismatch {
        interface: "CloacinaHost".into(),
        plugin_expects: 2,
        host_provides: 1,
    };
    let msg = format!("{err}");
    assert!(msg.contains("CloacinaHost") && msg.contains("v2") && msg.contains("v1"));
}
