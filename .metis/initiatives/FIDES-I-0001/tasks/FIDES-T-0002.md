---
id: implement-repr-c-descriptor-and
level: task
title: "Implement repr(C) descriptor and registry types"
short_code: "FIDES-T-0002"
created_at: 2026-03-29T00:33:50.078790+00:00
updated_at: 2026-03-29T00:33:50.078790+00:00
parent: FIDES-I-0001
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: FIDES-I-0001
---

# Implement repr(C) descriptor and registry types

## Parent Initiative

[[FIDES-I-0001]]

## Objective

Implement the `#[repr(C)]` types that form the stable ABI contract: `PluginRegistry`, `PluginDescriptor`, `BufferStrategyKind`, and the magic bytes constant. These types are read directly from dylib memory by the host, so layout correctness is critical.

## Acceptance Criteria

- [ ] `PluginRegistry` struct with fields: `magic: [u8; 8]`, `registry_version: u32`, `plugin_count: u32`, `descriptors: *const *const PluginDescriptor` — all `#[repr(C)]`
- [ ] `PluginDescriptor` struct with all fields from spec: `abi_version`, `interface_name`, `interface_hash`, `interface_version`, `capabilities`, `wire_format`, `buffer_strategy`, `plugin_name`, `vtable`, `free_buffer`
- [ ] `BufferStrategyKind` enum: `CallerAllocated = 0`, `PluginAllocated = 1`, `Arena = 2` — `#[repr(u8)]`
- [ ] `FIDES_MAGIC: [u8; 8] = *b"FIDES\0\0\0"` constant
- [ ] `REGISTRY_VERSION: u32 = 1` and `ABI_VERSION: u32 = 1` constants
- [ ] `WireFormat` enum: `Json = 0`, `Bincode = 1` — `#[repr(u8)]`
- [ ] All pointer fields documented with safety invariants
- [ ] Types are `Send + Sync` where appropriate (registry/descriptor are read-only after construction)

## Implementation Notes

### Technical Approach

File: `fides-core/src/descriptor.rs`

Key considerations:
- Use `*const c_char` for string fields (not `CStr` — must be repr(C) compatible)
- `vtable` is `*const c_void` — opaque to fides-core, typed by the macro
- `free_buffer` is `Option<unsafe extern "C" fn(*mut u8, usize)>` — nullable function pointer is repr(C) safe
- Add `unsafe impl Send for PluginDescriptor {}` and `Sync` with documented safety rationale (all fields are either primitives or static pointers)
- Consider adding a `PluginDescriptor::interface_name_str()` unsafe helper that converts `*const c_char` to `&str`

### Dependencies
- FIDES-T-0001 (workspace must exist)

## Status Updates

*To be added during implementation*