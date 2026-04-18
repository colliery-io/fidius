---
id: white-label-package-extension
level: task
title: "White-label package extension — interface-defined, propagated via manifest"
short_code: "FIDIUS-T-0061"
created_at: 2026-04-01T00:32:41.538440+00:00
updated_at: 2026-04-01T00:43:13.881332+00:00
parent: FIDIUS-I-0009
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0009
---

# White-label package extension — interface-defined, propagated via manifest

## Parent Initiative

[[FIDIUS-I-0009]]

## Objective

Allow interface authors to define a custom package extension (e.g. `.cloacina` instead of `.fid`). The extension is set once by `fidius init-interface --extension`, stored in the interface crate, propagated into `package.toml` by `fidius init-plugin`, and read by `pack_package` for default filenames.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `PackageHeader` gains optional `extension: Option<String>` field (defaults to `"fid"` when absent)
- [ ] `pack_package` uses the manifest's extension for default output filename
- [ ] `fidius init-interface --extension <ext>` writes a `fidius.toml` in the interface crate with the extension
- [ ] `fidius init-plugin` reads the interface crate's `fidius.toml` and writes the extension into the generated `package.toml`
- [ ] Existing packages without `extension` field continue to work (default `.fid`)
- [ ] Full pipeline test updated to exercise custom extension
- [ ] Unit tests for extension defaulting and custom extension

## Implementation Notes

### Flow
1. `fidius init-interface my-api --trait Processor --extension cloacina` → writes `my-api/fidius.toml` with `extension = "cloacina"`
2. `fidius init-plugin my-plugin --interface ./my-api --trait Processor` → reads `my-api/fidius.toml`, writes `extension = "cloacina"` into `package.toml`
3. `fidius package pack ./my-plugin/` → reads `extension` from manifest → outputs `my-plugin-1.0.0.cloacina`

### Files to modify
- `fidius-core/src/package.rs` — add `extension` to `PackageHeader`, update `pack_package` default naming
- `fidius-cli/src/commands.rs` — update `init_interface` (write `fidius.toml`), `init_plugin` (read it), `package_pack` (no change needed, pack_package handles it)
- `fidius-cli/src/main.rs` — add `--extension` arg to `InitInterface`
- `fidius-cli/tests/full_pipeline.rs` — test custom extension

## Status Updates

- 2026-03-31: Implemented. `PackageHeader` has optional `extension` field with `extension()` helper defaulting to "fid". `pack_package` uses it for default naming. `init-interface --extension` writes `fidius.toml`. `init-plugin` reads it and writes extension into generated `package.toml` (also now auto-generates `package.toml`). Full pipeline test exercises `.testpkg` custom extension. 2 new unit tests for custom ext + defaulting. All 92+ tests pass.