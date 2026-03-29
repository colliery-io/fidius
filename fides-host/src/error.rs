//! Error types for fides-host plugin loading and calling.

use fides_core::PluginError;

/// Errors that can occur when loading a plugin.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("library not found: {path}")]
    LibraryNotFound { path: String },

    #[error("symbol 'fides_get_registry' not found in {path}")]
    SymbolNotFound { path: String },

    #[error("invalid magic bytes (expected FIDES\\0\\0\\0)")]
    InvalidMagic,

    #[error("incompatible registry version: got {got}, expected {expected}")]
    IncompatibleRegistryVersion { got: u32, expected: u32 },

    #[error("incompatible ABI version: got {got}, expected {expected}")]
    IncompatibleAbiVersion { got: u32, expected: u32 },

    #[error("interface hash mismatch: got {got:#x}, expected {expected:#x}")]
    InterfaceHashMismatch { got: u64, expected: u64 },

    #[error("wire format mismatch: got {got}, expected {expected}")]
    WireFormatMismatch { got: u8, expected: u8 },

    #[error("buffer strategy mismatch: got {got}, expected {expected}")]
    BufferStrategyMismatch { got: u8, expected: u8 },

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

    #[error("plugin panicked during method call")]
    Panic,

    #[error("buffer too small")]
    BufferTooSmall,

    #[error("method not implemented (capability bit {bit} not set)")]
    NotImplemented { bit: u32 },
}
