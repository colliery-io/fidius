<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# How To: White-Label a Plugin Interface

This guide explains how to create a plugin interface crate that hides fidius
as an implementation detail. Plugin authors depend only on your interface
crate -- they never see or import `fidius` directly.

## Why white-label?

When building a plugin ecosystem (e.g., "Cloacina plugins"), you want plugin
authors to write:

```toml
[dependencies]
cloacina-plugin = "1.0"
```

Not:

```toml
[dependencies]
cloacina-plugin = "1.0"
fidius = "0.1"  # implementation detail leak
```

## Step 1: Create the interface crate

```bash
fidius init-interface cloacina-plugin --trait CloacinaPlugin --extension cloacina
```

## Step 2: Re-export fidius and use `crate = "..."`

Edit `cloacina-plugin/src/lib.rs`:

```rust
// Re-export fidius so plugin crates can access it through us
pub use fidius;

// Use crate = "crate" so generated code resolves fidius through
// this crate's re-export, not a direct fidius dependency
#[fidius::plugin_interface(version = 1, buffer = PluginAllocated, crate = "crate")]
pub trait CloacinaPlugin: Send + Sync {
    fn execute(&self, input: String) -> String;
    fn version(&self) -> String;
}

// Re-export the proc macros so plugin authors can use them
pub use fidius::plugin_impl;
pub use fidius::PluginError;
```

The `crate = "crate"` attribute tells the generated companion module to resolve
fidius types through `crate::fidius::` instead of `fidius::` directly.

## Step 3: Plugin authors use your crate only

A plugin author's `Cargo.toml`:

```toml
[package]
name = "my-cloacina-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
cloacina-plugin = "1.0"
```

Their `src/lib.rs`:

```rust
use cloacina_plugin::{plugin_impl, CloacinaPlugin, PluginError, __fidius_CloacinaPlugin};

pub struct MyPlugin;

#[plugin_impl(CloacinaPlugin, crate = "cloacina_plugin::fidius")]
impl CloacinaPlugin for MyPlugin {
    fn execute(&self, input: String) -> String {
        format!("processed: {input}")
    }

    fn version(&self) -> String {
        "1.0.0".to_string()
    }
}

cloacina_plugin::fidius::fidius_plugin_registry!();
```

Key points:

- `#[plugin_impl(CloacinaPlugin, crate = "cloacina_plugin::fidius")]` tells the
  shim codegen to resolve fidius types through the interface crate's re-export.
- `cloacina_plugin::fidius::fidius_plugin_registry!()` emits the registry
  symbol using the re-exported `fidius_plugin_registry!` macro (which uses
  `$crate::` internally, so it resolves correctly).
- The plugin's `Cargo.toml` has **no direct `fidius` dependency**.

## Custom file extension

If you used `--extension cloacina` when scaffolding, the `fidius.toml` in
your interface crate propagates the extension to plugins via `init-plugin`.
When plugins are packed with `fidius package pack`, the archive uses
`.cloacina` instead of `.fid`.

## See Also

- [Macro Reference](../api/rust/fidius-macro.md) -- `crate` attribute on `plugin_interface` and `plugin_impl`
- [Package Manifest Reference](../reference/package-manifest.md) -- `extension` field
- [ABI Specification](../reference/abi-specification.md) -- argument tuple encoding
