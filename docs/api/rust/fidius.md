# fidius <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Fidius — a Rust plugin framework for trait-to-dylib plugin systems.

This is the facade crate. Interface crates should depend on `fidius` only.
It re-exports everything needed to define interfaces and implement plugins.

