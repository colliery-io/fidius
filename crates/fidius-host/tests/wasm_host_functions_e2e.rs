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

//! Host functions over **WASM** end to end: a component whose methods call
//! back into the host through the `fidius:host-call` import — the wasm
//! variant of `host_functions_e2e.rs`. Covers the reentrant round-trip, the
//! per-call version/hash gate (typed mismatch errors, never a dispatch),
//! typed host errors, host panics, the unbound case, and once-only binding.
//!
//! Unlike the dylib channel (per-library global cell), each `load_wasm`
//! creates an executor with its own empty table registry — so every test
//! binds (or doesn't) its own fresh instance.

#![cfg(feature = "wasm")]
#![allow(unexpected_cfgs)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use fidius_core::host_ffi::{self, HostFunctionTable};
use fidius_core::PluginError;
use fidius_host::{LoadError, PluginHandle, PluginHost};
// Linked for its in-process descriptor registry (the wrong-backend test
// grabs a cdylib-backed handle from it).
use test_plugin_smoke as _;

// Same signatures as the fixture → same interface hashes + export name.
#[fidius_macro::host_interface(version = 1, crate = "fidius_core")]
pub trait WasmSlotHost: Send + Sync {
    fn release_slot(&self, task_id: String) -> Result<(), PluginError>;
    fn reclaim_slot(&self, task_id: String) -> Result<u32, PluginError>;
    fn get_value(&self, key: String) -> String;
    fn panicky(&self) -> u32;
}

#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait WasmDeferrable: Send + Sync {
    fn run(&self, task_id: String) -> String;
    fn host_bound(&self) -> bool;
    fn try_release(&self, task_id: String) -> String;
    fn observe_host_panic(&self) -> String;
}

fn component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/wasm-fixtures/hostcall");
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("run `cargo build --target wasm32-wasip2` (see T-0094 for the toolchain)");
        assert!(status.success(), "hostcall wasm build failed");
        let art = fixture.join("target/wasm32-wasip2/release/hostcall_guest.wasm");
        std::fs::read(&art).unwrap_or_else(|e| panic!("read {}: {e}", art.display()))
    })
}

fn stage_pkg(root: &std::path::Path) {
    let dir = root.join("hostcall-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        r#"
[package]
name = "hostcall-pkg"
version = "0.1.0"
interface = "wasm-deferrable"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "hostcall_guest.wasm"
"#,
    )
    .unwrap();
    std::fs::write(dir.join("hostcall_guest.wasm"), component()).unwrap();
}

/// Load a fresh handle over the fixture component (own table registry).
fn load_handle(tmp: &tempfile::TempDir) -> PluginHandle {
    stage_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    host.load_wasm(
        "hostcall-pkg",
        &__fidius_WasmDeferrable::WasmDeferrable_WASM_DESCRIPTOR,
    )
    .expect("load_wasm")
}

/// Recording host implementation shared by the tests.
#[derive(Default)]
struct RecordingHost {
    calls: Mutex<Vec<String>>,
    reclaims: AtomicU32,
}

impl WasmSlotHost for RecordingHost {
    fn release_slot(&self, task_id: String) -> Result<(), PluginError> {
        assert_eq!(host_ffi::host_callback_depth(), 1);
        if task_id == "reject-me" {
            return Err(PluginError::new("SLOT_REJECTED", "release refused by host"));
        }
        self.calls
            .lock()
            .unwrap()
            .push(format!("release:{task_id}"));
        Ok(())
    }
    fn reclaim_slot(&self, task_id: String) -> Result<u32, PluginError> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("reclaim:{task_id}"));
        Ok(self.reclaims.fetch_add(1, Ordering::SeqCst) + 1)
    }
    fn get_value(&self, key: String) -> String {
        self.calls.lock().unwrap().push(format!("get:{key}"));
        format!("value-of-{key}")
    }
    fn panicky(&self) -> u32 {
        panic!("host function exploded on purpose");
    }
}

/// Build a process-lifetime table for a recording host.
fn table(host: Arc<RecordingHost>) -> *const HostFunctionTable {
    __fidius_host_WasmSlotHost::__fidius_build_host_table(host as Arc<dyn WasmSlotHost>)
}

#[test]
fn wasm_reentrant_host_plugin_host_roundtrip() {
    let tmp = tempfile::TempDir::new().unwrap();
    let handle = load_handle(&tmp);
    let host = Arc::new(RecordingHost::default());
    // SAFETY: `table` builds a fresh, leaked (process-lifetime) table.
    unsafe { handle.bind_wasm_host_table(table(host.clone())) }.expect("bind");

    let bound: bool = handle
        .call_method(__fidius_WasmDeferrable::METHOD_HOST_BOUND, &())
        .expect("host_bound");
    assert!(bound, "probe reports bound after bind");

    let out: String = handle
        .call_method(
            __fidius_WasmDeferrable::METHOD_RUN,
            &("task-1".to_string(),),
        )
        .expect("run");
    assert_eq!(out, "deferred:task-1:cond=value-of-cond:task-1:reclaims=1");
    let calls = host.calls.lock().unwrap().clone();
    assert_eq!(
        calls,
        vec!["release:task-1", "get:cond:task-1", "reclaim:task-1"]
    );
    assert_eq!(host_ffi::host_callback_depth(), 0);
}

#[test]
fn wasm_host_error_arrives_typed() {
    let tmp = tempfile::TempDir::new().unwrap();
    let handle = load_handle(&tmp);
    // SAFETY: fresh, leaked (process-lifetime) table.
    unsafe { handle.bind_wasm_host_table(table(Arc::new(RecordingHost::default()))) }
        .expect("bind");

    let out: String = handle
        .call_method(
            __fidius_WasmDeferrable::METHOD_RUN,
            &("reject-me".to_string(),),
        )
        .expect("run returns a formatted error, not a failure");
    assert!(
        out.contains("release-error") && out.contains("SLOT_REJECTED"),
        "guest observed the host's typed PluginError: {out}"
    );
}

