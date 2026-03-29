---
id: cli-package-subcommands-validate
level: task
title: "CLI package subcommands — validate, build, inspect"
short_code: "FIDIUS-T-0031"
created_at: 2026-03-29T14:00:04.740807+00:00
updated_at: 2026-03-29T14:39:12.417524+00:00
parent: FIDIUS-I-0006
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0006
---

# CLI package subcommands — validate, build, inspect

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0006]]

## Objective

Implement `fidius package validate`, `fidius package build`, and `fidius package inspect` CLI subcommands. These are the consumer-side commands for working with source packages.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `fidius package validate <dir>` — reads `package.toml`, parses fixed header, prints summary (name, version, interface, deps). CLI validates header structure only (full schema validation requires host-side `M` type)
- [ ] `fidius package build <dir>` — runs `cargo build` inside the package dir, reports success/failure. `--release` flag (default) for release builds
- [ ] `fidius package inspect <dir>` — prints all manifest fields: name, version, interface, interface_version, dependencies, and raw metadata as TOML
- [ ] Missing `package.toml` → clear error
- [ ] Invalid TOML → clear parse error with line/column
- [ ] Missing `Cargo.toml` in package dir → `build` fails with clear error
- [ ] `fidius package` subcommand group shows help with all package commands

## Implementation Notes

### Technical Approach

Add a `Package` subcommand group to the existing clap CLI in `fidius-cli/src/main.rs`:

```rust
#[derive(Subcommand)]
enum PackageCommands {
    Validate { dir: PathBuf },
    Build { dir: PathBuf, #[arg(long, default_value = "true")] release: bool },
    Inspect { dir: PathBuf },
}
```

`validate` calls `fidius_core::package::load_manifest::<serde_json::Value>(dir)` — using `Value` as the metadata type accepts any metadata section.

`build` shells out to `cargo build --manifest-path <dir>/Cargo.toml [--release]`.

`inspect` loads the manifest and pretty-prints all fields.

### Dependencies
- FIDIUS-T-0030 (PackageManifest types must exist)

## Status Updates

- **2026-03-29**: Implemented all 5 package subcommands (validate, build, inspect, sign, verify) in one pass since signing reuses existing code. Package subcommand group with clap. Tested on test-plugin-smoke with package.toml. All commands working.