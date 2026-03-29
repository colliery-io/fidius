---
id: initialize-cargo-workspace-and
level: task
title: "Initialize Cargo workspace and crate stubs"
short_code: "FIDIUS-T-0001"
created_at: 2026-03-29T00:33:48.621582+00:00
updated_at: 2026-03-29T00:41:33.891215+00:00
parent: FIDIUS-I-0001
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0001
---

# Initialize Cargo workspace and crate stubs

## Parent Initiative

[[FIDIUS-I-0001]]

## Objective

Set up the Cargo workspace at the repo root with all five crates as members. Each crate gets a valid Cargo.toml with correct inter-crate dependencies, a minimal `src/lib.rs` (or `src/main.rs` for fidius-cli), and the workspace compiles clean with `cargo check`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Root `Cargo.toml` defines a workspace with members: `fidius-core`, `fidius-macro`, `fidius-host`, `fidius-cli`, `fidius`
- [ ] `fidius-core/Cargo.toml` exists with `serde`, `serde_json`, `bincode` dependencies
- [ ] `fidius-macro/Cargo.toml` exists as `proc-macro = true` with `syn`, `quote`, `proc-macro2` dependencies; depends on `fidius-core`
- [ ] `fidius-host/Cargo.toml` depends on `fidius-core`, `libloading`, `ed25519-dalek`
- [ ] `fidius-cli/Cargo.toml` depends on `fidius-host`, `fidius-core`, `clap`, `ed25519-dalek`; has `[[bin]]` target
- [ ] `fidius/Cargo.toml` depends on `fidius-core` and `fidius-macro` as public dependencies
- [ ] `cargo check --workspace` succeeds
- [ ] Git initialized with `.gitignore` for `/target` and `Cargo.lock`
- [ ] `.angreal/` directory with angreal task files for dev operations: `test`, `build`, `check`, `lint`

## Implementation Notes

### Technical Approach
- Use `cargo init` or manual creation for each crate
- Stub crates with empty `lib.rs` / `main.rs` files
- Use workspace-level dependency declarations where possible (`[workspace.dependencies]`)
- fidius-macro must be `proc-macro = true` in its Cargo.toml `[lib]` section
- Set up angreal as the dev harness with tasks for:
  - `angreal test` — run `cargo test --workspace` (and `cargo test --workspace --release` for bincode path)
  - `angreal build` — `cargo build --workspace`
  - `angreal check` — `cargo check --workspace` + `cargo clippy --workspace`
  - `angreal lint` — `cargo fmt --check` + clippy

### Dependencies
- None — this is the first task

## Status Updates

- **2026-03-29**: Workspace created. All 5 crates compile clean. Git initialized. Angreal tasks for test/build/check/lint created. Workspace dependencies centralized in root Cargo.toml.