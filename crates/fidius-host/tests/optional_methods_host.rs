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

//! Calling a method a plugin does NOT have — the interface-evolution / version-skew
//! case (a host built against a newer interface calls a method index a plugin
//! built against an older interface lacks) — must return a clean error, NEVER
//! dereference past the plugin's vtable (which would segfault).
//!
//! The host bounds-checks `index < method_count` before the vtable read, and the
//! cdylib executor now reads each slot as `Option<fn>` (defense-in-depth: a null
//! slot surfaces `NotImplemented` rather than calling a null pointer).

#![allow(unexpected_cfgs)]

use fidius_host::{CallError, PluginHandle};

#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Api: Send + Sync {
    fn base(&self) -> String;
}

pub struct Plugin;

#[fidius_macro::plugin_impl(Api, crate = "fidius_core")]
impl Api for Plugin {
    fn base(&self) -> String {
        "base".into()
    }
}

fidius_core::fidius_plugin_registry!();

#[test]
fn calling_a_method_the_plugin_lacks_is_a_clean_error_not_a_segfault() {
    let desc = PluginHandle::find_in_process_descriptor("Plugin").unwrap();
    let handle = PluginHandle::from_descriptor(desc).unwrap();

    // The one method (index 0) works.
    let b: String = handle.call_method(0, &()).expect("base");
    assert_eq!(b, "base");

    // A method index this plugin doesn't have (e.g. an optional added in a newer
    // interface version) — must be a clean error, not a dereference past the
    // vtable. Streaming + raw paths share the same guard.
    let unary: Result<String, CallError> = handle.call_method(1, &());
    assert!(
        matches!(
            unary,
            Err(CallError::InvalidMethodIndex { index: 1, count: 1 })
        ),
        "expected InvalidMethodIndex, got {unary:?}"
    );
    let raw = handle.call_method_raw(7, b"x");
    assert!(
        matches!(raw, Err(CallError::InvalidMethodIndex { .. })),
        "raw out-of-range must be a clean error, got {raw:?}"
    );
}
