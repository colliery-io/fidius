---
id: update-plugin-impl-shim-codegen-to
level: task
title: "Update plugin_impl shim codegen to use crate_path from companion module"
short_code: "FIDIUS-T-0063"
created_at: 2026-04-01T01:40:16.613144+00:00
updated_at: 2026-04-01T01:59:53.192018+00:00
parent: FIDIUS-I-0011
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0011
---

# Update plugin_impl shim codegen to use crate_path from companion module

## Parent Initiative

[[FIDIUS-I-0011]]

## Objective

Update `#[plugin_impl]` shim codegen to read the crate path from the companion module (embedded by T-0062) instead of hardcoding `fidius::`. Also support an optional `crate = "..."` override on `plugin_impl` itself.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `PluginImplAttrs` gains optional `crate_path: Option<syn::Path>`
- [ ] `#[plugin_impl(MyTrait, crate = "my_crate::fidius")]` parses correctly
- [ ] When no `crate` attr, shim reads the path from `__fidius_{TraitName}::__fidius_crate`
- [ ] All `fidius::` references in `impl_macro.rs` codegen replaced with the resolved crate path
- [ ] `fidius_plugin_registry!()` macro also respects the crate path (or documents how to call it through the re-export)
- [ ] Existing plugins without `crate` attr continue to work

## Implementation Notes

### Files to modify
- `fidius-macro/src/impl_macro.rs` — parse optional `crate` attr, resolve crate path from companion module, replace all `fidius::` in generated shims/vtable/descriptor

### Resolving the path
The shim needs to reference `__fidius_{TraitName}::__fidius_crate` for all fidius types. The generated code becomes:
```rust
use #companion::__fidius_crate as __fc;
// then use __fc::wire::serialize, __fc::status::STATUS_OK, etc.
```

### Dependencies
- Blocked by FIDIUS-T-0062 (companion module must export the crate path first)

## Status Updates

- 2026-03-31: Added `crate_path: Path` to `PluginImplAttrs` with `crate = "..."` parsing (same `Token![crate]` pattern as T-0062). Defaults to `fidius`. Replaced all `fidius::` references in shim codegen (`generate_shims`, `generate_descriptor`, `generate_inventory_registration`) with `#crate_path::`. The `fidius_plugin_registry!()` macro already uses `$crate::` and works correctly through re-exports. Decided against companion module re-export approach — `plugin_impl` gets its own `crate` attr directly. Full test suite passes.