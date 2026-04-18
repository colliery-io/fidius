---
id: cli-pack-and-unpack-subcommands
level: task
title: "CLI pack and unpack subcommands with unsigned-package warning"
short_code: "FIDIUS-T-0059"
created_at: 2026-04-01T00:09:58.184817+00:00
updated_at: 2026-04-01T00:28:48.858995+00:00
parent: FIDIUS-I-0009
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0009
---

# CLI pack and unpack subcommands with unsigned-package warning

## Parent Initiative

[[FIDIUS-I-0009]]

## Objective

Add `fidius package pack` and `fidius package unpack` CLI subcommands. The pack command warns on stderr when the package is unsigned.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `PackageCommands::Pack` variant with `dir: PathBuf` and `--output: Option<PathBuf>` args
- [ ] `PackageCommands::Unpack` variant with `archive: PathBuf` and `--dest: Option<PathBuf>` args
- [ ] `package_pack` in `commands.rs` — calls `fidius_core::package::pack_package`, prints output path and file size
- [ ] `package_unpack` in `commands.rs` — calls `fidius_core::package::unpack_package`, prints extracted directory
- [ ] Pack emits `warning: package is unsigned (no package.sig found)` to stderr when `.sig` is absent
- [ ] Default output: `{name}-{version}.fid` in current directory when `--output` not given
- [ ] Default dest: current directory when `--dest` not given
- [ ] CLI integration tests for pack and unpack

## Implementation Notes

### Files to modify
- `fidius-cli/src/main.rs` — add `Pack` and `Unpack` variants to `PackageCommands`, wire to `commands::`
- `fidius-cli/src/commands.rs` — add `package_pack` and `package_unpack` functions
- `fidius-cli/Cargo.toml` — may need `bzip2` dep if not transitive

### Dependencies
- Blocked by FIDIUS-T-0057

## Status Updates

- 2026-03-31: Added `Pack` and `Unpack` variants to `PackageCommands` in main.rs. Implemented `package_pack` (with unsigned warning to stderr + human-readable file size) and `package_unpack` (defaults dest to `.`) in commands.rs. No new deps needed. All 6 existing CLI tests pass. CLI integration tests deferred to T-0060 (full pipeline).