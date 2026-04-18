---
id: add-host-side-unpack-wrapper-in
level: task
title: "Add host-side unpack wrapper in fidius-host"
short_code: "FIDIUS-T-0058"
created_at: 2026-04-01T00:09:57.338047+00:00
updated_at: 2026-04-01T00:27:41.806937+00:00
parent: FIDIUS-I-0009
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0009
---

# Add host-side unpack wrapper in fidius-host

## Parent Initiative

[[FIDIUS-I-0009]]

## Objective

Expose `unpack_package` in `fidius-host/src/package.rs` so host applications can extract `.fid` archives programmatically. Follows the same pattern as existing `load_package_manifest` and `verify_package` wrappers.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `unpack_fid(archive, dest) -> Result<PathBuf, PackageError>` added to `fidius-host/src/package.rs`
- [ ] `bzip2` dep added to `fidius-host/Cargo.toml` (via workspace)
- [ ] Function re-exports or delegates to `fidius_core::package::unpack_package`
- [ ] Public API is documented with doc comment and example
- [ ] Emits `tracing::warn!` when unpacked package has no `package.sig`

## Implementation Notes

### Files to modify
- `fidius-host/Cargo.toml` — add `bzip2` dep if needed (may only need fidius-core transitive)
- `fidius-host/src/package.rs` — add thin wrapper

### Dependencies
- Blocked by FIDIUS-T-0057

## Status Updates

- 2026-03-31: Added `unpack_fid` to `fidius-host/src/package.rs`. Delegates to `fidius_core::package::unpack_package`, emits `tracing::warn!` gated behind `#[cfg(feature = "tracing")]` (tracing is optional in fidius-host). No new deps needed — fidius-core handles the bz2/tar transitively. Compiles with and without tracing feature.