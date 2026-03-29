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

//! FFI descriptor and registry types for the Fidius plugin framework.
//!
//! These types form the stable C ABI contract between host and plugin.
//! All types use `#[repr(C)]` layout and are read directly from dylib memory.

use std::ffi::c_char;
use std::ffi::c_void;

/// Magic bytes identifying a Fidius plugin registry.
pub const FIDIUS_MAGIC: [u8; 8] = *b"FIDIUS\0\0";

/// Current version of the `PluginRegistry` struct layout.
pub const REGISTRY_VERSION: u32 = 1;

/// Current version of the `PluginDescriptor` struct layout.
/// Bumped to 2 to add `method_count` field.
pub const ABI_VERSION: u32 = 2;

/// Buffer management strategy for an interface.
///
/// Selected per-trait via `#[plugin_interface(buffer = ...)]`.
/// Determines the FFI function pointer signatures in the vtable.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferStrategyKind {
    /// Host allocates output buffer; plugin writes into it.
    /// Returns `-1` with needed size if buffer is too small.
    CallerAllocated = 0,
    /// Plugin allocates output; host frees via `PluginDescriptor::free_buffer`.
    PluginAllocated = 1,
    /// Host provides a pre-allocated arena; plugin writes into it.
    /// Data is valid only until the next call.
    Arena = 2,
}

/// Wire serialization format.
///
/// Determined at compile time via `cfg(debug_assertions)`.
/// Host rejects plugins compiled with a mismatched format.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireFormat {
    /// JSON via `serde_json` — human-readable, used in debug builds.
    Json = 0,
    /// bincode — compact and fast, used in release builds.
    Bincode = 1,
}

/// Top-level registry exported by every Fidius plugin dylib.
///
/// Each dylib exports exactly one `FIDIUS_PLUGIN_REGISTRY` static symbol
/// pointing to this struct. The registry contains pointers to one or more
/// `PluginDescriptor`s (supporting multiple plugins per dylib).
///
/// # Safety
///
/// - `descriptors` must point to a valid array of `plugin_count` pointers.
/// - Each pointer in the array must point to a valid `PluginDescriptor`.
/// - All pointed-to data must have `'static` lifetime (typically link-time constants).
#[repr(C)]
pub struct PluginRegistry {
    /// Magic bytes — must equal `FIDIUS_MAGIC` (`b"FIDIUS\0\0"`).
    pub magic: [u8; 8],
    /// Layout version of this struct. Must equal `REGISTRY_VERSION`.
    pub registry_version: u32,
    /// Number of plugin descriptors in this registry.
    pub plugin_count: u32,
    /// Pointer to an array of `plugin_count` descriptor pointers.
    pub descriptors: *const *const PluginDescriptor,
}

// SAFETY: PluginRegistry contains only primitive fields and a pointer to
// static data. The pointed-to descriptors are immutable after construction
// and have 'static lifetime.
unsafe impl Send for PluginRegistry {}
unsafe impl Sync for PluginRegistry {}

/// Metadata descriptor for a single plugin within a dylib.
///
/// Contains all information the host needs to validate and call the plugin
/// without executing any plugin code. All string fields are pointers to
/// static, null-terminated C strings embedded in the dylib.
///
/// # Safety
///
/// - `interface_name` and `plugin_name` must point to valid, null-terminated,
///   UTF-8 C strings with `'static` lifetime.
/// - `vtable` must point to a valid `#[repr(C)]` vtable struct matching the
///   interface identified by `interface_name` and `interface_hash`.
/// - When `buffer_strategy == PluginAllocated`, `free_buffer` must be `Some`.
/// - All pointed-to data must outlive any `PluginHandle` derived from this descriptor.
#[repr(C)]
pub struct PluginDescriptor {
    /// Descriptor struct layout version. Must equal `ABI_VERSION`.
    pub abi_version: u32,
    /// Null-terminated name of the trait this plugin implements (e.g., `"ImageFilter"`).
    pub interface_name: *const c_char,
    /// FNV-1a hash of the required method signatures. Detects ABI drift.
    pub interface_hash: u64,
    /// User-specified interface version from `#[plugin_interface(version = N)]`.
    pub interface_version: u32,
    /// Bitfield where bit N indicates optional method N is implemented.
    /// Supports up to 64 optional methods per interface.
    pub capabilities: u64,
    /// Wire serialization format this plugin was compiled with.
    pub wire_format: u8,
    /// Buffer management strategy this plugin's vtable expects.
    pub buffer_strategy: u8,
    /// Null-terminated human-readable name for this plugin implementation.
    pub plugin_name: *const c_char,
    /// Opaque pointer to the interface-specific `#[repr(C)]` vtable struct.
    pub vtable: *const c_void,
    /// Deallocation function for plugin-allocated buffers.
    /// Must be `Some` when `buffer_strategy == PluginAllocated`.
    /// The host calls this after reading output data to free the plugin's allocation.
    pub free_buffer: Option<unsafe extern "C" fn(*mut u8, usize)>,
    /// Total number of methods in the vtable (required + optional).
    /// Used for bounds checking in `call_method`.
    pub method_count: u32,
}

