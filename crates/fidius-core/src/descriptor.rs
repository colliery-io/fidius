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

// ABI_VERSION is derived from the crate's semver per ADR-0002.
// Pre-1.0: every minor release is a breaking change → encode as MAJOR*10000 + MINOR*100.
// Post-1.0: minor releases must be ABI-additive (new fields at the end, new enum variants),
// so only MAJOR changes ABI_VERSION → encode as MAJOR*10000.
// Patch releases are always ABI-compatible and do not affect ABI_VERSION.
const fn parse_u32_const(s: &str) -> u32 {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut n = 0u32;
    while i < bytes.len() {
        n = n * 10 + (bytes[i] - b'0') as u32;
        i += 1;
    }
    n
}

const CRATE_MAJOR: u32 = parse_u32_const(env!("CARGO_PKG_VERSION_MAJOR"));
const CRATE_MINOR: u32 = parse_u32_const(env!("CARGO_PKG_VERSION_MINOR"));

/// Current version of the `PluginDescriptor` struct layout. Derived from the
/// `fidius-core` crate version per ADR-0002.
pub const ABI_VERSION: u32 = if CRATE_MAJOR == 0 {
    CRATE_MAJOR * 10000 + CRATE_MINOR * 100
} else {
    CRATE_MAJOR * 10000
};

/// Buffer management strategy for an interface.
///
/// Selected per-trait via `#[plugin_interface(buffer = ...)]`.
/// Determines the FFI function pointer signatures in the vtable.
///
/// Discriminant value `0` is reserved (previously `CallerAllocated`, removed
/// in 0.1.0 — its value proposition was subsumed by `PluginAllocated`).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferStrategyKind {
    /// Plugin allocates output; host frees via `PluginDescriptor::free_buffer`.
    /// VTable fns: `(in_ptr, in_len, out_ptr, out_len) -> i32`.
    PluginAllocated = 1,
    /// Host provides a pre-allocated arena buffer; plugin writes its serialized
    /// output into the buffer. Returns `STATUS_BUFFER_TOO_SMALL` (with needed
    /// size written to `out_len`) if the arena is too small; host grows and
    /// retries. Data is valid only until the next call.
    ///
    /// VTable fns: `(in_ptr, in_len, arena_ptr, arena_cap, out_offset, out_len) -> i32`.
    ///
    /// **Arena is allocation-avoidance, not zero-copy.** The plugin still
    /// serializes its output (bincode-encoded by default) and copies the
    /// bytes into the host-provided buffer; what Arena saves is the per-call
    /// `Box<[u8]>` allocation that PluginAllocated incurs. For true byte
    /// passthrough — the bytes themselves cross the boundary without an
    /// encoding step — annotate the trait method with `#[wire(raw)]`. Raw
    /// wire mode composes with both buffer strategies.
    Arena = 2,
}

/// Static key/value pair for method-level or trait-level metadata.
///
/// Both `key` and `value` point to null-terminated UTF-8 C strings with
/// `'static` lifetime (typically string literals embedded in the plugin's
/// `.rodata`). Fidius treats values as opaque — hosts define conventions
/// via their own metadata schemas. See ADR/spec for the `fidius.*`
/// reserved namespace.
#[repr(C)]
pub struct MetaKv {
    /// Null-terminated UTF-8 key. Never null.
    pub key: *const c_char,
    /// Null-terminated UTF-8 value. Never null (may be empty string).
    pub value: *const c_char,
}

// SAFETY: MetaKv fields are static, immutable pointers to `.rodata` strings.
unsafe impl Send for MetaKv {}
unsafe impl Sync for MetaKv {}

/// Per-method metadata entry. One entry per method in declaration order,
/// stored in the array referenced by `PluginDescriptor::method_metadata`.
///
/// Methods with no `#[method_meta(...)]` annotations have `kvs: null` and
/// `kv_count: 0` — the entry exists but is empty, so hosts can index
/// uniformly by method index.
#[repr(C)]
pub struct MethodMetaEntry {
    /// Pointer to an array of `kv_count` `MetaKv` entries, or null if this
    /// method has no metadata.
    pub kvs: *const MetaKv,
    /// Number of key/value pairs for this method. Zero when `kvs` is null.
    pub kv_count: u32,
}

// SAFETY: MethodMetaEntry fields reference static data.
unsafe impl Send for MethodMetaEntry {}
unsafe impl Sync for MethodMetaEntry {}

impl std::fmt::Display for BufferStrategyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferStrategyKind::PluginAllocated => write!(f, "PluginAllocated"),
            BufferStrategyKind::Arena => write!(f, "Arena"),
        }
    }
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
    /// Size in bytes of this descriptor struct at plugin build time.
    ///
    /// The host reads this field FIRST (it's at offset 0) before trusting any
    /// other offset calculation. Any field whose offset is >= `descriptor_size`
    /// is not present in this plugin's build — the plugin was compiled against
    /// an older fidius version that didn't have that field yet.
    ///
    /// Enables post-1.0 minor releases to add new fields at the end of this
    /// struct without breaking older plugins. See ADR-0002.
    pub descriptor_size: u32,
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
    /// Pointer to an array of `method_count` `MethodMetaEntry` structs,
    /// one per method in declaration order. Each entry may be empty
    /// (kvs=null, kv_count=0) if the method declared no metadata.
    ///
    /// Null if the interface used no `#[method_meta(...)]` annotations
    /// at all (optimization for the common case).
    pub method_metadata: *const MethodMetaEntry,
    /// Pointer to an array of `trait_metadata_count` `MetaKv` entries for
    /// trait-level metadata (declared via `#[trait_meta(...)]`).
    ///
    /// Null if no trait-level metadata was declared.
    pub trait_metadata: *const MetaKv,
    /// Number of entries in `trait_metadata`. Zero when `trait_metadata`
    /// is null.
    pub trait_metadata_count: u32,
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
            1 => Ok(BufferStrategyKind::PluginAllocated),
            2 => Ok(BufferStrategyKind::Arena),
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
