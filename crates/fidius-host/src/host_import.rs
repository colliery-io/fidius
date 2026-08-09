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

//! Host-import discovery and binding (plugin → host callback channel).
//!
//! A plugin dylib may export the optional `fidius_get_host_imports` symbol
//! (emitted by `fidius_plugin_registry!()`) advertising which
//! `#[fidius::host_interface]` surfaces it can consume, each with the
//! version and signature hash it was **built against**. This module reads
//! that registry and implements the load-time gate: [`bind_host_interface`]
//! refuses to install a [`HostFunctionTable`] whose version or hash differs
//! from the plugin's expectation, returning a typed [`LoadError`] instead of
//! letting positional bincode mis-dispatch at call time.
//!
//! Host applications normally don't call this module directly — the
//! `#[host_interface]` macro generates a typed `<Trait>Binding::bind` that
//! feeds the interface's own constants through [`bind_host_interface`].

use std::ffi::CStr;

use fidius_core::descriptor::{ABI_VERSION, FIDIUS_MAGIC};
use fidius_core::host_ffi::{
    bind_status_message, HostFunctionTable, HostImportDescriptor, HostImportRegistry, BIND_OK,
    HOST_IMPORTS_VERSION,
};
use libloading::Library;

use crate::error::LoadError;

/// Owned metadata about one host interface a plugin declares it can consume.
#[derive(Debug, Clone)]
pub struct HostImportInfo {
    /// The host interface (trait) name, e.g. `"CloacinaHost"`.
    pub interface_name: String,
    /// FNV-1a signature hash the plugin was built against.
    pub interface_hash: u64,
    /// `#[host_interface(version = N)]` the plugin was built against.
    pub interface_version: u32,
}

/// Signature of the optional `fidius_get_host_imports` export.
type GetHostImportsFn = unsafe extern "C" fn() -> *const HostImportRegistry;

/// Read and validate a library's host-import registry. Returns `None` when
/// the library exports no `fidius_get_host_imports` symbol (a plugin built
/// before the host-callback channel existed, or a non-fidius dylib) — the
/// caller treats that as "no imports".
fn read_registry(library: &Library) -> Result<Option<&'static HostImportRegistry>, LoadError> {
    let get_imports: libloading::Symbol<GetHostImportsFn> =
        match unsafe { library.get(b"fidius_get_host_imports") } {
            Ok(sym) => sym,
            Err(_) => return Ok(None),
        };

    let registry = unsafe { &*get_imports() };
    if registry.magic != FIDIUS_MAGIC {
        return Err(LoadError::HostImportRegistryInvalid {
            reason: "bad magic bytes".into(),
        });
    }
    if registry.registry_version != HOST_IMPORTS_VERSION {
        return Err(LoadError::HostImportRegistryInvalid {
            reason: format!(
                "unsupported registry version {} (host supports {})",
                registry.registry_version, HOST_IMPORTS_VERSION
            ),
        });
    }
    if registry.import_count > 0 && registry.imports.is_null() {
        return Err(LoadError::HostImportRegistryInvalid {
            reason: "null imports array with nonzero count".into(),
        });
    }
    Ok(Some(registry))
}

/// Iterate a validated registry's descriptors.
fn descriptors(
    registry: &'static HostImportRegistry,
) -> impl Iterator<Item = &'static HostImportDescriptor> {
    (0..registry.import_count as usize).map(move |i| {
        // SAFETY: read_registry validated count/array; descriptors are static
        // data in the loaded library.
        unsafe { &**registry.imports.add(i) }
    })
}

/// List the host interfaces a loaded library declares it can consume.
///
/// Empty for plugins that use no host interface **and** for plugins built
/// before the channel existed (no export symbol) — both load and run
/// unchanged, nothing to bind.
pub fn list_host_imports(library: &Library) -> Result<Vec<HostImportInfo>, LoadError> {
    let Some(registry) = read_registry(library)? else {
        return Ok(Vec::new());
    };
    let mut out = Vec::with_capacity(registry.import_count as usize);
    for desc in descriptors(registry) {
        // SAFETY: interface_name is a static NUL-terminated string per the
        // HostImportDescriptor contract (macro-generated from a str literal).
        let name = unsafe { CStr::from_ptr(desc.interface_name) }
            .to_str()
            .map_err(|_| LoadError::HostImportRegistryInvalid {
                reason: "import interface_name is not valid UTF-8".into(),
            })?
            .to_string();
        out.push(HostImportInfo {
            interface_name: name,
            interface_hash: desc.interface_hash,
            interface_version: desc.interface_version,
        });
    }
    Ok(out)
}

