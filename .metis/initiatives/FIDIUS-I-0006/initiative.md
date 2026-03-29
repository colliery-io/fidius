---
id: source-packages-distributable
level: initiative
title: "Source Packages — Distributable Plugin Source with Schema Validation"
short_code: "FIDIUS-I-0006"
created_at: 2026-03-29T13:37:09.050544+00:00
updated_at: 2026-03-29T13:37:09.050544+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: XL
initiative_id: source-packages-distributable
---

# Source Packages — Distributable Plugin Source with Schema Validation

## Context

Fidius handles the plugin lifecycle from trait definition through FFI calling, but has no story for *distribution*. Plugin providers compile cdylibs locally and hand them to consumers — which breaks across architectures and provides no metadata contract.

This initiative adds a **source package** layer: plugin providers distribute source code with a TOML manifest, and plugin consumers validate the manifest against a host-defined schema, then compile locally. This solves cross-architecture distribution and enables host-controlled metadata validation.

See FIDIUS-A-0001 for the full design decision.

## Goals & Non-Goals

**Goals:**
- `PackageManifest<T>` type in fidius-core — generic over host-defined metadata schema, parses `package.toml`
- Fixed header fields: name, version, interface, interface_version, dependencies
- Host-defined `[metadata]` section validated via serde deserialization into `T`
- `fidius package validate <dir>` CLI command — validates manifest against schema
- `fidius package build <dir>` CLI command — `cargo build --release` inside the package dir
- `fidius package sign <dir>` CLI command — signs `package.toml` with Ed25519
- Package discovery: scan a directory for valid packages (dirs containing `package.toml`)
- Integration with existing fidius-host loading: build produces cdylib, host loads it

**Non-Goals:**
- Package registry / remote distribution (future work)
- Automatic dependency resolution and fetching (future work — declare deps now, resolve later)
- WASM compilation target
- Binary package distribution

## Use Cases

### Use Case 1: Plugin Provider Ships a Package
- **Actor**: Plugin developer
- **Scenario**: Writes plugin source, creates `package.toml` with metadata, signs the manifest, distributes as a tarball or git repo
- **Expected Outcome**: Consumer receives a directory with source + signed manifest

### Use Case 2: Host Validates and Builds a Package
- **Actor**: Host application developer
- **Scenario**: Receives a package directory, runs `fidius package validate` to check manifest against their schema, runs `fidius package build` to compile, loads the resulting dylib via `PluginHost`
- **Expected Outcome**: Plugin is loaded and callable, with host-specific metadata available

### Use Case 3: Schema Mismatch Rejection
- **Actor**: Host application
- **Scenario**: Package manifest is missing a required metadata field that the host's schema demands
- **Expected Outcome**: `fidius package validate` fails with a clear serde error showing which field is missing

## Detailed Design

### Package Manifest (`package.toml`)

```toml
[package]
name = "blur-filter"
version = "1.2.0"
interface = "image-filter-api"
interface_version = 1

[dependencies]
base-filters = ">=1.0"

[metadata]
# Host-defined fields — validated via serde
category = "image-processing"
min_host_version = "2.0"
tags = ["blur", "gaussian"]
```

### Core Types

```rust
// fidius-core/src/package.rs
#[derive(Deserialize)]
pub struct PackageManifest<M> {
    pub package: PackageHeader,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
    pub metadata: M,
}

#[derive(Deserialize)]
pub struct PackageHeader {
    pub name: String,
    pub version: String,
    pub interface: String,
    pub interface_version: u32,
    pub source_hash: Option<String>,
}
```

### CLI Commands

```
fidius package validate <dir> [--schema-check]
fidius package build <dir> [--release]
fidius package sign --key <secret> <dir>
fidius package verify --key <public> <dir>
fidius package inspect <dir>
```

### Host-Side Integration

```rust
// Host application
#[derive(Deserialize)]
struct MySchema {
    category: String,
    min_host_version: String,
}

let manifest = fidius_host::load_package_manifest::<MySchema>("./packages/blur/")?;
// manifest.package.name == "blur-filter"
// manifest.metadata.category == "image-processing"

fidius_host::build_package("./packages/blur/")?;
// Now load the compiled dylib
let host = PluginHost::builder()
    .search_path("./packages/blur/target/release/")
    .build()?;
```

### Signing Flow

Provider signs `package.toml`:
```
fidius package sign --key mykey.secret ./blur-filter/
→ writes ./blur-filter/package.toml.sig
```

Consumer verifies before building:
```
fidius package verify --key author.public ./blur-filter/
→ verifies package.toml against package.toml.sig
```

## Alternatives Considered

See FIDIUS-A-0001 for the full alternatives analysis. Binary distribution, archive formats, and WASM were considered and rejected for v1.

## Testing Strategy

- Unit tests: `PackageManifest<T>` deserialization with valid/invalid TOML
- Unit tests: schema validation — missing fields, wrong types, extra fields
- Integration tests: create a test package, validate, build, load the resulting dylib
- CLI tests: `fidius package validate/build/sign/verify/inspect` via assert_cmd
- Signing round-trip: sign → verify → tamper → verify fails

## Implementation Plan

1. `PackageManifest<T>` and `PackageHeader` types in fidius-core
2. Manifest parsing and schema validation
3. CLI `package validate` command
4. CLI `package build` command (wraps cargo build)
5. CLI `package sign/verify` commands (reuse existing signing)
6. CLI `package inspect` command
7. Host-side `load_package_manifest::<T>()` helper
8. Package discovery (scan dir for `package.toml` files)
9. Integration tests + documentation