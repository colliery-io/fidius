<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# How-To: Create a Package Metadata Schema

This guide shows how to define a host-side schema type for validating the
`[metadata]` section of `package.toml` manifests.

## Problem

You are building a host application that loads source packages. You want to
enforce that every package provides specific metadata fields (e.g., `category`,
`min_host_version`) and reject packages that omit required fields or use
incorrect types.

## Solution

Define a Rust struct that derives `serde::Deserialize`, then pass it as the
type parameter to `load_package_manifest::<MySchema>()`.

### Step 1: Define the schema struct

```rust
use serde::Deserialize;

/// Host-defined metadata schema for plugin packages.
///
/// Every package.toml [metadata] section must deserialize into this type.
/// Missing required fields or type mismatches cause a `PackageError::ParseError`.
#[derive(Debug, Deserialize)]
pub struct PluginMetadata {
    /// Required: plugin category for the host's catalog UI.
    pub category: String,

    /// Required: minimum host version this plugin supports.
    pub min_host_version: String,

    /// Optional: human-readable description.
    #[serde(default)]
    pub description: Option<String>,

    /// Optional: list of tags for search/filtering.
    #[serde(default)]
    pub tags: Vec<String>,
}
```

Fields without `#[serde(default)]` are required — if the TOML does not contain
them, parsing fails with `PackageError::ParseError`. Fields with
`#[serde(default)]` or `Option<T>` are optional.

Extra fields in the TOML that are not present in the struct are silently ignored
by serde's default behavior.

### Step 2: Write the matching TOML

In `package.toml`:

```toml
[package]
name = "blur-filter"
version = "2.0.0"
interface = "image-plugin-api"
interface_version = 3

[metadata]
category = "image-processing"
min_host_version = "4.1.0"
description = "Gaussian blur filter"
tags = ["blur", "image", "filter"]
```

### Step 3: Load and validate

Use `fidius_host::package::load_package_manifest` with your schema type:

```rust
use std::path::Path;
use fidius_host::package::load_package_manifest;
use fidius_core::package::PackageError;

fn load_plugin_package(dir: &Path) -> Result<(), PackageError> {
    let manifest = load_package_manifest::<PluginMetadata>(dir)?;

    println!("Loaded: {} v{}", manifest.package.name, manifest.package.version);
    println!("Category: {}", manifest.metadata.category);
    println!("Min host version: {}", manifest.metadata.min_host_version);

    if let Some(desc) = &manifest.metadata.description {
        println!("Description: {}", desc);
    }

    for tag in &manifest.metadata.tags {
        println!("Tag: {}", tag);
    }

    Ok(())
}
```

If the `[metadata]` section is missing a required field (e.g., `category`),
`load_package_manifest` returns `PackageError::ParseError` with a message
identifying the missing field.

### Step 4: Discover and validate multiple packages

Use `fidius_host::package::discover_packages` to scan a directory for package
subdirectories, then validate each one:

```rust
use fidius_host::package::{discover_packages, load_package_manifest};

fn validate_all(packages_dir: &Path) -> Result<(), PackageError> {
    let package_dirs = discover_packages(packages_dir)?;

    for dir in &package_dirs {
        match load_package_manifest::<PluginMetadata>(dir) {
            Ok(manifest) => {
                println!("Valid: {} v{}", manifest.package.name, manifest.package.version);
            }
            Err(e) => {
                eprintln!("Invalid package in {}: {}", dir.display(), e);
            }
        }
    }

    Ok(())
}
```

`discover_packages` scans the given directory for subdirectories that contain a
`package.toml` file. It returns paths sorted alphabetically.

## Untyped loading

If you need to load a manifest without schema validation (e.g., in a CLI tool),
use `fidius_core::package::load_manifest_untyped`. This accepts any `[metadata]`
section by deserializing it as `toml::Value`:

```rust
use fidius_core::package::load_manifest_untyped;

let manifest = load_manifest_untyped(Path::new("./my-package/"))?;
// manifest.metadata is toml::Value — access fields dynamically
```

## See also

- [Tutorial: Source Packages](../tutorials/source-packages.md) — end-to-end
  walkthrough
- [Package Manifest Reference](../reference/package-manifest.md) — complete
  `package.toml` format
- [Error Reference](../reference/errors.md) — `PackageError` variants
