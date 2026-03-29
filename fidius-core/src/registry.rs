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

//! Plugin registry assembly for multi-plugin dylibs.
//!
//! Each `#[plugin_impl]` submits its `PluginDescriptor` pointer via `inventory::submit!`.
//! Plugin crates call `fidius_plugin_registry!()` once in their lib.rs to emit the
//! `fidius_get_registry` export function that the host calls via `dlsym`.

use crate::descriptor::{PluginDescriptor, PluginRegistry, FIDIUS_MAGIC, REGISTRY_VERSION};

/// A submitted descriptor pointer. Used with `inventory` for distributed collection.
pub struct DescriptorEntry {
    pub descriptor: &'static PluginDescriptor,
}

inventory::collect!(DescriptorEntry);

/// Build the plugin registry from all submitted descriptors.
///
/// Allocates a `Vec` of descriptor pointers and leaks it to get a `'static` pointer.
/// Called once; the result is cached in `OnceLock`.
fn build_registry() -> PluginRegistry {
    let entries: Vec<*const PluginDescriptor> = inventory::iter::<DescriptorEntry>()
        .map(|e| e.descriptor as *const PluginDescriptor)
        .collect();

    let count = entries.len() as u32;
    let ptr = entries.as_ptr();
    std::mem::forget(entries);

    PluginRegistry {
        magic: FIDIUS_MAGIC,
        registry_version: REGISTRY_VERSION,
        plugin_count: count,
        descriptors: ptr,
    }
}

/// Get or build the plugin registry.
///
/// Returns a `'static` reference to the registry. Built on first call,
/// cached for subsequent calls.
pub fn get_registry() -> &'static PluginRegistry {
    static REGISTRY: std::sync::OnceLock<PluginRegistry> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(build_registry)
}

/// Emit the `fidius_get_registry` export function.
///
/// Call this once in your plugin cdylib's `lib.rs`. The host calls
/// `dlsym("fidius_get_registry")` and invokes it to get the registry.
///
/// ```ignore
/// fidius::fidius_plugin_registry!();
/// ```
#[macro_export]
macro_rules! fidius_plugin_registry {
    () => {
        #[no_mangle]
        pub extern "C" fn fidius_get_registry() -> *const fidius_core::descriptor::PluginRegistry {
            fidius_core::registry::get_registry() as *const _
        }
    };
}
