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

    /// Python loader failed (only produced with the `python` feature on).
    /// Wraps `fidius_python::PythonLoadError` as a string to keep the
    /// fidius-host public error enum type-clean across feature gates.
    #[error("python load failed: {0}")]
    PythonLoad(String),
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

    /// A method was dispatched through the wrong wire path — a `#[wire(raw)]`
    /// method called via the typed path, or vice versa. Backend-agnostic: the
    /// Python and (future) WASM executors both enforce the raw/typed split.
    #[error(
        "wire-mode mismatch on method '{method}': declared wire_raw={declared}, dispatcher used wire_raw={attempted}"
    )]
    WireModeMismatch {
        method: String,
        declared: bool,
        attempted: bool,
    },

    /// A runtime-level fault originating inside an execution backend that is
    /// *not* a plugin-raised [`PluginError`] — e.g. a future WASM trap
    /// (unreachable, out-of-bounds) or an interpreter-level failure. Carries
    /// the backend's runtime name and a message. Plugin-raised errors (Python
    /// exceptions included) stay in [`CallError::Plugin`] so their structured
    /// `code`/`message`/`details` (including tracebacks) round-trip.
    #[error("{runtime} backend error: {message}")]
    Backend { runtime: String, message: String },
}

/// Fold the Python backend's call error into the unified [`CallError`].
///
/// `fidius-python` deliberately does not depend on `fidius-host` (that would
/// cycle — the host optionally depends on it), so the conversion lives here,
/// behind the `python` feature, where both types are visible. Plugin-raised
/// Python exceptions map to [`CallError::Plugin`] with the traceback preserved
/// in `PluginError.details` (built by `fidius_python::pyerr_to_plugin_error`).
#[cfg(feature = "python")]
impl From<fidius_python::PythonCallError> for CallError {
    fn from(e: fidius_python::PythonCallError) -> Self {
        use fidius_python::PythonCallError as P;
        match e {
            P::InvalidMethodIndex { index, count } => CallError::InvalidMethodIndex {
                index,
                count: count as u32,
            },
            P::WireModeMismatch {
                method,
                declared,
                attempted,
            } => CallError::WireModeMismatch {
                method: method.to_string(),
                declared,
                attempted,
            },
            P::InputDecode(msg) => CallError::Deserialization(msg),
            P::OutputEncode(msg) => CallError::Serialization(msg),
            P::Plugin(err) => CallError::Plugin(err),
        }
    }
}
