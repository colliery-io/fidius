//! Core plugin loading and descriptor validation.

use std::ffi::c_void;
use std::path::Path;
use std::sync::Arc;

use fides_core::descriptor::*;
use libloading::Library;

use crate::error::LoadError;
use crate::types::PluginInfo;

/// A loaded plugin library with validated descriptors.
pub struct LoadedLibrary {
    /// The dynamically loaded library. Must stay alive while any PluginHandle exists.
    pub library: Arc<Library>,
    /// Validated plugin descriptors with owned metadata.
    pub plugins: Vec<LoadedPlugin>,
}

/// A single validated plugin from a loaded library.
pub struct LoadedPlugin {
    /// Owned metadata copied from the FFI descriptor.
    pub info: PluginInfo,
    /// Raw vtable pointer (points into the loaded library's memory).
    pub vtable: *const c_void,
    /// Free function for plugin-allocated buffers.
    pub free_buffer: Option<unsafe extern "C" fn(*mut u8, usize)>,
    /// Reference to the library to keep it alive.
    pub library: Arc<Library>,
}

// SAFETY: vtable and free_buffer point to static data in the loaded library.
// The Arc<Library> ensures the library stays loaded.
unsafe impl Send for LoadedPlugin {}
unsafe impl Sync for LoadedPlugin {}

/// Load a plugin library from a path.
///
/// Opens the dylib, calls `fides_get_registry()`, validates the registry
/// and all descriptors, copies FFI data to owned types.
pub fn load_library(path: &Path) -> Result<LoadedLibrary, LoadError> {
    let path_str = path.display().to_string();

    // dlopen
    let library = unsafe { Library::new(path) }.map_err(|e| {
        if e.to_string().contains("No such file") || e.to_string().contains("not found") {
            LoadError::LibraryNotFound {
                path: path_str.clone(),
            }
        } else {
            LoadError::LibLoading(e)
        }
    })?;

    // dlsym("fides_get_registry")
    let get_registry: libloading::Symbol<unsafe extern "C" fn() -> *const PluginRegistry> =
        unsafe { library.get(b"fides_get_registry") }.map_err(|_| LoadError::SymbolNotFound {
            path: path_str.clone(),
        })?;

    // Call to get the registry pointer
    let registry = unsafe { &*get_registry() };

    // Validate magic
    if registry.magic != FIDES_MAGIC {
        return Err(LoadError::InvalidMagic);
    }

    // Validate registry version
    if registry.registry_version != REGISTRY_VERSION {
        return Err(LoadError::IncompatibleRegistryVersion {
            got: registry.registry_version,
            expected: REGISTRY_VERSION,
        });
    }

    let library = Arc::new(library);

    // Iterate descriptors and validate each
    let mut plugins = Vec::with_capacity(registry.plugin_count as usize);
    for i in 0..registry.plugin_count {
        let desc = unsafe { &**registry.descriptors.add(i as usize) };
        let plugin = validate_descriptor(desc, &library)?;
        plugins.push(plugin);
    }

    Ok(LoadedLibrary { library, plugins })
}

/// Validate a single descriptor and copy to owned types.
fn validate_descriptor(
    desc: &PluginDescriptor,
    library: &Arc<Library>,
) -> Result<LoadedPlugin, LoadError> {
    // Check ABI version
    if desc.abi_version != ABI_VERSION {
        return Err(LoadError::IncompatibleAbiVersion {
            got: desc.abi_version,
            expected: ABI_VERSION,
        });
    }

    // Copy FFI strings to owned
    let interface_name = unsafe { desc.interface_name_str() }.to_string();
    let plugin_name = unsafe { desc.plugin_name_str() }.to_string();

    let info = PluginInfo {
        name: plugin_name,
        interface_name,
        interface_hash: desc.interface_hash,
        interface_version: desc.interface_version,
        capabilities: desc.capabilities,
        wire_format: desc.wire_format_kind(),
        buffer_strategy: desc.buffer_strategy_kind(),
    };

    Ok(LoadedPlugin {
        info,
        vtable: desc.vtable,
        free_buffer: desc.free_buffer,
        library: Arc::clone(library),
    })
}

/// Validate a loaded plugin against expected interface parameters.
pub fn validate_against_interface(
    plugin: &LoadedPlugin,
    expected_hash: Option<u64>,
    expected_wire: Option<WireFormat>,
    expected_strategy: Option<BufferStrategyKind>,
) -> Result<(), LoadError> {
    if let Some(hash) = expected_hash {
        if plugin.info.interface_hash != hash {
            return Err(LoadError::InterfaceHashMismatch {
                got: plugin.info.interface_hash,
                expected: hash,
            });
        }
    }

    if let Some(wire) = expected_wire {
        if plugin.info.wire_format != wire {
            return Err(LoadError::WireFormatMismatch {
                got: plugin.info.wire_format as u8,
                expected: wire as u8,
            });
        }
    }

    if let Some(strategy) = expected_strategy {
        if plugin.info.buffer_strategy != strategy {
            return Err(LoadError::BufferStrategyMismatch {
                got: plugin.info.buffer_strategy as u8,
                expected: strategy as u8,
            });
        }
    }

    Ok(())
}
