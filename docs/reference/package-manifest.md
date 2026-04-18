<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# Package Manifest Reference

Complete reference for the `package.toml` manifest format, the `PackageError`
type, and the `fidius package` CLI commands.

**Source:** `crates/fidius-core/src/package.rs`, `crates/fidius-host/src/package.rs`,
`crates/fidius-cli/src/commands.rs`

---

## `package.toml` Format

A source package is a directory containing plugin source code and a
`package.toml` manifest. The manifest has three sections: a fixed `[package]`
header, an optional `[dependencies]` section, and a host-defined `[metadata]`
section.

### `[package]` — Required header fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Package name (e.g., `"blur-filter"`). |
| `version` | string | yes | Package version (e.g., `"1.2.0"`). |
| `interface` | string | yes | Name of the interface crate this package implements. |
| `interface_version` | integer | yes | Expected interface version number. |
| `extension` | string | no | Custom file extension for `.fid` archives (e.g., `"cloacina"`). Defaults to `"fid"` when absent. Set by the interface author via `fidius init-interface --extension`. |
| `source_hash` | string | no | SHA-256 hash of the source directory contents. Used for integrity verification. |

Example:

```toml
[package]
name = "calc-plugin"
version = "1.0.0"
interface = "calculator-interface"
interface_version = 1
extension = "cloacina"
source_hash = "a1b2c3d4..."
```

### `[dependencies]` — Package dependencies

Maps dependency names to version requirement strings. This section is optional
and defaults to an empty map.

```toml
[dependencies]
base-utils = ">=1.0"
helper = "0.5"
```

Dependencies are stored as a `BTreeMap<String, String>` (sorted by key).

### `[metadata]` — Host-defined metadata

Free-form section validated by the host application against a schema type. When
the host calls `load_package_manifest::<M>(dir)`, the `[metadata]` section must
deserialize into `M`. If it does not, parsing fails with
`PackageError::ParseError`.

```toml
[metadata]
category = "math"
description = "Basic arithmetic operations"
tags = ["calc", "math"]
```

See [Create a Package Schema](../how-to/create-package-schema.md) for how to
define and use a schema type.

### Complete example

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

---

## Rust types

### `PackageManifest<M>`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest<M> {
    pub package: PackageHeader,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
    pub metadata: M,
}
```

Generic over the host's metadata schema type `M`. If `M` is
`toml::Value`, any metadata is accepted (see `load_manifest_untyped`).

**Crate:** `fidius_core::package`

### `PackageHeader`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageHeader {
    pub name: String,
    pub version: String,
    pub interface: String,
    pub interface_version: u32,
    pub extension: Option<String>,
    pub source_hash: Option<String>,
}
```

Fixed fields required by fidius. Present in every `package.toml`.

The `extension()` method returns the package extension, defaulting to `"fid"`:

```rust
header.extension() // "fid" when None, or the custom value
```

### `PackResult`

```rust
#[derive(Debug)]
pub struct PackResult {
    pub path: PathBuf,
    pub unsigned: bool,
}
```

Returned by `pack_package`. Contains the path to the created archive and
whether the package was unsigned (no `package.sig` found).

**Crate:** `fidius_core::package`

---

## Functions

### `fidius_core::package::load_manifest::<M>(dir)`

Load and parse a `package.toml` from a package directory. The type parameter
`M: DeserializeOwned` is the host's metadata schema.

```rust
pub fn load_manifest<M: DeserializeOwned>(
    dir: &Path,
) -> Result<PackageManifest<M>, PackageError>
```

### `fidius_core::package::load_manifest_untyped(dir)`

Load a manifest accepting any `[metadata]` section. Uses `toml::Value` as the
metadata type.

```rust
pub fn load_manifest_untyped(
    dir: &Path,
) -> Result<PackageManifest<toml::Value>, PackageError>
```

### `fidius_host::package::load_package_manifest::<M>(dir)`

Host-side entry point. Delegates to `fidius_core::package::load_manifest`.

```rust
pub fn load_package_manifest<M: DeserializeOwned>(
    dir: &Path,
) -> Result<PackageManifest<M>, PackageError>
```

### `fidius_host::package::discover_packages(dir)`

Scan a directory for subdirectories containing `package.toml`. Returns paths
sorted alphabetically.

```rust
pub fn discover_packages(dir: &Path) -> Result<Vec<PathBuf>, PackageError>
```

### `fidius_host::package::build_package(dir, release)`

Build a package by running `cargo build` inside the package directory. Returns
the path to the compiled cdylib on success.

```rust
pub fn build_package(dir: &Path, release: bool) -> Result<PathBuf, PackageError>
```