// SAFETY: PluginDescriptor fields are either primitives, pointers to static
// data, or function pointers. All are immutable after construction and the
// pointed-to data has 'static lifetime.
unsafe impl Send for PluginDescriptor {}
unsafe impl Sync for PluginDescriptor {}

/// A `Sync` wrapper for a raw pointer to a `PluginDescriptor`.
///
/// Used in static contexts where a `*const PluginDescriptor` needs to live
/// in a `static` variable (which requires `Sync`). The pointed-to descriptor
/// must have `'static` lifetime.
#[repr(transparent)]
pub struct DescriptorPtr(pub *const PluginDescriptor);

// SAFETY: The pointer targets static data that is immutable after construction.
unsafe impl Send for DescriptorPtr {}
unsafe impl Sync for DescriptorPtr {}

impl PluginDescriptor {
    /// Read the `interface_name` field as a Rust `&str`.
    ///
    /// # Safety
    ///
    /// `interface_name` must point to a valid, null-terminated, UTF-8 C string
    /// that outlives the returned reference.
    pub unsafe fn interface_name_str(&self) -> &str {
        let cstr = unsafe { std::ffi::CStr::from_ptr(self.interface_name) };
        cstr.to_str().expect("interface_name is not valid UTF-8")
    }

    /// Read the `plugin_name` field as a Rust `&str`.
    ///
    /// # Safety
    ///
    /// `plugin_name` must point to a valid, null-terminated, UTF-8 C string
    /// that outlives the returned reference.
    pub unsafe fn plugin_name_str(&self) -> &str {
        let cstr = unsafe { std::ffi::CStr::from_ptr(self.plugin_name) };
        cstr.to_str().expect("plugin_name is not valid UTF-8")
    }

    /// Returns the `buffer_strategy` field as a `BufferStrategyKind`.
    ///
    /// Returns `Err(value)` if the discriminant is unknown. This can happen
    /// with malformed plugins — callers should reject rather than panic.
    pub fn buffer_strategy_kind(&self) -> Result<BufferStrategyKind, u8> {
        match self.buffer_strategy {
            0 => Ok(BufferStrategyKind::CallerAllocated),
            1 => Ok(BufferStrategyKind::PluginAllocated),
            2 => Ok(BufferStrategyKind::Arena),
            v => Err(v),
        }
    }

    /// Returns the `wire_format` field as a `WireFormat`.
    ///
    /// Returns `Err(value)` if the discriminant is unknown.
    pub fn wire_format_kind(&self) -> Result<WireFormat, u8> {
        match self.wire_format {
            0 => Ok(WireFormat::Json),
            1 => Ok(WireFormat::Bincode),
            v => Err(v),
        }
    }

    /// Check if the given optional method capability bit is set.
    ///
    /// Returns `false` for bit indices >= 64 rather than panicking.
    pub fn has_capability(&self, bit: u32) -> bool {
        if bit >= 64 {
            return false;
        }
        self.capabilities & (1u64 << bit) != 0
    }
}
