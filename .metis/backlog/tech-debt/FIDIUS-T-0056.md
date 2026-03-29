---
id: r-25-document-implicit-contracts
level: task
title: "R-25: Document implicit contracts and add compile-time checks"
short_code: "FIDIUS-T-0056"
created_at: 2026-03-29T18:02:37.990169+00:00
updated_at: 2026-03-29T18:07:08.809569+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# R-25: Document implicit contracts and add compile-time checks

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective

Make the framework's implicit contracts explicit through compile-time checks and documentation. Several critical invariants exist only as implicit knowledge, causing confusing errors for newcomers.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] **Unit struct check**: `#[plugin_impl]` emits a clear compile error if the impl type is not a unit struct
- [ ] **Companion module docs**: `#[plugin_interface]` doc comment mentions `__fidius_{Trait}` module and contents
- [ ] **Const-eval comment**: Comment in `impl_macro.rs` explaining manual byte comparison
- [ ] **Send + Sync docs**: `PluginHandle` doc states thread-safety requirements and safety rationale
- [ ] **Vtable ordering docs**: Companion module doc states method indices follow declaration order

## Implementation Notes

### Technical Approach

1. Unit struct: in `generate_plugin_impl`, emit `const _: () = { let _ = #impl_type; };` — fails with clearer error if type needs construction args
2. Doc comments: add `///` to generated companion module and PluginHandle's Send+Sync impls
3. Code comments: add `//` at the const-eval string comparison

### Dependencies
- None

## Status Updates

*To be added during implementation*