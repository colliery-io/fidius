---
id: 001-package-format-and-schema
level: adr
title: "Package Format and Schema Validation Design"
number: 1
short_code: "FIDIUS-A-0001"
created_at: 2026-03-29T13:37:09.658038+00:00
updated_at: 2026-04-17T13:17:23.221941+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-1: Package Format and Schema Validation Design

## Context

Fidius currently operates at the plugin level: define a trait, compile to cdylib, load at runtime. But distribution is unaddressed — there's no standard way to package, distribute, validate, and build plugin source code.

The cloacina project uses "packaged workflows" where source code is distributed with metadata and compiled on the target machine. This solves cross-architecture distribution (no need to pre-compile for every target) and enables host-defined metadata schemas.

Key requirements:
- Plugin providers distribute **source code**, not compiled binaries
- Plugin consumers (hosts) **compile locally**, solving cross-arch
- The host project defines a **manifest schema** — packages must conform to it
- Packages can **declare dependencies** on other packages
- The source package is **signed** by the provider; the consumer verifies before building
- Schema validation uses **serde deserialization** — if the manifest doesn't deserialize into the host's schema type, it's rejected

## Decision

### Package Format

A package is a directory (or tarball of a directory) with this structure:

```
my-package/
├── package.toml        # Manifest — conforms to host-defined schema
├── Cargo.toml          # Standard Rust crate, crate-type = ["cdylib"]
├── src/
│   └── lib.rs          # Plugin implementations
└── package.toml.sig    # Ed25519 signature over package.toml (optional)
```

The manifest is TOML. It has a **fixed header** (fields fidius always requires) and an **extensible metadata section** (fields the host's schema defines).

### Fixed Header

```toml
[package]
name = "blur-filter"
version = "1.2.0"
interface = "image-filter-api"        # Which interface crate this implements
interface_version = 1                  # Expected interface version

[dependencies]
# Other packages this depends on (name = version requirement)
base-filters = ">=1.0"
```

### Host-Defined Metadata (Schema)

The host defines a Rust type that the `[metadata]` section must deserialize into:

```rust
// In the host application
#[derive(Deserialize)]
struct MyPackageMetadata {
    category: String,
    min_host_version: String,
    #[serde(default)]
    tags: Vec<String>,
}
```

```toml
# In the package's package.toml
[metadata]
category = "image-processing"
min_host_version = "2.0"
tags = ["blur", "gaussian"]
```

Schema validation = `toml::from_str::<PackageManifest<MyPackageMetadata>>(content)?`. If deserialization fails, the package doesn't conform.

### Signing

The provider signs `package.toml` (not the source tree or the compiled binary). The consumer verifies the manifest signature before building. This establishes trust in the metadata and dependency declarations.

Source integrity can optionally be extended with a content hash in the manifest:

```toml
[package]
# ...
source_hash = "sha256:abc123..."   # Hash of src/ directory contents
```

### Build Flow

```
Provider Side:
  1. Write plugin source + package.toml
  2. fidius package sign --key secret.key ./my-package/
  3. Distribute (tarball, git, file copy)

Consumer Side:
  1. fidius package validate ./my-package/ --schema MyPackageMetadata
  2. fidius package build ./my-package/
     → cargo build --release inside the package dir
     → produces target/release/libmy_package.dylib
  3. Load via existing fidius-host machinery
```

## Alternatives Analysis

| Option | Pros | Cons | Risk Level | Implementation Cost |
|--------|------|------|------------|-------------------|
| Source packages (chosen) | Cross-arch, verifiable source, host-defined schemas | Requires Rust toolchain on consumer, slower than binary | Low | Medium |
| Binary packages | Fast to load, no build step | Cross-arch nightmare, larger artifacts, can't inspect source | High | Medium |
| Archive format (.fidius) | Single distributable file, cleaner than a directory | Extra pack/unpack step, complexity | Low | High |
| WASM packages | Universal binary, sandboxed | Performance overhead, limited Rust support, different ABI | High | High |

## Rationale

Source distribution is the right default for a Rust plugin framework because:

1. **Cross-architecture is solved automatically** — the consumer compiles for their target
2. **Auditability** — consumers can inspect the source before building
3. **Host-controlled compilation** — the host can set optimization flags, features, etc.
4. **Schema flexibility** — each host project defines its own metadata requirements; no global schema needed
5. **Signing the manifest** — simpler than signing arbitrary source trees; the manifest declares what the package is and the consumer can verify that declaration came from a trusted author

The archive format (.fidius tarball) is a good future extension but not needed for v1 — directories work fine.

## Consequences

### Positive
- No cross-compilation problem
- Host projects get full control over package metadata
- Existing signing infrastructure (Ed25519) is reused
- Source auditability
- Dependency declarations enable future dependency resolution

### Negative
- Consumer must have a Rust toolchain installed
- Build step adds latency compared to binary distribution
- Source distribution means plugin providers expose their implementation
- Dependency resolution is not trivial (future work)

### Neutral
- `package.toml` is a new file format to document and maintain
- The `fidius-cli` grows with `package` subcommands
- The fixed header fields (`name`, `version`, `interface`, `dependencies`) establish a contract that's hard to change later