---
id: finalize-fides-facade-crate-re
level: task
title: "Finalize fides facade crate re-exports"
short_code: "FIDES-T-0019"
created_at: 2026-03-29T11:27:09.619205+00:00
updated_at: 2026-03-29T11:32:28.905176+00:00
parent: FIDES-I-0004
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDES-I-0004
---

# Finalize fides facade crate re-exports

## Parent Initiative

[[FIDES-I-0004]]

## Objective

Ensure the `fides` facade crate re-exports everything an interface crate author needs: macros (`plugin_interface`, `plugin_impl`), core types (`PluginError`, `fides_plugin_registry!`), and feature flags (`async`). An interface crate should need only `fides` as a dependency.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `fides/src/lib.rs` re-exports: `plugin_interface`, `plugin_impl` from fides-macro
- [ ] Re-exports `PluginError`, `fides_plugin_registry!`, descriptor types from fides-core
- [ ] `fides/Cargo.toml` has `async` feature that forwards to `fides-core/async`
- [ ] A test crate depending only on `fides` (not fides-core or fides-macro directly) can define a trait, implement it, and emit a registry
- [ ] `cargo check -p fides` succeeds

## Implementation Notes

### Technical Approach
The facade already exists with basic re-exports from I-0002. This task audits and completes the public API surface. The `fides_plugin_registry!` macro is already `#[macro_export]` in fides-core so it's available via `fides_core::fides_plugin_registry!()` — may want to re-export for ergonomics.

### Dependencies
- None — all upstream crates are complete

## Status Updates

- **2026-03-29**: Facade finalized. Re-exports: macros (plugin_interface, plugin_impl), descriptor types, PluginError, status codes, wire module, hash functions, inventory, registry. Added `async` feature forwarding to fides-core/async. Compiles clean.