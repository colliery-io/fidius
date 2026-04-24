---
id: safe-archive-extraction-in-unpack
level: task
title: "Safe archive extraction in unpack_package — validate entries before extracting"
short_code: "FIDIUS-T-0084"
created_at: 2026-04-22T12:43:00.148314+00:00
updated_at: 2026-04-22T13:09:58.726610+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Safe archive extraction in unpack_package — validate entries before extracting

## Type

Feature / security hardening.

## Summary

`fidius_core::package::unpack_package` currently calls `tar::Archive::unpack()` directly, which trusts paths and entry types inside the archive. This exposes every consumer (cloacina has ~8 call sites) to path traversal, symlink-overwrite, and decompression-bomb attacks on any code path that accepts a `.fid` / `.cloacina` archive from a less-than-fully-trusted source.

Ask: validate each entry before extracting, rejecting dangerous entries and enforcing a size budget.

## Current Behavior

`crates/fidius-core/src/package.rs:312`:

```rust
let decoder = BzDecoder::new(file);
let mut tar = tar::Archive::new(decoder);
tar.unpack(dest)?;
```

No checks on entry type, no path validation, no size cap.

## Attack Classes

1. **Path traversal** — an entry named `../../etc/cron.d/pwn` can write outside `dest`. The `tar` crate's implicit guards are best-effort and platform-dependent.
2. **Symlink / hardlink overwrite** — a two-entry archive (`foo -> /etc/passwd`, then regular `foo` with malicious content) causes the extractor to follow the link and overwrite an arbitrary file the process can write.
3. **Absolute paths** — entries like `/etc/passwd` can bypass `dest` on some tar implementations.
4. **Decompression bomb** — a few-KB bzip2 archive can expand to GB/TB, filling disk or OOMing the host. No ratio or absolute limit is enforced today.

## Proposed Behavior

Replace the single `unpack` call with a validating loop:

```rust
const MAX_DECOMPRESSED: u64 = 500 * 1024 * 1024; // 500 MB
const MAX_RATIO: u64 = 10;                        // decompressed / compressed

let compressed_size = std::fs::metadata(archive)?.len();
let mut total: u64 = 0;

for entry in tar.entries()? {
    let mut entry = entry?;
    let path = entry.path()?.into_owned();

    // 1. Reject link entries — prevents symlink-overwrite attacks
    let ty = entry.header().entry_type();
    if ty.is_symlink() || ty.is_hard_link() {
        return Err(PackageError::InvalidArchive(
            format!("archive contains link entry '{}' — rejected", path.display())
        ));
    }

    // 2. Reject absolute paths and parent-dir components
    for c in path.components() {
        match c {
            std::path::Component::ParentDir => {
                return Err(PackageError::InvalidArchive(
                    format!("archive entry '{}' contains '..' — rejected", path.display())
                ));
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(PackageError::InvalidArchive(
                    format!("archive entry '{}' is absolute — rejected", path.display())
                ));
            }
            _ => {}
        }
    }

    // 3. Enforce cumulative size budget (catches bombs early)
    total = total.saturating_add(entry.header().size().unwrap_or(0));
    if total > MAX_DECOMPRESSED || total > compressed_size.saturating_mul(MAX_RATIO) {
        return Err(PackageError::InvalidArchive(
            format!("decompressed size {} exceeds limits", total)
        ));
    }

    // 4. Safe to extract
    entry.unpack_in(dest)?;
}
```

## Design Notes / Open Questions

- **Configurable limits.** Suggest `const` defaults matching the values above, but expose a builder / `UnpackOptions { max_decompressed, max_ratio }` so consumers with legitimate large archives (e.g. vendored Python deps) can raise the cap. Cloacina's tenant-upload path wants the strict default; cloacina-compiler building a local package might want it higher.
- **Partial-extract cleanup.** If validation fails mid-archive, files already written to `dest` remain. Either (a) extract into a temp subdir and rename on success, or (b) document that callers pass a throwaway `dest` and clean up on error. (a) is nicer but changes the signature semantics.
- **Error variants.** Consider distinct error kinds (`SymlinkRejected`, `PathTraversal`, `SizeLimitExceeded`) rather than folding all into `InvalidArchive` — consumers may want to surface different messages to users vs. audit logs.
- **Symlink policy.** Outright rejection is the safest default. If there's a legitimate use case for symlinks inside `.fid` packages, the alternative is to resolve and validate each symlink target stays within `dest` — but this is easy to get wrong (TOCTOU between validation and extraction). Recommend starting with rejection.
- **Platform.** The checks are portable. Windows has additional concerns with reserved names (`CON`, `PRN`, etc.) and alternate data streams (`foo:bar`) — optional hardening, not blocking.

## Prior Art

The pre-fidius version of this lived in cloacina itself (commit `eeebd80` on `archive/cloacina-server-week1`, task T-0220) and did exactly the four-rule loop above. When cloacina adopted fidius for packaging, those checks were dropped because the assumption was that fidius would own safe extraction. This FR restores that assumption.

## Impact on Cloacina

Once fidius ships a version with the safe extractor (enabled by default), cloacina bumps the `fidius-core` dep and the regression closes for all ~8 call sites at once. No cloacina-side code changes needed beyond the version bump.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `unpack_package` rejects entries with `..` components.
- [ ] `unpack_package` rejects entries with absolute paths (root or drive prefix).
- [ ] `unpack_package` rejects symlink and hardlink entries.
- [ ] `unpack_package` enforces a cumulative decompressed-size cap and a compressed-to-decompressed ratio cap, rejecting the archive as soon as either is exceeded.
- [ ] Default limits are strict (suggest 500 MB / 10× ratio) and applied when callers don't override.
- [ ] An `UnpackOptions`-style knob (or equivalent) exists for callers that need higher caps.
- [ ] Error types distinguish the rejection reason clearly enough for consumers to surface useful messages.
- [ ] Partial-extract cleanup policy is either implemented (extract-to-temp-then-rename) or explicitly documented.
- [ ] All test cases listed below pass.
- [ ] Release notes flag the behavior change and any caller-visible error surface changes.

