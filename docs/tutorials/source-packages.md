<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# Tutorial: Source Packages

This tutorial walks you through creating, validating, signing, distributing, and
building a **source package**. By the end you will have a working package that a
consumer can build and load into a host application.

A **source package** is a directory containing plugin source code and a
`package.toml` manifest. The manifest declares the package name, version,
interface compatibility, optional dependencies, and an extensible `[metadata]`
section that hosts can validate against a custom schema. Source packages are
distributed as source and compiled on the consumer side — this avoids
cross-platform binary compatibility issues.

## Prerequisites

- Rust toolchain installed
- `fidius-cli` installed (`cargo install fidius-cli`)
- Familiarity with creating a plugin (see [Your First Plugin](your-first-plugin.md))

## 1. Create the interface crate

Start by scaffolding an interface crate for a simple calculator:

```bash
fidius init-interface calculator-interface --trait Calculator
```

Edit `calculator-interface/src/lib.rs` to define the methods:

```rust
pub use fidius::{plugin_impl, PluginError};

#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Calculator: Send + Sync {
    fn add(&self, input: String) -> String;
    fn multiply(&self, input: String) -> String;
}
```

## 2. Create the plugin crate

Scaffold a plugin that implements the calculator interface:

```bash
fidius init-plugin calc-plugin --interface calculator-interface --trait Calculator
```

Implement the trait in `calc-plugin/src/lib.rs`:

```rust
use calculator_interface::{plugin_impl, Calculator, PluginError};

pub struct MyCalculator;

#[plugin_impl(Calculator)]
impl Calculator for MyCalculator {
    fn add(&self, input: String) -> String {
        // Parse "a,b" and return the sum
        let parts: Vec<f64> = input.split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        let sum: f64 = parts.iter().sum();
        sum.to_string()
    }

    fn multiply(&self, input: String) -> String {
        let parts: Vec<f64> = input.split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        let product: f64 = parts.iter().product();
        product.to_string()
    }
}

fidius_core::fidius_plugin_registry!();
```

## 3. Add a package manifest

Create `calc-plugin/package.toml`:

```toml
[package]
name = "calc-plugin"
version = "1.0.0"
interface = "calculator-interface"
interface_version = 1

[dependencies]
calculator-interface = ">=0.1"

[metadata]
category = "math"
description = "Basic arithmetic operations"
```

The `[package]` section contains fixed header fields required by fidius. The
`[metadata]` section is free-form — hosts can validate it against a custom
schema type (see [Create a Package Schema](../how-to/create-package-schema.md)).

## 4. Validate the manifest

Use the CLI to check that the manifest is well-formed:

```bash
fidius package validate ./calc-plugin
```

Expected output:

```
Package: calc-plugin v1.0.0
  Interface: calculator-interface (version 1)
  Dependencies:
    calculator-interface = ">=0.1"
  Metadata: 2 field(s)

Manifest valid.
```

## 5. Inspect the package

To see all metadata including the `[metadata]` section values:

```bash
fidius package inspect ./calc-plugin
```

Expected output:

```
Package: ./calc-plugin
  Name: calc-plugin
  Version: 1.0.0
  Interface: calculator-interface
  Interface version: 1
  Dependencies:
    calculator-interface = ">=0.1"
  Metadata:
    category = "math"
    description = "Basic arithmetic operations"
```

## 6. Build the package

Build the plugin cdylib from source:

```bash
fidius package build ./calc-plugin
```

This runs `cargo build --release` inside the package directory. Pass `--debug`
to build in debug mode instead:

```bash
fidius package build ./calc-plugin --debug
```

## 7. Sign the package

Generate a keypair and sign the package manifest:

```bash
fidius keygen --out publisher
fidius package sign --key publisher.secret ./calc-plugin
```

This signs the `package.toml` file and writes a `.sig` file alongside it.

## 8. Pack the package for distribution

Pack the signed package into a `.fid` archive for distribution:

```bash
fidius package pack ./calc-plugin
```

Expected output:

```
Packed: calc-plugin-1.0.0.fid (12.4 KB)
```

The `.fid` file is a bzip2-compressed tar archive containing all source files
plus the `package.sig`. Distribute this single file via GitHub Releases, artifact
stores, or any file-based channel.

> **Note:** If you pack without signing first, a warning is emitted:
> `warning: package is unsigned (no package.sig found)`

### Custom file extensions

Interface authors can define a custom extension via
`fidius init-interface --extension <ext>`. This writes a `fidius.toml` in the
interface crate, which `fidius init-plugin` propagates into the plugin's
`package.toml`. For example, with `extension = "cloacina"`, the pack command
produces `calc-plugin-1.0.0.cloacina` instead of `.fid`.

## 9. Consumer: unpack, verify, and build

On the consumer side, unpack the archive, verify the signature, and build:

```bash
fidius package unpack calc-plugin-1.0.0.fid --dest ./plugins
fidius package verify --key publisher.public ./plugins/calc-plugin-1.0.0
fidius package build ./plugins/calc-plugin-1.0.0
```

## 10. Consumer: load the built plugin

In the host application, use `PluginHost` to load the compiled dylib:

```rust
use fidius_host::host::PluginHost;
use fidius_host::types::LoadPolicy;

let host = PluginHost::builder()
    .search_path("./calc-plugin/target/release/")
    .load_policy(LoadPolicy::Strict)
    .build()?;

let plugin = host.load("MyCalculator")?;
```

With `LoadPolicy::Strict` (the default), signature and validation failures are
hard errors. Use `LoadPolicy::Lenient` during development to downgrade signature
failures to warnings.

## What you learned

- A source package is a directory with plugin source and a `package.toml`
- `fidius package validate` checks the manifest structure
- `fidius package inspect` shows all manifest fields
- `fidius package build` compiles the cdylib from source
- `fidius package sign` / `fidius package verify` provide integrity checking
- `fidius package pack` creates a distributable `.fid` archive (warns if unsigned)
- `fidius package unpack` extracts an archive for building
- Interface authors can set a custom archive extension via `--extension`
- `LoadPolicy::Strict` vs `LoadPolicy::Lenient` controls enforcement at load time

## Next steps

- [Create a Package Schema](../how-to/create-package-schema.md) — validate
  `[metadata]` against a host-defined Rust type
- [Package Manifest Reference](../reference/package-manifest.md) — complete
  `package.toml` format and `PackageError` variants
- [Signing and Verifying Plugins](signing-plugins.md) — deep dive into
  Ed25519 signing