The `release` parameter controls whether `--release` is passed to `cargo build`.
The function locates the output cdylib by scanning the target directory for a
file with the platform-appropriate extension (`.dylib` on macOS, `.so` on Linux,
`.dll` on Windows).

### `fidius_core::package::pack_package(dir, output)`

Create a `.fid` archive (tar + bzip2) from a package directory. The archive
contains a single top-level directory `{name}-{version}/` with all source files.
Excludes `target/` and `.git/` directories. Includes `package.sig` if present.

```rust
pub fn pack_package(
    dir: &Path,
    output: Option<&Path>,
) -> Result<PackResult, PackageError>
```

If `output` is `None`, the archive is written to the current directory as
`{name}-{version}.{ext}` where `ext` is the manifest's `extension` field
(defaulting to `"fid"`). Returns a `PackResult` with `unsigned: true` if
`package.sig` was not found.

### `fidius_core::package::unpack_package(archive, dest)`

Extract a `.fid` archive to a destination directory. Returns the path to the
extracted top-level package directory. Validates that `package.toml` exists.

```rust
pub fn unpack_package(
    archive: &Path,
    dest: &Path,
) -> Result<PathBuf, PackageError>
```

### `fidius_host::package::unpack_fid(archive, dest)`

Host-side wrapper for `unpack_package`. Emits a `tracing::warn!` when the
unpacked package has no `package.sig` (requires the `tracing` feature).

```rust
pub fn unpack_fid(
    archive: &Path,
    dest: &Path,
) -> Result<PathBuf, PackageError>
```

---

## `PackageError`

Errors from package manifest loading and building. Defined in
`fidius_core::package`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("package.toml not found in {path}")]
    ManifestNotFound { path: String },

    #[error("failed to parse package.toml: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("io error reading package.toml: {0}")]
    Io(#[from] std::io::Error),

    #[error("package build failed: {0}")]
    BuildFailed(String),

    #[error("package.sig not found in {path}")]
    SignatureNotFound { path: String },

    #[error("package signature invalid for {path}")]
    SignatureInvalid { path: String },

    #[error("archive error: {0}")]
    ArchiveError(String),

    #[error("invalid archive: {0}")]
    InvalidArchive(String),
}
```

| Variant | Trigger | Resolution |
|---------|---------|------------|
| `ManifestNotFound` | No `package.toml` in the given directory. | Ensure the directory contains a `package.toml` file. |
| `ParseError` | TOML syntax error or `[metadata]` schema validation failure (the metadata did not deserialize into the host's schema type `M`). | Fix the TOML syntax or add the missing metadata fields. |
| `Io` | Filesystem error reading the manifest file. | Check file permissions and that the path exists. |
| `BuildFailed` | `cargo build` returned a non-zero exit code, or `Cargo.toml` was not found in the package directory. | Fix compilation errors or ensure `Cargo.toml` is present. |
| `SignatureNotFound` | No `package.sig` in the package directory. | Sign the package with `fidius package sign`. |
| `SignatureInvalid` | No trusted key verified the signature. | Ensure the correct public key is being used. |
| `ArchiveError` | Error creating or reading a `.fid` archive. | Check file permissions and disk space. |
| `InvalidArchive` | Archive does not contain a valid package (no `package.toml`). | Ensure the archive was created with `fidius package pack`. |

See also the [Errors Reference](errors.md) for `LoadError` and other error types.

---

## CLI Commands

All package commands are under the `fidius package` subcommand group. For
argument details see the [CLI Reference](cli.md#package).

| Command | Description |
|---------|-------------|
| `fidius package validate <DIR>` | Parse and validate the manifest; print summary. |
| `fidius package build <DIR> [--debug]` | Build the cdylib from source (release by default). |
| `fidius package inspect <DIR>` | Print all manifest fields including metadata values. |
| `fidius package sign --key <SECRET_KEY> <DIR>` | Sign the `package.toml` with an Ed25519 key. |
| `fidius package verify --key <PUBLIC_KEY> <DIR>` | Verify the `package.toml` signature. |
| `fidius package pack <DIR> [--output <PATH>]` | Pack source into a `.fid` archive. Warns if unsigned. |
| `fidius package unpack <ARCHIVE> [--dest <DIR>]` | Extract a `.fid` archive. |

---

## See Also

- [Tutorial: Source Packages](../tutorials/source-packages.md) — end-to-end walkthrough
- [Create a Package Schema](../how-to/create-package-schema.md) — host-side schema validation
- [CLI Reference](cli.md#package) — full argument tables for `fidius package` commands
- [Errors Reference](errors.md) — `PackageError` and `LoadError` variants
