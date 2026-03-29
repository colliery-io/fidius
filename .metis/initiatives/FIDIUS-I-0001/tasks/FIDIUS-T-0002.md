---
id: implement-repr-c-descriptor-and
level: task
title: "Implement repr(C) descriptor and registry types"
short_code: "FIDIUS-T-0002"
created_at: 2026-03-29T00:33:50.078790+00:00
updated_at: 2026-03-29T00:42:33.650142+00:00
parent: FIDIUS-I-0001
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0001
---

# Implement repr(C) descriptor and registry types

## Parent Initiative

[[FIDIUS-I-0001]]

## Objective

Implement the `#[repr(C)]` types that form the stable ABI contract: `PluginRegistry`, `PluginDescriptor`, `BufferStrategyKind`, and the magic bytes constant. These types are read directly from dylib memory by the host, so layout correctness is critical.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `PluginRegistry` struct with fields: `magic: [u8; 8]`, `registry_version: u32`, `plugin_count: u32`, `descriptors: *const *const PluginDescriptor` — all `#[repr(C)]`
- [ ] `PluginDescriptor` struct with all fields from spec: `abi_version`, `interface_name`, `interface_hash`, `interface_version`, `capabilities`, `wire_format`, `buffer_strategy`, `plugin_name`, `vtable`, `free_buffer`
- [ ] `BufferStrategyKind` enum: `CallerAllocated = 0`, `PluginAllocated = 1`, `Arena = 2` — `#[repr(u8)]`
- [ ] `FIDIUS_MAGIC: [u8; 8] = *b"FIDIUS\0\0"` constant
- [ ] `REGISTRY_VERSION: u32 = 1` and `ABI_VERSION: u32 = 1` constants
- [ ] `WireFormat` enum: `Json = 0`, `Bincode = 1` — `#[repr(u8)]`
- [ ] All pointer fields documented with safety invariants
- [ ] Types are `Send + Sync` where appropriate (registry/descriptor are read-only after construction)

## Implementation Notes

### Technical Approach

File: `fidius-core/src/descriptor.rs`

Key considerations:
- Use `*const c_char` for string fields (not `CStr` — must be repr(C) compatible)
- `vtable` is `*const c_void` — opaque to fidius-core, typed by the macro
- `free_buffer` is `Option<unsafe extern "C" fn(*mut u8, usize)>` — nullable function pointer is repr(C) safe
- Add `unsafe impl Send for PluginDescriptor {}` and `Sync` with documented safety rationale (all fields are either primitives or static pointers)
- Consider adding a `PluginDescriptor::interface_name_str()` unsafe helper that converts `*const c_char` to `&str`

### Dependencies
- FIDIUS-T-0001 (workspace must exist)

## Status Updates

- **2026-03-29**: Implemented in `fidius-core/src/descriptor.rs`. All types repr(C), Send+Sync with safety docs. Includes helper methods: `interface_name_str()`, `plugin_name_str()`, `buffer_strategy_kind()`, `wire_format_kind()`, `has_capability()`. Compiles clean.