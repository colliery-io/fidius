---
id: plugin-interface-macro-vtable-and
level: task
title: "plugin_interface macro — VTable and hash generation"
short_code: "FIDIUS-T-0007"
created_at: 2026-03-29T00:53:33.581971+00:00
updated_at: 2026-03-29T01:02:03.235040+00:00
parent: FIDIUS-I-0002
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0002
---

# plugin_interface macro — VTable and hash generation

## Parent Initiative

[[FIDIUS-I-0002]]

## Objective

Implement the `#[plugin_interface]` attribute macro. When applied to a trait, it emits the original trait plus: a `#[repr(C)]` vtable struct with one function pointer per method, an interface hash constant, capability bit constants for optional methods, and a descriptor-builder macro/function for use by `#[plugin_impl]`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `#[plugin_interface(version = 1, buffer = PluginAllocated)]` compiles on a trait
- [ ] Generates `{Trait}_VTable` struct with `#[repr(C)]` — required methods as `unsafe extern "C" fn(...)`, optional methods as `Option<unsafe extern "C" fn(...)>`
- [ ] Function pointer signatures follow the PluginAllocated pattern: `(in_ptr, in_len, out_ptr_ptr, out_len) -> i32`
- [ ] Generates `const {TRAIT}_INTERFACE_HASH: u64` from sorted required method signatures via `fidius_core::hash::fnv1a`
- [ ] Generates `const {TRAIT}_CAP_{METHOD}: u64` for each optional method (bit position)
- [ ] Generates `{TRAIT}_INTERFACE_VERSION: u32` from the `version` attribute
- [ ] Generates `{TRAIT}_BUFFER_STRATEGY: u8` from the `buffer` attribute
- [ ] Generates a `build_{trait}_descriptor` helper function/macro that `plugin_impl` calls to populate `PluginDescriptor`
- [ ] The original trait definition is preserved in the output (the macro is additive)
- [ ] CallerAllocated and Arena match arms exist but emit `compile_error!("not yet supported")`

## Implementation Notes

### Technical Approach

File: `fidius-macro/src/interface.rs`

Takes the `InterfaceIR` from T-0006, generates TokenStream:

1. Re-emit the original trait
2. Generate vtable struct — for each method, emit a function pointer field. For PluginAllocated: `unsafe extern "C" fn(*const u8, u32, *mut *mut u8, *mut u32) -> i32`. Optional methods wrap in `Option<>`.
3. Compute hash at macro expansion time using `fidius_core::hash::fnv1a` on the concatenated sorted signature strings (proc macros can call regular Rust functions)
4. Emit constants and the descriptor builder

### Dependencies
- FIDIUS-T-0006 (IR parsing)

## Status Updates

- **2026-03-29**: Implemented in `fidius-macro/src/interface.rs`. Generates: VTable struct (repr(C), PluginAllocated signatures), interface hash constant (computed at macro expansion time via fidius_core::hash), capability bit constants, version/strategy constants, const descriptor builder function, and cleaned trait (strips #[optional] attrs). 5 integration tests pass. CallerAllocated/Arena emit compile_error as designed.