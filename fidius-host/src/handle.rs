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

//! PluginHandle — type-safe proxy for calling plugin methods via FFI.

use std::ffi::c_void;
use std::sync::Arc;

use libloading::Library;
use serde::de::DeserializeOwned;
use serde::Serialize;

use fidius_core::status::*;
use fidius_core::wire;
use fidius_core::PluginError;

use crate::error::CallError;
use crate::types::PluginInfo;

/// Type alias for the PluginAllocated FFI function pointer signature.
type FfiFn = unsafe extern "C" fn(*const u8, u32, *mut *mut u8, *mut u32) -> i32;

/// A handle to a loaded plugin, ready for calling methods.
///
/// Holds an `Arc<Library>` to keep the dylib loaded as long as any handle exists.
/// Call methods via `call_method()` which handles serialization, FFI, and cleanup.
///
/// `PluginHandle` is `Send + Sync`. Plugin methods take `&self` (enforced by
/// the macro), so concurrent calls from multiple threads are safe as long as
/// the plugin implementation is thread-safe internally.
pub struct PluginHandle {
    /// Keeps the library alive.
    _library: Arc<Library>,
    /// Pointer to the `#[repr(C)]` vtable struct in the loaded library.
    vtable: *const c_void,
    /// Free function for plugin-allocated output buffers.
    free_buffer: Option<unsafe extern "C" fn(*mut u8, usize)>,
    /// Capability bitfield for optional method support.
    capabilities: u64,
    /// Total number of methods in the vtable.
    method_count: u32,
    /// Owned plugin metadata.
    info: PluginInfo,
}

// SAFETY: PluginHandle is Send + Sync because:
// - vtable and free_buffer are function pointers to static code in the loaded library
// - Arc<Library> is Send + Sync and ensures the library stays loaded
// - All access through call_method is read-only (no mutation of handle state)
//
// Plugin implementations must be thread-safe (&self methods, no &mut self)
// if the PluginHandle is shared across threads. This is enforced at compile
// time by the #[plugin_interface] macro which rejects &mut self methods.
unsafe impl Send for PluginHandle {}
unsafe impl Sync for PluginHandle {}

impl PluginHandle {
    /// Create a new PluginHandle. Crate-private — use `from_loaded()` instead.
    #[allow(dead_code)]
    pub(crate) fn new(
        library: Arc<Library>,
        vtable: *const c_void,
        free_buffer: Option<unsafe extern "C" fn(*mut u8, usize)>,
        capabilities: u64,
        method_count: u32,
        info: PluginInfo,
    ) -> Self {
        Self {
            _library: library,
            vtable,
            free_buffer,
            capabilities,
            method_count,
            info,
        }
    }

    /// Create a PluginHandle from a LoadedPlugin.
    pub fn from_loaded(plugin: crate::loader::LoadedPlugin) -> Self {
        Self {
            _library: plugin.library,
            vtable: plugin.vtable,
            free_buffer: plugin.free_buffer,
            capabilities: plugin.info.capabilities,
            method_count: plugin.method_count,
            info: plugin.info,
        }
    }

    /// Call a plugin method by vtable index.
    ///
    /// Serializes the input, calls the FFI function pointer at the given index,
    /// checks the status code, deserializes the output, and frees the plugin-allocated buffer.
    ///
    /// # Arguments
    /// * `index` - The method index in the vtable (0-based, in declaration order)
    /// * `input` - The input argument to serialize and pass to the plugin
    pub fn call_method<I: Serialize, O: DeserializeOwned>(
        &self,
        index: usize,
        input: &I,
    ) -> Result<O, CallError> {
        // Bounds check: ensure index is within the vtable
        if index >= self.method_count as usize {
            return Err(CallError::NotImplemented { bit: index as u32 });
        }

        // Serialize input
        let input_bytes =
            wire::serialize(input).map_err(|e| CallError::Serialization(e.to_string()))?;

        // Get the function pointer from the vtable
        let fn_ptr = unsafe {
            let fn_ptrs = self.vtable as *const FfiFn;
            *fn_ptrs.add(index)
        };

        // Call the FFI function
        let mut out_ptr: *mut u8 = std::ptr::null_mut();
        let mut out_len: u32 = 0;

        let status = unsafe {
            fn_ptr(
                input_bytes.as_ptr(),
                input_bytes.len() as u32,
                &mut out_ptr,
                &mut out_len,
            )
        };

        // Handle status codes
        match status {
            STATUS_OK => {}
            STATUS_BUFFER_TOO_SMALL => return Err(CallError::BufferTooSmall),
            STATUS_SERIALIZATION_ERROR => {
                return Err(CallError::Serialization("FFI serialization failed".into()))
            }
            STATUS_PLUGIN_ERROR => {
                // Output buffer contains a serialized PluginError
                if !out_ptr.is_null() && out_len > 0 {
                    let output_slice =
                        unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
                    let plugin_err: PluginError = wire::deserialize(output_slice)
                        .map_err(|e| CallError::Deserialization(e.to_string()))?;

                    // Free the buffer
                    if let Some(free) = self.free_buffer {
                        unsafe { free(out_ptr, out_len as usize) };
                    }

                    return Err(CallError::Plugin(plugin_err));
                }
                return Err(CallError::Plugin(PluginError::new(
                    "UNKNOWN",
                    "plugin returned error but no error data",
                )));
            }
            STATUS_PANIC => {
                // Try to extract panic message from output buffer
                let msg = if !out_ptr.is_null() && out_len > 0 {
                    let slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
                    let msg = wire::deserialize::<String>(slice)
                        .unwrap_or_else(|_| "unknown panic".into());
                    if let Some(free) = self.free_buffer {
                        unsafe { free(out_ptr, out_len as usize) };
                    }
                    msg
                } else {
                    "unknown panic".into()
                };
                return Err(CallError::Panic(msg));
            }
            _ => return Err(CallError::UnknownStatus { code: status }),
        }

        // Defensive check: ensure plugin set the output pointer
        if out_ptr.is_null() {
            return Err(CallError::Serialization(
                "plugin returned null output buffer".into(),
            ));
        }

        // Deserialize output
        let output_slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
        let result: Result<O, CallError> =
            wire::deserialize(output_slice).map_err(|e| CallError::Deserialization(e.to_string()));

        // Free the plugin-allocated buffer
        if let Some(free) = self.free_buffer {
            unsafe { free(out_ptr, out_len as usize) };
        }

        result
    }

    /// Check if an optional method is supported (capability bit is set).
    ///
    /// Returns `false` for bit indices >= 64 rather than panicking.
    pub fn has_capability(&self, bit: u32) -> bool {
        if bit >= 64 {
            return false;
        }
        self.capabilities & (1u64 << bit) != 0
    }

    /// Access the plugin's owned metadata.
    pub fn info(&self) -> &PluginInfo {
        &self.info
    }
}
