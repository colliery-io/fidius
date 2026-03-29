---
id: update-spec-document-for-registry
level: task
title: "Update spec document for registry function and fidius rename"
short_code: "FIDIUS-T-0026"
created_at: 2026-03-29T12:20:39.576106+00:00
updated_at: 2026-03-29T12:41:17.677084+00:00
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

# Update spec document for registry function and fidius rename

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective

The ABI specification (FIDIUS-S-0001) still references the original design patterns that changed during implementation. Update it to match the actual implementation: `fidius_get_registry()` function instead of static `FIDIUS_PLUGIN_REGISTRY` symbol, `inventory`-based multi-plugin assembly, `fidius_plugin_registry!()` macro, and the `fidius` naming throughout.

## Technical Debt Impact

- **Current Problems**: Spec says `dlsym("FIDIUS_PLUGIN_REGISTRY")` but the actual export is `fidius_get_registry()`. New contributors reading the spec will be confused.
- **Benefits of Fixing**: Spec matches implementation — single source of truth.
- **Risk Assessment**: None — documentation only.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Load sequence updated: `dlsym("fidius_get_registry")` → call → get `*const PluginRegistry`
- [ ] Multi-plugin section updated: `inventory` crate + `fidius_plugin_registry!()` macro
- [ ] All code examples use `fidius` naming
- [ ] Developer workflow section references `fidius` CLI commands
- [ ] `PluginDescriptor.details` field documented as `Option<String>` (not `Option<Value>`)

## Status Updates

- **2026-03-29**: Updated spec. Registry: static symbol → `fidius_get_registry()` function + inventory + `fidius_plugin_registry!()` macro. Load sequence updated. Plugin example includes registry macro call. PluginError.details documented as `Option<String>`. All `fides` → `fidius` refs fixed.