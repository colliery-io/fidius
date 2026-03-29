# fidius-core::status <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


FFI status codes returned by plugin method shims.

These `i32` values are the return type of every `extern "C"` function
in a plugin vtable. The host checks the status code before reading
the output buffer.

