# fidius-core::async_runtime <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Per-dylib async runtime for plugin methods.

When the `async` feature is enabled, this module provides a lazily-initialized
tokio runtime shared across all plugin implementations in the dylib.
The generated shims call `FIDIUS_RUNTIME.block_on(...)` for async methods.

