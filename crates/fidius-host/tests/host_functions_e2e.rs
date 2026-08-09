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

//! End-to-end tests for the plugin → host callback channel over a real,
//! dynamically loaded cdylib — including the reentrant path
//! (host → plugin → host).
//!
//! The bound host-function table is a per-library global, so this file
//! binds exactly once (in `bound_plugin()`) and every test here runs
//! against the bound state. The "host never bound" behavior lives in
//! `host_functions_unbound.rs`, which is a separate test binary and
//! therefore a separate process with its own dylib load.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use fidius_core::host_ffi;
use fidius_host::{CallError, LoadError, PluginHandle};
use fidius_test::dylib_fixture;

use fidius_core::PluginError;
use test_plugin_hostcall::__fidius_Deferrable::{
    METHOD_HOST_BOUND, METHOD_OBSERVE_HOST_PANIC, METHOD_PANIC_AFTER_CALLBACK, METHOD_RUN_DEFERRED,
    METHOD_VALUE_FROM_THREAD,
};
use test_plugin_hostcall::{TestHost, TestHostBinding};

fn plugin_source_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/test-plugin-hostcall")
}

/// Recording host implementation. Thread-safe; asserts the documented
/// callback contract (depth counter) from inside each host function.
#[derive(Default)]
struct RecordingHost {
    /// (entry, thread the callback arrived on). Tests key entries by task id
    /// so parallel tests don't interfere.
    calls: Mutex<Vec<(String, std::thread::ThreadId)>>,
    reclaims: AtomicU32,
}

impl RecordingHost {
    fn record(&self, entry: impl Into<String>) {
        // Taking this mutex INSIDE a callback is fine — the host never holds
        // it across a plugin call. That is exactly the documented lock rule.
        self.calls
            .lock()
            .unwrap()
            .push((entry.into(), std::thread::current().id()));
    }
    fn calls(&self) -> Vec<(String, std::thread::ThreadId)> {
        self.calls.lock().unwrap().clone()
    }
}

impl TestHost for RecordingHost {
    fn release_slot(&self, task_id: String) -> Result<(), PluginError> {
        assert_eq!(
            host_ffi::host_callback_depth(),
            1,
            "host function runs at callback depth 1"
        );
        if task_id == "reject-me" {
            return Err(PluginError::new("SLOT_REJECTED", "release refused by host"));
        }
        self.record(format!("release:{task_id}"));
        Ok(())
    }

    fn reclaim_slot(&self, task_id: String) -> Result<u32, PluginError> {
        self.record(format!("reclaim:{task_id}"));
        Ok(self.reclaims.fetch_add(1, Ordering::SeqCst) + 1)
    }

    fn get_value(&self, key: String) -> String {
        self.record(format!("get:{key}"));
        format!("value-of-{key}")
    }

    fn panicky(&self) -> u32 {
        panic!("host function exploded on purpose");
    }
}

/// One shared loaded-plugin + bound-host state for the whole test binary.
fn bound_plugin() -> &'static (PluginHandle, Arc<RecordingHost>) {
    static STATE: OnceLock<(PluginHandle, Arc<RecordingHost>)> = OnceLock::new();
    STATE.get_or_init(|| {
        let fixture = dylib_fixture(plugin_source_dir()).build();
        let lib = fidius_host::loader::load_library(fixture.dylib_path()).expect("load");

        // Discovery: the plugin advertises the host interface it was built
        // against, with the version + hash for the load-time gate.
        let imports = lib.host_imports().expect("read imports");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].interface_name, "TestHost");
        assert_eq!(
            imports[0].interface_version,
            TestHostBinding::INTERFACE_VERSION
        );
        assert_eq!(imports[0].interface_hash, TestHostBinding::INTERFACE_HASH);

        let host = Arc::new(RecordingHost::default());
        let bound = TestHostBinding::bind(&lib, host.clone() as Arc<dyn TestHost>)
            .expect("bind host interface");
        assert!(bound, "plugin imports TestHost, so bind installs the table");

        let plugin = lib
            .plugins
            .into_iter()
            .find(|p| p.info.name == "DeferrablePlugin")
            .expect("DeferrablePlugin in registry");
        (PluginHandle::from_loaded(plugin), host)
    })
}

#[test]
fn reentrant_host_plugin_host_roundtrip() {
    let (handle, host) = bound_plugin();
    let my_thread = std::thread::current().id();

    // host → plugin (run_deferred) → host (release/get/reclaim), all while
    // the plugin call is live on this stack.
    let out: String = handle
        .call_method(METHOD_RUN_DEFERRED, &("task-1".to_string(),))
        .expect("run_deferred");

    assert!(
        out.starts_with("deferred:task-1:cond=value-of-cond:task-1:reclaims="),
        "unexpected roundtrip output: {out}"
    );
    let calls = host.calls();
    for expected in ["release:task-1", "get:cond:task-1", "reclaim:task-1"] {
        let (_, thread) = calls
            .iter()
            .find(|(entry, _)| entry == expected)
            .unwrap_or_else(|| panic!("missing callback {expected}: {calls:?}"));
        // Same-stack reentrancy: the callbacks arrived on the host thread
        // that is currently inside the plugin call.
        assert_eq!(*thread, my_thread, "callback {expected} on the same stack");
    }

    // Depth counter unwound after the callback chain.
    assert_eq!(host_ffi::host_callback_depth(), 0);
}

