// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// Host-function fixture (wasm variant): `run` calls back into the host —
// release a slot, read a value, reclaim — while the plugin method is live.
// Error paths are formatted into the return string so the host test can
// assert the typed errors the guest observed (NotBound, VersionMismatch,
// HashMismatch, HostPanic, Host(PluginError)).

use fidius_guest::PluginError;
use fidius_macro::{host_interface, plugin_impl, plugin_interface};

#[host_interface(version = 1, crate = "fidius_guest")]
pub trait WasmSlotHost: Send + Sync {
    /// Errors for `task_id == "reject-me"` (typed host error path).
    fn release_slot(&self, task_id: String) -> Result<(), PluginError>;
    /// Returns the total number of reclaims so far.
    fn reclaim_slot(&self, task_id: String) -> Result<u32, PluginError>;
    /// Infallible host function shape.
    fn get_value(&self, key: String) -> String;
    /// Always panics on the host side.
    fn panicky(&self) -> u32;
}

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait WasmDeferrable: Send + Sync {
    /// The full round-trip: host → plugin (`run`) → host (release/get/reclaim).
    fn run(&self, task_id: String) -> String;
    /// Whether a matching host table is bound (probe).
    fn host_bound(&self) -> bool;
    /// Calls a host function and formats the outcome (typed-error observation).
    fn try_release(&self, task_id: String) -> String;
    /// Calls the panicking host function and reports the typed error it got.
    fn observe_host_panic(&self) -> String;
}

pub struct WasmDeferrablePlugin;

#[plugin_impl(WasmDeferrable, crate = "fidius_guest")]
impl WasmDeferrable for WasmDeferrablePlugin {
    fn run(&self, task_id: String) -> String {
        let host = match WasmSlotHostClient::bound() {
            Ok(h) => h,
            Err(e) => return format!("bound-error:{e:?}"),
        };
        if let Err(e) = host.release_slot(&task_id) {
            return format!("release-error:{e:?}");
        }
        let cond = match host.get_value(&format!("cond:{task_id}")) {
            Ok(v) => v,
            Err(e) => return format!("get-error:{e:?}"),
        };
        match host.reclaim_slot(&task_id) {
            Ok(n) => format!("deferred:{task_id}:cond={cond}:reclaims={n}"),
            Err(e) => format!("reclaim-error:{e:?}"),
        }
    }

    fn host_bound(&self) -> bool {
        WasmSlotHostClient::is_bound()
    }

    fn try_release(&self, task_id: String) -> String {
        match WasmSlotHostClient::bound() {
            Ok(host) => match host.release_slot(&task_id) {
                Ok(()) => "released".to_string(),
                Err(e) => format!("call-error:{e:?}"),
            },
            Err(e) => format!("not-bound-or-mismatch:{e:?}"),
        }
    }

    fn observe_host_panic(&self) -> String {
        match WasmSlotHostClient::bound() {
            Ok(host) => match host.panicky() {
                Ok(v) => format!("unexpected-ok:{v}"),
                Err(e) => format!("{e:?}"),
            },
            Err(e) => format!("bound-error:{e:?}"),
        }
    }
}