## Test Cases

- Archive with `../foo` entry → rejected.
- Archive with absolute `/tmp/foo` entry → rejected.
- Archive with symlink entry → rejected.
- Archive with hardlink entry → rejected.
- Two-entry symlink-then-file attack → rejected at entry 1, second entry never reached.
- Bomb archive (small compressed, huge declared size) → rejected on first oversize entry.
- Normal package → extracts identically to current behavior.

## Related

- Consumer: cloacina, ~8 `unpack_package` call sites; regression closes on version bump.
- Tangentially related to FIDIUS-I-0006 / FIDIUS-I-0009 (.fid archive format) — this is hardening of that delivery path, not a format change.

## Status Updates

### 2026-04-22 — design decisions locked

Resolving the five open questions in the FR before writing code:

- **API shape.** Keep `unpack_package(archive, dest)` with strict defaults (fully backward-compatible signature). Add `unpack_package_with_options(archive, dest, &UnpackOptions)` for callers that need higher caps. Both return `Result<PathBuf, PackageError>` unchanged.
- **`UnpackOptions` fields.** `max_decompressed: u64` (default 500 MB), `max_ratio: u64` (default 10), `max_entries: u32` (default 10_000 — catches inode-exhaustion attacks via zillions-of-empty-files archives, a case the FR didn't enumerate but which follows the same reasoning).
- **Error variants.** Add distinct variants: `PathTraversal`, `AbsolutePath`, `SymlinkRejected`, `HardlinkRejected`, `SizeLimitExceeded`, `TooManyEntries`. Keep `InvalidArchive` for the "no package.toml" case.
- **Partial-extract cleanup.** Option (a): extract into a `tempfile::TempDir` created inside `dest`, atomically `rename` the package subdirectory to its final location on success. On any validation failure, the `TempDir` drops and everything cleans up. Adds runtime dep on `tempfile` in `fidius-core`.
- **Bomb defence depth.** Cumulative declared-size check (from tar headers) + `max_entries` count. Tar's own parsing enforces actual bytes match declared size, so declared-size is a sufficient proxy without adding a LimitedReader wrapper.
- **Symlink policy.** Outright rejection.
- **Windows hardening.** Out of scope here.

### 2026-04-22 — implementation landed

Changes:

- `crates/fidius-core/src/package.rs`: added `UnpackOptions` struct with strict defaults (500 MB / 10× ratio / 10,000 entries); added six new `PackageError` variants (`PathTraversal`, `AbsolutePath`, `SymlinkRejected`, `HardlinkRejected`, `SizeLimitExceeded`, `TooManyEntries`); rewrote `unpack_package` as a thin wrapper over new `unpack_package_with_options`; extraction now stages into a `tempfile::TempDir` under `dest` and atomically renames the package directory on success so rejected archives leave no trace.
- `crates/fidius-host/src/package.rs`: updated `unpack_fid` doc comment describing the new safety guarantees.
- `Cargo.toml`, `crates/fidius-core/Cargo.toml`: promoted `tempfile` to a workspace dep and a runtime dep of `fidius-core`.

Tests added (all passing, 25/25 in `package::` module):

- `unpack_rejects_parent_dir_component` — archive with `../escaped` entry rejected with `PathTraversal`.
- `unpack_rejects_absolute_path` — archive with `/tmp/...` entry rejected with `AbsolutePath`.
- `unpack_rejects_symlink` / `unpack_rejects_hardlink` — link entries rejected with their dedicated variants.
- `unpack_symlink_then_file_rejected_at_first_entry` — the classic overwrite attack; sentinel file outside extract dir verified untouched.
- `unpack_rejects_declared_size_bomb` — 600 MB declared size triggers `SizeLimitExceeded` under default cap.
- `unpack_rejects_ratio_bomb` — tight `max_ratio` triggers `SizeLimitExceeded` even when absolute cap is disabled.
- `unpack_rejects_too_many_entries` — `max_entries=10` on a 50-entry archive triggers `TooManyEntries`.
- `unpack_staging_cleans_up_on_rejection` — confirms `dest` is empty after a rejected archive.
- `unpack_with_options_accepts_large_archive` — round-trip under loosened options exercises the options path end-to-end.

Caller impact: `fidius-host::unpack_fid` and `fidius-cli` both delegate to `unpack_package` and pick up the strict defaults transparently, with no signature changes. Cloacina's ~8 call sites will inherit the hardening on the next version bump.

Verification:

- `cargo test -p fidius-core package::` → 25 passed.
- `angreal test` → all test groups green.
- `angreal lint` → clean.
- `angreal check` → clean.

Acceptance criteria status:

- [x] Rejects `..` components.
- [x] Rejects absolute paths.
- [x] Rejects symlink and hardlink entries.
- [x] Enforces cumulative decompressed-size cap and ratio cap.
- [x] Default limits strict (500 MB / 10× / 10k entries).
- [x] `UnpackOptions` provides configurable escape hatch.
- [x] Distinct error variants per rejection reason.
- [x] Partial-extract cleanup implemented via `tempfile::TempDir` staging + atomic rename.
- [x] All FR-listed test cases pass, plus `max_entries`, ratio-bomb, and staging-cleanup tests added for defence-in-depth.
- [ ] Release notes — deferred to the release that ships this change (out of scope for the task itself).