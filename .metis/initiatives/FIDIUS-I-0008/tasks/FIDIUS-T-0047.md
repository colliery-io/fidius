---
id: r-08-fix-cli-scaffolding
level: task
title: "R-08: Fix CLI scaffolding dependencies"
short_code: "FIDIUS-T-0047"
created_at: 2026-03-29T17:19:43.116393+00:00
updated_at: 2026-03-29T17:29:22.020480+00:00
parent: FIDIUS-I-0008
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0008
---

# R-08: Fix CLI scaffolding dependencies

**Addresses**: COR-10, COR-11, LEG-10, EVO-05, API-11 | **Effort**: 2-3 hours

## Objective

Fix the CLI scaffolding commands (`init_interface`, `init_plugin`) so that generated projects have correct dependency declarations and metadata, ensuring new users can build scaffolded projects without manual Cargo.toml edits.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `init_interface` resolves `fidius-core` independently via `resolve_dep("fidius-core", version)` instead of inheriting it from `fidius`
- [ ] For local/path resolution, `fidius-core` path is derived relative to workspace root
- [ ] `init_plugin` uses `resolve_dep("fidius-core", version)` instead of hardcoded `"0.1"`
- [ ] User-Agent URL references `colliery-io/fidius` (not a placeholder or wrong URL)
- [ ] `env!("CARGO_PKG_VERSION")` is used as version fallback
- [ ] Scaffolded interface project compiles with `cargo check`
- [ ] Scaffolded plugin project compiles with `cargo check`

## Implementation Notes

1. In `fidius-cli/src/commands.rs`, `init_interface` function: add a separate `resolve_dep("fidius-core", version)` call and use the result in the generated Cargo.toml.
2. In `init_plugin` function: replace the hardcoded `fidius-core = { version = "0.1" }` line with output from `resolve_dep("fidius-core", version)`.
3. Fix the User-Agent string to use the correct repository URL (LEG-14).
4. Consider using `env!("CARGO_PKG_VERSION")` as fallback when version argument is not provided.

### Dependencies

- None.

### Files

- `fidius-cli/src/commands.rs` -- all changes in this file

## Status Updates

*To be added during implementation*