/// Install a host-function table into a loaded plugin library, gating on the
/// version and signature hash the plugin was built against.
///
/// `expected_name` / `expected_hash` / `expected_version` describe the
/// surface the **host** provides (the interface's generated constants);
/// `build_table` is called lazily, only once the plugin's matching import
/// has passed the gate, and must return a process-lifetime table.
///
/// Returns:
/// - `Ok(true)` — the plugin imports this interface and the table was
///   installed;
/// - `Ok(false)` — the plugin does not declare this import (older plugin,
///   or one that doesn't use host functions): nothing to do;
/// - `Err(LoadError::HostInterfaceVersionMismatch | HostInterfaceHashMismatch)`
///   — the plugin was built against a **different revision** of this
///   interface. The table is not installed; any plugin-side call would
///   surface as a typed `NotBound` error, never a mis-dispatch;
/// - `Err(LoadError::HostBindFailed)` — the plugin's bind shim rejected the
///   table (e.g. already bound by an earlier call).
pub fn bind_host_interface(
    library: &Library,
    expected_name: &str,
    expected_hash: u64,
    expected_version: u32,
    build_table: impl FnOnce() -> *const HostFunctionTable,
) -> Result<bool, LoadError> {
    let Some(registry) = read_registry(library)? else {
        return Ok(false);
    };

    for desc in descriptors(registry) {
        // SAFETY: static NUL-terminated string per the descriptor contract.
        let name = unsafe { CStr::from_ptr(desc.interface_name) }.to_str();
        if name != Ok(expected_name) {
            continue;
        }

        // ABI gate first: a descriptor from an incompatible fidius build
        // can't be trusted beyond its identity fields.
        if desc.abi_version != ABI_VERSION {
            return Err(LoadError::IncompatibleAbiVersion {
                got: desc.abi_version,
                expected: ABI_VERSION,
            });
        }
        // Version gate: the declared interface revision must match exactly.
        if desc.interface_version != expected_version {
            return Err(LoadError::HostInterfaceVersionMismatch {
                interface: expected_name.to_string(),
                plugin_expects: desc.interface_version,
                host_provides: expected_version,
            });
        }
        // Hash gate: catches signature drift within a version (the same
        // protection the plugin-interface hash provides in the forward
        // direction — bincode is positional, so drift must never dispatch).
        if desc.interface_hash != expected_hash {
            return Err(LoadError::HostInterfaceHashMismatch {
                interface: expected_name.to_string(),
                plugin_expects: desc.interface_hash,
                host_provides: expected_hash,
            });
        }

        let table = build_table();
        // SAFETY: `bind` is a static function in the loaded library; the
        // table is process-lifetime per `build_table`'s contract.
        let status = unsafe { (desc.bind)(table) };
        if status != BIND_OK {
            return Err(LoadError::HostBindFailed {
                interface: expected_name.to_string(),
                code: status,
                message: bind_status_message(status).to_string(),
            });
        }

        #[cfg(feature = "tracing")]
        tracing::debug!(
            interface = expected_name,
            version = expected_version,
            "bound host interface"
        );
        return Ok(true);
    }

    Ok(false)
}

impl crate::loader::LoadedLibrary {
    /// The host interfaces this library's plugins declare they can consume.
    /// See [`list_host_imports`].
    pub fn host_imports(&self) -> Result<Vec<HostImportInfo>, LoadError> {
        list_host_imports(&self.library)
    }
}

impl crate::loader::LoadedPlugin {
    /// The host interfaces this plugin's library declares it can consume.
    /// Imports are per-**library**: plugins in the same dylib share them.
    pub fn host_imports(&self) -> Result<Vec<HostImportInfo>, LoadError> {
        list_host_imports(&self.library)
    }
}
