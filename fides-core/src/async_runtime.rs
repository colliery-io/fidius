//! Per-dylib async runtime for plugin methods.
//!
//! When the `async` feature is enabled, this module provides a lazily-initialized
//! tokio runtime shared across all plugin implementations in the dylib.
//! The generated shims call `FIDES_RUNTIME.block_on(...)` for async methods.

/// The shared tokio runtime for this dylib.
///
/// Initialized on first use. One runtime per dylib, shared across all
/// plugin implementations.
pub static FIDES_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
    std::sync::LazyLock::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create fides async runtime")
    });
