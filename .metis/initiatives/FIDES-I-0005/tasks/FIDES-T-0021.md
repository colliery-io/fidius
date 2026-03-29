---
id: cli-scaffold-clap-setup-init
level: task
title: "CLI scaffold — clap setup, init-interface, init-plugin"
short_code: "FIDES-T-0021"
created_at: 2026-03-29T11:35:17.590017+00:00
updated_at: 2026-03-29T11:49:56.462133+00:00
parent: FIDES-I-0005
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDES-I-0005
---

# CLI scaffold — clap setup, init-interface, init-plugin

## Parent Initiative

[[FIDES-I-0005]]

## Objective

Set up the `fides` CLI binary with clap derive, implement the `init-interface` and `init-plugin` scaffolding subcommands. These generate the correct crate topology so plugin developers don't have to manually create Cargo.toml and lib.rs files.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `fides --help` prints usage with subcommands
- [ ] `fides init-interface <name> --trait <TraitName>` creates `<name>/Cargo.toml` (depends on `fides`) and `<name>/src/lib.rs` (re-exports + trait stub with `#[fides::plugin_interface]`)
- [ ] `fides init-plugin <name> --interface <value>` — smart dependency resolution from a single `--interface` arg:
  - If `<value>` is a filesystem path (exists on disk) → use `{ path = "<value>" }`
  - Else check crates.io for `<value>` → if found, use latest version `{ version = "<latest>" }`
  - Else warn: `"could not find '<value>' as a local path or on crates.io, using path dep"` and use `{ path = "<value>" }`
- [ ] `init-plugin` supports optional `--version <ver>` to pin a specific crates.io version instead of latest
- [ ] Same smart resolution for `init-interface`'s fides dependency: check if local fides path exists, else use crates.io version
- [ ] Crates.io check via `https://crates.io/api/v1/crates/<name>` (simple HTTP GET, no auth needed)
- [ ] Both commands accept `--path` to control output directory (default: current dir)
- [ ] Generated files are valid Rust that compiles (modulo the interface crate not existing yet for the plugin)
- [ ] Errors if target directory already exists
- [ ] `cargo build -p fides-cli` succeeds

## Implementation Notes

### Technical Approach

File: `fides-cli/src/main.rs` with clap derive.

Subcommand enum:
```rust
#[derive(Subcommand)]
enum Commands {
    InitInterface { name: String, #[arg(long)] trait_name: String, #[arg(long)] path: Option<PathBuf> },
    InitPlugin { name: String, #[arg(long)] interface: String, #[arg(long)] path: Option<PathBuf> },
    Keygen { ... },
    Sign { ... },
    Verify { ... },
    Inspect { ... },
}
```

Scaffolding writes files using `std::fs::write` with template strings.

### Dependencies
- None — this is the first CLI task

## Status Updates

- **2026-03-29**: Full CLI implemented in `fides-cli/src/main.rs` + `commands.rs`. All 6 subcommands: init-interface, init-plugin, keygen, sign, verify, inspect. Smart dep resolution via crates.io check (ureq). init-interface tested — generates valid Cargo.toml + lib.rs. inspect tested on test-plugin-smoke — shows registry metadata correctly. Note: `fides` name is taken on crates.io (unrelated crate) — need to pick alternate publish name.