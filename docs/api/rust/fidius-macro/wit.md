# fidius-macro::wit <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


WIT mapping/rendering for the macro's WASM codegen.

The implementation lives in the shared `fidius-wit` crate so the same logic
backs the macro, the `build.rs` helper, and the `fidius wit` CLI
(FIDIUS-I-0023). This module re-exports the pieces the macro uses.

