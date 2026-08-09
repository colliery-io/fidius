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

//! Host-import registry assembly (plugin → host callback channel).
//!
//! Each `#[fidius::host_interface]` trait submits a [`HostImportDescriptor`]
//! pointer via `inventory::submit!` (from the interface crate, gated to
//! non-wasm builds). `fidius_plugin_registry!()` emits the optional
//! `fidius_get_host_imports` export that collects them, so a plugin dylib
//! that links a host interface automatically advertises it. Plugins with no
//! host interfaces export an **empty** registry — and plugins built before
//! this channel existed export no such symbol at all; the host treats both
//! identically (nothing to bind).

use fidius_guest::descriptor::FIDIUS_MAGIC;
use fidius_guest::host_ffi::{HostImportDescriptor, HostImportRegistry, HOST_IMPORTS_VERSION};

/// A submitted host-import descriptor pointer, collected via `inventory`.
pub struct HostImportEntry {
    pub descriptor: &'static HostImportDescriptor,
}

inventory::collect!(HostImportEntry);

/// Build the host-import registry from all submitted descriptors.
fn build_host_import_registry() -> HostImportRegistry {
    let entries: Vec<*const HostImportDescriptor> = inventory::iter::<HostImportEntry>()
        .map(|e| e.descriptor as *const HostImportDescriptor)
        .collect();

    let count = entries.len() as u32;
    let ptr = entries.as_ptr();
    std::mem::forget(entries);

    HostImportRegistry {
        magic: FIDIUS_MAGIC,
        registry_version: HOST_IMPORTS_VERSION,
        import_count: count,
        imports: ptr,
    }
}

/// Get or build the host-import registry (cached after first call).
pub fn get_host_import_registry() -> &'static HostImportRegistry {
    static REGISTRY: std::sync::OnceLock<HostImportRegistry> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(build_host_import_registry)
}
