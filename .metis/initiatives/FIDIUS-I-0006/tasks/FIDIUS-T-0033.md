 ---
id: host-side-package-integration-and
level: task
title: "Host-side package integration and discovery"
short_code: "FIDIUS-T-0033"
created_at: 2026-03-29T14:00:06.893631+00:00
updated_at: 2026-03-29T14:40:40.074045+00:00
parent: FIDIUS-I-0006
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0006
---

# Host-side package integration and discovery

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0006]]

## Objective

Add package-aware helpers to fidius-host so host applications can load manifests with schema validation, discover packages in a directory, and build packages programmatically. This is the Rust API that host developers use — the CLI commands (T-0031) wrap these.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `load_package_manifest::<M>(dir: &Path) -> Result<PackageManifest<M>, PackageError>` in fidius-host — loads and validates manifest with host's schema type
- [ ] `discover_packages(dir: &Path) -> Result<Vec<PathBuf>, PackageError>` — scans a directory for subdirs containing `package.toml`, returns their paths
- [ ] `build_package(dir: &Path, release: bool) -> Result<PathBuf, PackageError>` — runs `cargo build` inside the package dir, returns the path to the compiled cdylib
- [ ] The cdylib output path is platform-aware (`.dylib`/`.so`/`.dll`)
- [ ] Schema mismatch (manifest missing required metadata field) returns clear serde error via `PackageError`
- [ ] Unit tests: discover finds packages, build compiles test-plugin-smoke as a package

## Implementation Notes

### Technical Approach

File: `fidius-host/src/package.rs`

`load_package_manifest` wraps `fidius_core::package::load_manifest` — same function, just re-exported at the host level for convenience.

`build_package` shells out to `cargo build --manifest-path <dir>/Cargo.toml [--release]`, then finds the cdylib in `target/{debug,release}/`. Uses the crate name from `Cargo.toml` to predict the output filename.

`discover_packages` iterates directory entries, checks for `package.toml` in each subdir.

### Dependencies
- FIDIUS-T-0030 (core types)

## Status Updates

- **2026-03-29**: Implemented in `fidius-host/src/package.rs`. `load_package_manifest::<M>()`, `discover_packages()`, `build_package()` with platform-aware cdylib detection. Compiles clean.