#[test]
fn wasm_host_panic_surfaces_as_typed_error() {
    let tmp = tempfile::TempDir::new().unwrap();
    let handle = load_handle(&tmp);
    // SAFETY: fresh, leaked (process-lifetime) table.
    unsafe { handle.bind_wasm_host_table(table(Arc::new(RecordingHost::default()))) }
        .expect("bind");

    let out: String = handle
        .call_method(__fidius_WasmDeferrable::METHOD_OBSERVE_HOST_PANIC, &())
        .expect("guest survives a host panic");
    assert!(
        out.contains("HostPanic") && out.contains("exploded on purpose"),
        "guest observed a typed HostPanic: {out}"
    );
}

#[test]
fn wasm_unbound_is_a_typed_error_not_a_trap() {
    let tmp = tempfile::TempDir::new().unwrap();
    let handle = load_handle(&tmp);
    // No bind at all on this executor.
    let bound: bool = handle
        .call_method(__fidius_WasmDeferrable::METHOD_HOST_BOUND, &())
        .expect("host_bound");
    assert!(!bound);
    let out: String = handle
        .call_method(
            __fidius_WasmDeferrable::METHOD_TRY_RELEASE,
            &("t".to_string(),),
        )
        .expect("guest handles the unbound case");
    assert!(
        out.contains("not-bound-or-mismatch") && out.contains("NotBound"),
        "typed NotBound: {out}"
    );
}

#[test]
fn wasm_version_mismatch_is_typed_and_never_dispatches() {
    let tmp = tempfile::TempDir::new().unwrap();
    let handle = load_handle(&tmp);
    let host = Arc::new(RecordingHost::default());
    // Simulate a host built against v2 of the interface: same table, bumped
    // version. (The plugin was built expecting v1.)
    let good = table(host.clone());
    let mut doctored = unsafe { std::ptr::read(good) };
    doctored.interface_version += 1;
    // SAFETY: the doctored copy is leaked too (process lifetime).
    unsafe { handle.bind_wasm_host_table(Box::into_raw(Box::new(doctored))) }
        .expect("bind the v2 table");

    let bound: bool = handle
        .call_method(__fidius_WasmDeferrable::METHOD_HOST_BOUND, &())
        .expect("host_bound");
    assert!(!bound, "probe fails the version gate");
    let out: String = handle
        .call_method(
            __fidius_WasmDeferrable::METHOD_TRY_RELEASE,
            &("t".to_string(),),
        )
        .expect("guest observes the typed mismatch");
    assert!(
        out.contains("VersionMismatch")
            && out.contains("plugin_expects: 1")
            && out.contains("host_provides: 2"),
        "typed VersionMismatch with both revisions: {out}"
    );
    // Nothing was ever dispatched into the host implementation.
    assert!(host.calls.lock().unwrap().is_empty());
}

#[test]
fn wasm_hash_mismatch_is_typed_and_never_dispatches() {
    let tmp = tempfile::TempDir::new().unwrap();
    let handle = load_handle(&tmp);
    let host = Arc::new(RecordingHost::default());
    let good = table(host.clone());
    let mut doctored = unsafe { std::ptr::read(good) };
    doctored.interface_hash ^= 0xdead_beef;
    // SAFETY: the doctored copy is leaked too (process lifetime).
    unsafe { handle.bind_wasm_host_table(Box::into_raw(Box::new(doctored))) }
        .expect("bind the drifted table");

    let out: String = handle
        .call_method(
            __fidius_WasmDeferrable::METHOD_TRY_RELEASE,
            &("t".to_string(),),
        )
        .expect("guest observes the typed mismatch");
    assert!(out.contains("HashMismatch"), "typed HashMismatch: {out}");
    assert!(host.calls.lock().unwrap().is_empty());
}

#[test]
fn wasm_second_bind_is_refused() {
    let tmp = tempfile::TempDir::new().unwrap();
    let handle = load_handle(&tmp);
    // SAFETY: fresh, leaked (process-lifetime) tables.
    unsafe { handle.bind_wasm_host_table(table(Arc::new(RecordingHost::default()))) }
        .expect("first bind");
    let err = unsafe { handle.bind_wasm_host_table(table(Arc::new(RecordingHost::default()))) }
        .expect_err("second bind refused");
    match err {
        LoadError::HostBindFailed {
            interface, code, ..
        } => {
            assert_eq!(interface, "WasmSlotHost");
            assert_eq!(code, host_ffi::BIND_ERR_ALREADY_BOUND);
        }
        other => panic!("expected HostBindFailed, got {other:?}"),
    }
}

#[test]
fn wasm_bind_on_a_cdylib_handle_is_refused() {
    // The wasm bind entry point must not accept a non-wasm backend: cdylib
    // handles bind through the dylib import registry instead.
    let desc = PluginHandle::find_in_process_descriptor("BasicCalculator").expect("descriptor");
    let handle = PluginHandle::from_descriptor(desc).expect("handle");
    // SAFETY: fresh, leaked (process-lifetime) table.
    let err = unsafe { handle.bind_wasm_host_table(table(Arc::new(RecordingHost::default()))) }
        .expect_err("wrong backend");
    match err {
        LoadError::HostBindFailed { code, .. } => {
            assert_eq!(code, host_ffi::BIND_ERR_WRONG_BACKEND);
        }
        other => panic!("expected HostBindFailed, got {other:?}"),
    }
}
