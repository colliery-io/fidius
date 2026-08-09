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

//! Behavior of the host-function channel when the host never binds, and
//! backward compatibility for plugins that use no host interface.
//!
//! Separate test binary from `host_functions_e2e.rs` on purpose: the bound
//! table is a per-library (per-process) global, and these tests need a
//! process in which `TestHost` was **never** bound.

use std::path::PathBuf;

use fidius_host::PluginHandle;
use fidius_test::dylib_fixture;

use test_plugin_hostcall::__fidius_Deferrable::{METHOD_HOST_BOUND, METHOD_TRY_RELEASE};

fn hostcall_plugin_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/test-plugin-hostcall")
}

fn smoke_plugin_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/test-plugin-smoke")
}

#[test]
fn unbound_host_interface_is_a_typed_error_not_a_crash() {
    let fixture = dylib_fixture(hostcall_plugin_dir()).build();
    let lib = fidius_host::loader::load_library(fixture.dylib_path()).expect("load");
    let plugin = lib
        .plugins
        .into_iter()
        .find(|p| p.info.name == "DeferrablePlugin")
        .expect("plugin");
    let handle = PluginHandle::from_loaded(plugin);

    // The plugin loads and runs with no host bound.
    let bound: bool = handle.call_method(METHOD_HOST_BOUND, &()).expect("call");
    assert!(!bound, "host never bound TestHost in this process");

    // A host-function call from plugin code surfaces the typed NotBound
    // error — never a null-pointer crash or mis-dispatch.
    let outcome: String = handle
        .call_method(METHOD_TRY_RELEASE, &("task-x".to_string(),))
        .expect("plugin handles the unbound case");
    assert!(
        outcome.contains("not-bound") && outcome.contains("NotBound"),
        "expected a typed NotBound error, got: {outcome}"
    );
}

#[test]
fn plugin_without_host_interfaces_loads_and_runs_unchanged() {
    // test-plugin-smoke declares no host interface: its import registry is
    // empty, binding is a clean no-op, and calls behave exactly as before
    // the host-callback channel existed.
    let fixture = dylib_fixture(smoke_plugin_dir()).build();
    let lib = fidius_host::loader::load_library(fixture.dylib_path()).expect("load");

    let imports = lib.host_imports().expect("read imports");
    assert!(imports.is_empty(), "smoke plugin declares no host imports");

    let bound = fidius_host::host_import::bind_host_interface(
        &lib.library,
        "TestHost",
        test_plugin_hostcall::TestHostBinding::INTERFACE_HASH,
        test_plugin_hostcall::TestHostBinding::INTERFACE_VERSION,
        || panic!("no table should be built for a plugin with no imports"),
    )
    .expect("binding against a no-import plugin is not an error");
    assert!(!bound);

    // The plugin still works.
    let plugin = lib
        .plugins
        .into_iter()
        .find(|p| p.info.name == "BasicCalculator")
        .expect("calculator");
    let handle = PluginHandle::from_loaded(plugin);
    let sum: i64 = handle
        .call_method(1, &(2i64, 3i64))
        .expect("add_direct unchanged");
    assert_eq!(sum, 5);
}
