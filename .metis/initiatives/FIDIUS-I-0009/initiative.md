---
id: fid-archive-format-compressed
level: initiative
title: ".fid Archive Format — Compressed Source Package Distribution"
short_code: "FIDIUS-I-0009"
created_at: 2026-04-01T00:05:50.285925+00:00
updated_at: 2026-04-17T13:17:20.046847+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: S
initiative_id: fid-archive-format-compressed
---

# .fid Archive Format — Compressed Source Package Distribution Initiative

## Context

Fidius packages are currently source directories with a `package.toml` manifest, but there's no standard way to distribute them as a single file. Plugin authors need to zip/tar things manually, and host applications have no built-in way to accept a single distributable artifact.

Since packages are source code (built by the host for the target architecture), a compressed source archive is the natural distribution unit. This enables publishing to GitHub Releases, artifact stores, or any file-based distribution channel.

## Goals & Non-Goals

**Goals:**
- Define `.fid` as the canonical fidius package archive format (tar + bzip2)
- `fidius package pack` creates a `.fid` from a package directory
- `fidius package unpack` extracts a `.fid` to a destination directory
- Default output naming: `{name}-{version}.fid`
- Archives include all source files (same set `package_digest` walks — excludes `target/`, `.git/`, `*.sig`)
- `package.sig` is included if present (signing happens before packing)
- Host-side `unpack_package()` API for programmatic extraction
- Warn when packing an unsigned package (no `package.sig` present)

**Non-Goals:**
- Binary distribution (packages are source; host builds for target arch)
- Package registry or index service
- Compression format options (bz2 only, no gz/zstd/etc.)
- Streaming/partial extraction

## Detailed Design

### Archive format

A `.fid` file is a bzip2-compressed tar archive. The archive contains a single top-level directory named `{name}-{version}/` with the package contents inside:

```
blur-filter-1.0.0/
├── package.toml
├── package.sig        # if signed before packing
├── Cargo.toml
└── src/
    └── lib.rs
```

### File selection

Reuse the same exclusion rules as `collect_files` in `fidius-core/src/package.rs`: skip `target/`, `.git/`. Unlike `package_digest`, `.sig` files **are** included in the archive (they are part of the distribution).

### New dependencies

- `tar` crate on `fidius-core` (archive creation/extraction)
- `bzip2` crate on `fidius-core` (compression/decompression)

### Code changes

**`fidius-core/src/package.rs`:**
- `pack_package(dir: &Path, output: &Path) -> Result<PathBuf, PackageError>` — create `.fid` archive
- `unpack_package(archive: &Path, dest: &Path) -> Result<PathBuf, PackageError>` — extract and validate manifest exists
- New `PackageError` variants for archive-related failures

**`fidius-host/src/package.rs`:**
- Re-export or thin wrapper for `unpack_package` for host-side consumption

**`fidius-cli/src/main.rs` + `commands.rs`:**
- `fidius package pack <dir> [--output <path>]` — pack a package directory into a `.fid`
- `fidius package unpack <archive> [--dest <path>]` — extract a `.fid` archive

### CLI UX

```
$ fidius package pack ./blur-filter/
Packed: blur-filter-1.0.0.fid (12.4 KB)

$ fidius package unpack blur-filter-1.0.0.fid --dest ./plugins/
Unpacked: ./plugins/blur-filter-1.0.0/
```

## Alternatives Considered

- **tar.gz / tar.bz2 with standard extensions**: Rejected in favor of a single branded `.fid` extension that's unambiguous. bz2 chosen as the sole compression format.
- **Binary distribution archives**: Rejected because fidius packages are source code built by the host application, which naturally handles cross-architecture distribution.
- **Multiple compression options**: Rejected to keep the format simple — one extension, one format, no negotiation.

## Implementation Plan

1. Add `tar` and `bzip2` workspace deps
2. Implement `pack_package` and `unpack_package` in `fidius-core`
3. Add host-side wrapper in `fidius-host`
4. Add CLI subcommands `pack` and `unpack`
5. Unit tests for pack/unpack round-trip, manifest validation on unpack
6. Integration test in the full pipeline (scaffold → package → sign → pack → unpack → build → load)