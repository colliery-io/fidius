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
    /// Buffer management strategy.
    pub buffer_strategy: BufferStrategyKind,
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
