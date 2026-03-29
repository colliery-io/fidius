//! Owned metadata types for loaded plugins.

use fidius_core::descriptor::{BufferStrategyKind, WireFormat};

/// Owned metadata for a discovered or loaded plugin.
///
/// All data copied from FFI descriptor — no raw pointers.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Human-readable plugin name (e.g., "BlurFilter").
    pub name: String,
    /// Interface trait name (e.g., "ImageFilter").
    pub interface_name: String,
    /// FNV-1a hash of required method signatures.
    pub interface_hash: u64,
    /// User-specified interface version.
    pub interface_version: u32,
    /// Capability bitfield (optional method support).
    pub capabilities: u64,
    /// Wire serialization format.
    pub wire_format: WireFormat,
    /// Buffer management strategy.
    pub buffer_strategy: BufferStrategyKind,
}

/// Controls how strictly the host validates plugins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadPolicy {
    /// Reject any validation failure, require signatures if configured.
    Strict,
    /// Warn on unsigned plugins but allow loading.
    Lenient,
}

impl Default for LoadPolicy {
    fn default() -> Self {
        LoadPolicy::Strict
    }
}
