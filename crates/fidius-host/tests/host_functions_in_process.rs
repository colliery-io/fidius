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

//! In-process (no dylib) host-function tests: `bind_in_process` + the
//! plugin-side bind shim's defensive re-validation of a hand-rolled table.
//!
//! Separate test binary: `bind_in_process` installs into this process's
//! copy of the fixture's table cell, which must stay unbound in the other
//! host-function test binaries.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use fidius_core::host_ffi;
use fidius_core::PluginError;
use fidius_host::PluginHandle;

use test_plugin_hostcall::__fidius_Deferrable::METHOD_RUN_DEFERRED;
use test_plugin_hostcall::__fidius_host_TestHost;
use test_plugin_hostcall::{TestHost, TestHostBinding};

struct CountingHost {
    reclaims: AtomicU32,
}

impl TestHost for CountingHost {
    fn release_slot(&self, _task_id: String) -> Result<(), PluginError> {
        Ok(())
    }
    fn reclaim_slot(&self, _task_id: String) -> Result<u32, PluginError> {
        Ok(self.reclaims.fetch_add(1, Ordering::SeqCst) + 1)
    }
    fn get_value(&self, key: String) -> String {
        format!("in-process-{key}")
    }
    fn panicky(&self) -> u32 {
        panic!("boom");
    }
}

/// A doctored table must be rejected by the plugin-side bind shim before
/// anything is stored — the defense against hand-rolled hosts that skip
/// `bind_host_interface`'s gate. Runs FIRST (before the successful bind)
/// via explicit call from the one #[test] below, since the cell is global.
fn doctored_tables_are_rejected_by_the_plugin_shim() {
    let host: Arc<dyn TestHost> = Arc::new(CountingHost {
        reclaims: AtomicU32::new(0),
    });
    let good = TestHostBinding::table(host);

    // Wrong hash.
    let mut bad = unsafe { std::ptr::read(good) };
    bad.interface_hash ^= 1;
    let bad_ptr = Box::into_raw(Box::new(bad));
    let status = unsafe { __fidius_host_TestHost::__fidius_host_bind(bad_ptr) };
    assert_eq!(status, host_ffi::BIND_ERR_HASH_MISMATCH);

    // Wrong version.
    let mut bad = unsafe { std::ptr::read(good) };
    bad.interface_version += 1;
    let bad_ptr = Box::into_raw(Box::new(bad));
    let status = unsafe { __fidius_host_TestHost::__fidius_host_bind(bad_ptr) };
    assert_eq!(status, host_ffi::BIND_ERR_VERSION_MISMATCH);

    // Wrong ABI.
    let mut bad = unsafe { std::ptr::read(good) };
    bad.abi_version += 1;
    let bad_ptr = Box::into_raw(Box::new(bad));
    let status = unsafe { __fidius_host_TestHost::__fidius_host_bind(bad_ptr) };
    assert_eq!(status, host_ffi::BIND_ERR_ABI);

    // Null.
    let status = unsafe { __fidius_host_TestHost::__fidius_host_bind(std::ptr::null()) };
    assert_eq!(status, host_ffi::BIND_ERR_NULL);

    // Nothing was installed by any of the rejected binds.
    assert!(__fidius_host_TestHost::__FIDIUS_HOST_TABLE.get().is_none());
}

#[test]
fn in_process_bind_and_reentrant_call() {
    // Order matters (global cell): rejection cases first, then the real bind.
    doctored_tables_are_rejected_by_the_plugin_shim();

    let host = Arc::new(CountingHost {
        reclaims: AtomicU32::new(0),
    });
    TestHostBinding::bind_in_process(host.clone() as Arc<dyn TestHost>).expect("bind");

    // Second in-process bind is refused.
    let err =
        TestHostBinding::bind_in_process(host as Arc<dyn TestHost>).expect_err("already bound");
    assert_eq!(err.code, host_ffi::BIND_ERR_ALREADY_BOUND);

    // Drive the plugin through the in-process descriptor path; its methods
    // call back into CountingHost.
    let desc = PluginHandle::find_in_process_descriptor("DeferrablePlugin").expect("descriptor");
    let handle = PluginHandle::from_descriptor(desc).expect("handle");
    let out: String = handle
        .call_method(METHOD_RUN_DEFERRED, &("t".to_string(),))
        .expect("run_deferred");
    assert_eq!(out, "deferred:t:cond=in-process-cond:t:reclaims=1");
}
