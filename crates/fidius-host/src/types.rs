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

//! Owned metadata types for loaded plugins.

use fidius_core::descriptor::BufferStrategyKind;

/// Plugin runtime kind. Mirrors `fidius_core::package::PackageRuntime` and
/// surfaces it in the host-facing `PluginInfo`. Re-exported here so host
/// callers don't need a transitive `fidius-core` use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginRuntimeKind {
    /// Cdylib + `PluginRegistry` (the original fidius substrate).
    Cdylib,
    /// `.py` package loaded via `fidius-python`'s embedded interpreter.
    /// Only produced when the `python` feature is enabled on `fidius-host`.
    Python,
}

/// Owned metadata for a discovered or loaded plugin.
///
/// All data copied from FFI descriptor — no raw pointers. `capabilities` and
/// `buffer_strategy` are cdylib-specific concepts; for python plugins they
/// take their default values (0 / `PluginAllocated`) and have no runtime
/// meaning.
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
    /// Capability bitfield (optional method support). Cdylib only.
    pub capabilities: u64,
    /// Buffer management strategy. Cdylib only.
    pub buffer_strategy: BufferStrategyKind,
    /// Runtime kind. New in 0.2 — defaults to `Cdylib` for backward
    /// compatibility with code that constructs `PluginInfo` directly.
    pub runtime: PluginRuntimeKind,
}

impl PluginInfo {
    /// True if this is a cdylib-backed plugin.
    pub fn is_cdylib(&self) -> bool {
        matches!(self.runtime, PluginRuntimeKind::Cdylib)
    }

    /// True if this is a Python plugin.
    pub fn is_python(&self) -> bool {
        matches!(self.runtime, PluginRuntimeKind::Python)
    }
}

/// Controls how strictly the host validates plugins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadPolicy {
    /// Reject any validation failure, require signatures if configured.
    #[default]
    Strict,
    /// Warn on unsigned plugins but allow loading.
    Lenient,
}
