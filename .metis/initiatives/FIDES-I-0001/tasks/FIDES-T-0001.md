---
id: initialize-cargo-workspace-and
level: task
title: "Initialize Cargo workspace and crate stubs"
short_code: "FIDES-T-0001"
created_at: 2026-03-29T00:33:48.621582+00:00
updated_at: 2026-03-29T00:40:00.725769+00:00
parent: FIDES-I-0001
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: FIDES-I-0001
---

# Initialize Cargo workspace and crate stubs

## Parent Initiative

[[FIDES-I-0001]]

## Objective

Set up the Cargo workspace at the repo root with all five crates as members. Each crate gets a valid Cargo.toml with correct inter-crate dependencies, a minimal `src/lib.rs` (or `src/main.rs` for fides-cli), and the workspace compiles clean with `cargo check`.

## Acceptance Criteria

## Acceptance Criteria

- [ ] Root `Cargo.toml` defines a workspace with members: `fides-core`, `fides-macro`, `fides-host`, `fides-cli`, `fides`
- [ ] `fides-core/Cargo.toml` exists with `serde`, `serde_json`, `bincode` dependencies
- [ ] `fides-macro/Cargo.toml` exists as `proc-macro = true` with `syn`, `quote`, `proc-macro2` dependencies; depends on `fides-core`
- [ ] `fides-host/Cargo.toml` depends on `fides-core`, `libloading`, `ed25519-dalek`
- [ ] `fides-cli/Cargo.toml` depends on `fides-host`, `fides-core`, `clap`, `ed25519-dalek`; has `[[bin]]` target
- [ ] `fides/Cargo.toml` depends on `fides-core` and `fides-macro` as public dependencies
- [ ] `cargo check --workspace` succeeds
- [ ] Git initialized with `.gitignore` for `/target` and `Cargo.lock`
- [ ] `.angreal/` directory with angreal task files for dev operations: `test`, `build`, `check`, `lint`

## Implementation Notes

### Technical Approach
- Use `cargo init` or manual creation for each crate
- Stub crates with empty `lib.rs` / `main.rs` files
- Use workspace-level dependency declarations where possible (`[workspace.dependencies]`)
- fides-macro must be `proc-macro = true` in its Cargo.toml `[lib]` section
- Set up angreal as the dev harness with tasks for:
  - `angreal test` — run `cargo test --workspace` (and `cargo test --workspace --release` for bincode path)
  - `angreal build` — `cargo build --workspace`
  - `angreal check` — `cargo check --workspace` + `cargo clippy --workspace`
  - `angreal lint` — `cargo fmt --check` + clippy

### Dependencies
- None — this is the first task

## Status Updates

*To be added during implementation*