#[test]
fn host_error_arrives_typed_in_the_plugin() {
    let (handle, _) = bound_plugin();
    // The host rejects "reject-me"; the plugin wraps the typed
    // HostCallError::Host(PluginError) it received into its own PluginError.
    let err = handle
        .call_method::<_, String>(METHOD_RUN_DEFERRED, &("reject-me".to_string(),))
        .expect_err("host rejected the release");
    match err {
        CallError::Plugin(pe) => {
            assert_eq!(pe.code, "RELEASE_FAILED");
            assert!(
                pe.message.contains("SLOT_REJECTED"),
                "plugin saw the host's typed error: {}",
                pe.message
            );
        }
        other => panic!("expected CallError::Plugin, got {other:?}"),
    }
}

#[test]
fn host_panic_surfaces_as_typed_error_not_unwinding() {
    let (handle, _) = bound_plugin();
    let observed: String = handle
        .call_method(METHOD_OBSERVE_HOST_PANIC, &())
        .expect("plugin survives a host panic");
    assert!(
        observed.contains("HostPanic") && observed.contains("exploded on purpose"),
        "plugin observed a typed HostPanic error: {observed}"
    );
}

#[test]
fn plugin_panic_after_callback_surfaces_as_call_error() {
    let (handle, _) = bound_plugin();
    let err = handle
        .call_method::<_, String>(METHOD_PANIC_AFTER_CALLBACK, &("task-p".to_string(),))
        .expect_err("plugin panicked");
    match err {
        CallError::Panic(msg) => assert!(msg.contains("plugin panicked after callback")),
        other => panic!("expected CallError::Panic, got {other:?}"),
    }
}

#[test]
fn callbacks_work_from_plugin_spawned_threads() {
    let (handle, _) = bound_plugin();
    let v: String = handle
        .call_method(METHOD_VALUE_FROM_THREAD, &("threaded".to_string(),))
        .expect("value_from_thread");
    assert_eq!(v, "value-of-threaded");
}

#[test]
fn plugin_reports_interface_bound() {
    let (handle, _) = bound_plugin();
    let bound: bool = handle
        .call_method(METHOD_HOST_BOUND, &())
        .expect("host_bound");
    assert!(bound);
}

#[test]
fn second_bind_fails_loudly_without_disturbing_the_first() {
    let (handle, _) = bound_plugin();
    let fixture = dylib_fixture(plugin_source_dir()).build();
    let lib = fidius_host::loader::load_library(fixture.dylib_path()).expect("load");
    let another = Arc::new(RecordingHost::default());
    let err = TestHostBinding::bind(&lib, another as Arc<dyn TestHost>)
        .expect_err("binding twice is refused");
    match err {
        LoadError::HostBindFailed {
            interface, code, ..
        } => {
            assert_eq!(interface, "TestHost");
            assert_eq!(code, host_ffi::BIND_ERR_ALREADY_BOUND);
        }
        other => panic!("expected HostBindFailed, got {other:?}"),
    }
    // The original binding still works.
    let bound: bool = handle
        .call_method(METHOD_HOST_BOUND, &())
        .expect("host_bound");
    assert!(bound);
}

#[test]
fn version_mismatch_fails_at_bind_and_never_builds_a_table() {
    let (_, _) = bound_plugin(); // ensure fixture built
    let fixture = dylib_fixture(plugin_source_dir()).build();
    let lib = fidius_host::loader::load_library(fixture.dylib_path()).expect("load");

    // Simulate a host built against host-interface v2 loading a v1 plugin:
    // same name + hash constants except the version. The gate must fail
    // BEFORE the table builder runs.
    let err = fidius_host::host_import::bind_host_interface(
        &lib.library,
        "TestHost",
        TestHostBinding::INTERFACE_HASH,
        TestHostBinding::INTERFACE_VERSION + 1,
        || panic!("table must not be built when the version gate fails"),
    )
    .expect_err("version mismatch");
    match err {
        LoadError::HostInterfaceVersionMismatch {
            interface,
            plugin_expects,
            host_provides,
        } => {
            assert_eq!(interface, "TestHost");
            assert_eq!(plugin_expects, TestHostBinding::INTERFACE_VERSION);
            assert_eq!(host_provides, TestHostBinding::INTERFACE_VERSION + 1);
        }
        other => panic!("expected HostInterfaceVersionMismatch, got {other:?}"),
    }
}

#[test]
fn signature_drift_fails_at_bind_and_never_builds_a_table() {
    let (_, _) = bound_plugin();
    let fixture = dylib_fixture(plugin_source_dir()).build();
    let lib = fidius_host::loader::load_library(fixture.dylib_path()).expect("load");

    // Same declared version, drifted signatures (different hash) — the
    // protection that keeps positional bincode from ever mis-dispatching.
    let err = fidius_host::host_import::bind_host_interface(
        &lib.library,
        "TestHost",
        TestHostBinding::INTERFACE_HASH ^ 0xdead_beef,
        TestHostBinding::INTERFACE_VERSION,
        || panic!("table must not be built when the hash gate fails"),
    )
    .expect_err("hash mismatch");
    match err {
        LoadError::HostInterfaceHashMismatch { interface, .. } => {
            assert_eq!(interface, "TestHost");
        }
        other => panic!("expected HostInterfaceHashMismatch, got {other:?}"),
    }
}

#[test]
fn binding_an_undeclared_interface_is_a_clean_no() {
    let (_, _) = bound_plugin();
    let fixture = dylib_fixture(plugin_source_dir()).build();
    let lib = fidius_host::loader::load_library(fixture.dylib_path()).expect("load");

    let bound = fidius_host::host_import::bind_host_interface(
        &lib.library,
        "SomeOtherHostInterface",
        1,
        1,
        || panic!("table must not be built for an undeclared import"),
    )
    .expect("no error for an undeclared interface");
    assert!(!bound, "plugin does not import SomeOtherHostInterface");
}
