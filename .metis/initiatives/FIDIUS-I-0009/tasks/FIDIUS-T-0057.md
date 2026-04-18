ic---
id: add-tar-bzip2-deps-and-implement
level: task
title: "Add tar/bzip2 deps and implement pack_package/unpack_package in fidius-core"
short_code: "FIDIUS-T-0057"
created_at: 2026-04-01T00:09:56.009074+00:00
updated_at: 2026-04-01T00:26:46.632940+00:00
parent: FIDIUS-I-0009
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0009
---

# Add tar/bzip2 deps and implement pack_package/unpack_package in fidius-core

## Parent Initiative

[[FIDIUS-I-0009]]

## Objective

Add `tar` and `bzip2` as workspace dependencies and implement the core `pack_package` and `unpack_package` functions in `fidius-core/src/package.rs`. These are the foundation that the CLI and host crate build on.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `tar` and `bzip2` added to `[workspace.dependencies]` in root `Cargo.toml` and to `fidius-core/Cargo.toml`
- [ ] `pack_package(dir, output) -> Result<PathBuf, PackageError>` creates a `.fid` (tar+bz2) archive
- [ ] Archive contains a top-level `{name}-{version}/` directory prefix
- [ ] File selection: excludes `target/`, `.git/`; **includes** `.sig` files
- [ ] `pack_package` returns a `PackageWarning::Unsigned` or similar indicator when `package.sig` is absent
- [ ] `unpack_package(archive, dest) -> Result<PathBuf, PackageError>` extracts the archive and returns the extracted directory path
- [ ] `unpack_package` validates that a `package.toml` exists in the extracted contents
- [ ] New `PackageError` variants for archive failures (`ArchiveError`, `InvalidArchive`)
- [ ] Unit tests: pack/unpack round-trip, missing manifest on unpack fails, unsigned warning

## Implementation Notes

### Files to modify
- `Cargo.toml` (workspace) — add `tar = "0.4"` and `bzip2 = "0.5"` to `[workspace.dependencies]`
- `fidius-core/Cargo.toml` — add `tar` and `bzip2` deps
- `fidius-core/src/package.rs` — new functions and error variants

### Approach
- `pack_package`: load manifest for name/version, create `BzEncoder` wrapping a `File`, create `tar::Builder`, use a modified `collect_files` that includes `.sig` files, add each file with path prefixed by `{name}-{version}/`
- `unpack_package`: open file, `BzDecoder` → `tar::Archive`, unpack to dest, find the top-level dir, verify `package.toml` exists inside it
- Return type for pack should communicate the unsigned warning — either a `PackResult` struct with a `warnings: Vec<PackWarning>` field, or print to stderr and keep the return type simple

## Status Updates

- 2026-03-31: Added `tar = "0.4"` and `bzip2 = "0.5"` to workspace and fidius-core deps. Implemented `pack_package`, `unpack_package`, `PackResult`, `collect_archive_files`, and two new `PackageError` variants. All 13 package tests pass including 5 new ones (round-trip, sig inclusion, exclusions, invalid archive, default naming).