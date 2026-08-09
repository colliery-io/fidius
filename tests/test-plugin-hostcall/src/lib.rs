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

//! Test fixture for the plugin → host callback channel (host functions).
//!
//! Models the cloacina `defer_until` shape: the host calls
//! `run_deferred(task_id)` on the plugin, and the plugin — mid-execution —
//! calls back into the host to release its concurrency slot, poll a
//! condition value, and reclaim the slot. That is the reentrant
//! host → plugin → host path end to end.

use fidius::{host_interface, plugin_impl, plugin_interface, PluginError};

/// The host functions this plugin can call back into.
///
/// `TestHostClient` (plugin side) and `TestHostBinding` (host side) are
/// generated from this trait.
#[host_interface(version = 1)]
pub trait TestHost: Send + Sync {
    /// Release the task's concurrency slot. Errors for `task_id == "reject-me"`
    /// so tests can observe a host-raised typed error.
    fn release_slot(&self, task_id: String) -> Result<(), PluginError>;

    /// Reclaim a concurrency slot (may block in a real host while waiting
    /// for capacity). Returns the total number of reclaims so far.
    fn reclaim_slot(&self, task_id: String) -> Result<u32, PluginError>;

    /// Fetch a host-side value (infallible host function shape).
    fn get_value(&self, key: String) -> String;

    /// Always panics — proves a host-side panic is caught at the boundary
    /// and surfaces to the plugin as a typed `HostCallError::HostPanic`.
    fn panicky(&self) -> u32;
}

/// The plugin interface the host drives.
#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Deferrable: Send + Sync {
    /// The reentrant path: release slot → read condition → reclaim slot,
    /// all as host callbacks made while this plugin call is live.
    fn run_deferred(&self, task_id: String) -> Result<String, PluginError>;

    /// Calls a host function from a plugin-spawned thread (the threading
    /// contract allows callbacks from threads the host doesn't own).
    fn value_from_thread(&self, key: String) -> String;

    /// Calls the panicking host function and reports the typed error it got.
    fn observe_host_panic(&self) -> String;

    /// Whether the host bound the `TestHost` interface for this plugin.
    fn host_bound(&self) -> bool;

    /// Calls a host function and formats the outcome — used by the
    /// "host never bound" test to observe the typed `NotBound` error.
    fn try_release(&self, task_id: String) -> String;

    /// Panics *after* calling a host function — proves plugin-side panics
    /// around a callback still surface as `CallError::Panic` to the host.
    fn panic_after_callback(&self, task_id: String) -> String;
}

pub struct DeferrablePlugin;

#[plugin_impl(Deferrable)]
impl Deferrable for DeferrablePlugin {
    fn run_deferred(&self, task_id: String) -> Result<String, PluginError> {
        let host =
            TestHostClient::bound().map_err(|e| PluginError::new("HOST_UNBOUND", e.to_string()))?;
        host.release_slot(&task_id)
            .map_err(|e| PluginError::new("RELEASE_FAILED", e.to_string()))?;
        // "Poll the condition" mid-deferral — a genuine mid-execution call
        // in the plugin → host direction.
        let cond = host
            .get_value(&format!("cond:{task_id}"))
            .map_err(|e| PluginError::new("GET_VALUE_FAILED", e.to_string()))?;
        let reclaims = host
            .reclaim_slot(&task_id)
            .map_err(|e| PluginError::new("RECLAIM_FAILED", e.to_string()))?;
        Ok(format!(
            "deferred:{task_id}:cond={cond}:reclaims={reclaims}"
        ))
    }

    fn value_from_thread(&self, key: String) -> String {
        std::thread::spawn(move || {
            let host = TestHostClient::bound().expect("host bound");
            host.get_value(&key).expect("get_value from spawned thread")
        })
        .join()
        .expect("plugin-spawned thread")
    }

    fn observe_host_panic(&self) -> String {
        let host = TestHostClient::bound().expect("host bound");
        match host.panicky() {
            Ok(v) => format!("unexpected-ok:{v}"),
            Err(e) => format!("{e:?}"),
        }
    }

    fn host_bound(&self) -> bool {
        TestHostClient::is_bound()
    }

    fn try_release(&self, task_id: String) -> String {
        match TestHostClient::bound() {
            Ok(host) => match host.release_slot(&task_id) {
                Ok(()) => "released".to_string(),
                Err(e) => format!("call-error:{e:?}"),
            },
            Err(e) => format!("not-bound:{e:?}"),
        }
    }

    fn panic_after_callback(&self, task_id: String) -> String {
        let host = TestHostClient::bound().expect("host bound");
        let _ = host.release_slot(&task_id);
        panic!("plugin panicked after callback for {task_id}");
    }
}

fidius::fidius_plugin_registry!();
