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

//! Error types for fidius-host plugin loading and calling.

use fidius_core::PluginError;

/// Errors that can occur when loading a plugin.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("library not found: {path}")]
    LibraryNotFound { path: String },

    #[error("symbol 'fidius_get_registry' not found in {path}")]
    SymbolNotFound { path: String },

    #[error("invalid magic bytes (expected FIDIUS\\0\\0)")]
    InvalidMagic,

    #[error("incompatible registry version: got {got}, expected {expected}")]
    IncompatibleRegistryVersion { got: u32, expected: u32 },

    #[error("incompatible ABI version: got {got}, expected {expected}")]
    IncompatibleAbiVersion { got: u32, expected: u32 },

    #[error("interface hash mismatch: got {got:#x}, expected {expected:#x}")]
    InterfaceHashMismatch { got: u64, expected: u64 },

    #[error("buffer strategy mismatch: plugin uses {got}, host expects {expected}")]
    BufferStrategyMismatch {
        got: fidius_core::descriptor::BufferStrategyKind,
        expected: fidius_core::descriptor::BufferStrategyKind,
    },

    #[error("architecture mismatch: expected {expected}, got {got}")]
    ArchitectureMismatch { expected: String, got: String },

    #[error("unknown buffer strategy discriminant: {value}")]
    UnknownBufferStrategy { value: u8 },

    #[error("signature verification failed for {path}")]
    SignatureInvalid { path: String },

    #[error("signature required but no .sig file found for {path}")]
    SignatureRequired { path: String },

    #[error("plugin '{name}' not found")]
    PluginNotFound { name: String },

    #[error("libloading error: {0}")]
    LibLoading(#[from] libloading::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors that can occur when calling a plugin method.
#[derive(Debug, thiserror::Error)]
pub enum CallError {
    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("deserialization error: {0}")]
    Deserialization(String),

    #[error("plugin error: {0}")]
    Plugin(PluginError),

    #[error("plugin panicked: {0}")]
    Panic(String),

    #[error("buffer too small")]
    BufferTooSmall,

    /// Optional method is not implemented by this plugin — its capability bit is unset.
    /// Returned when a method marked `#[optional]` is called on a plugin that chose not
    /// to implement it. Not returned for out-of-range method indices; see `InvalidMethodIndex`.
    #[error("method not implemented (capability bit {bit} not set)")]
    NotImplemented { bit: u32 },

    #[error("invalid method index {index} (plugin has {count} method(s))")]
    InvalidMethodIndex { index: usize, count: u32 },

    #[error("unknown FFI status code: {code}")]
    UnknownStatus { code: i32 },
}
