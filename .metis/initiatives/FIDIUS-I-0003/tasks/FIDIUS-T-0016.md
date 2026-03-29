---
id: pluginhandle-type-safe-ffi-calling
level: task
title: "PluginHandle — type-safe FFI calling proxy"
short_code: "FIDIUS-T-0016"
created_at: 2026-03-29T01:28:33.183474+00:00
updated_at: 2026-03-29T11:23:06.216124+00:00
parent: FIDIUS-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0003
---

# PluginHandle — type-safe FFI calling proxy

## Parent Initiative

[[FIDIUS-I-0003]]

## Objective

Implement `PluginHandle` — the struct that makes calling a plugin feel like calling a Rust method. It holds `Arc<Library>` (keeps the dylib alive), the vtable pointer, the descriptor's `free_buffer` fn, capabilities bitfield, and owned metadata. Provides a generic `call_method()` that handles serialize → FFI call → status check → deserialize → free_buffer.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `PluginHandle` struct with: `Arc<Library>`, vtable pointer (`*const c_void`), `free_buffer` fn, capabilities, `PluginInfo`
- [ ] `call_method<I: Serialize, O: DeserializeOwned>(index: usize, input: &I) -> Result<O, CallError>` — generic method call through vtable
- [ ] Handles PluginAllocated buffer lifecycle: reads output, calls `free_buffer` after deserializing
- [ ] `CallError` enum: `Serialization`, `Deserialization`, `PluginError(PluginError)`, `Panic`, `BufferTooSmall`
- [ ] `has_capability(bit: u32) -> bool` — check optional method support
- [ ] `info() -> &PluginInfo` — access owned metadata
- [ ] `PluginHandle` is `Send + Sync` (Arc<Library> is Send+Sync, vtable pointer is read-only)
- [ ] Dropping `PluginHandle` drops the `Arc<Library>` ref — library unloads when last handle drops

## Implementation Notes

### Technical Approach

File: `fidius-host/src/handle.rs`

The vtable is an opaque `*const c_void`. To call method N, we treat it as a pointer to an array of function pointers and index into it:
```rust
let fn_ptrs = vtable as *const unsafe extern "C" fn(*const u8, u32, *mut *mut u8, *mut u32) -> i32;
let method_fn = *fn_ptrs.add(index);
```

This works because the VTable is `#[repr(C)]` and all function pointers have the same signature (PluginAllocated pattern).

For optional methods (index >= required_count), the fn pointer is actually an `Option<fn>` — need to check the capability bit before calling.

### Dependencies
- FIDIUS-T-0013 (types), FIDIUS-T-0014 (loader provides Arc<Library>)

## Status Updates

- **2026-03-29**: Implemented in `fidius-host/src/handle.rs`. `call_method<I,O>(index, input)` handles full lifecycle: serialize → FFI call → status check → deserialize → free_buffer. Handles all status codes including STATUS_PLUGIN_ERROR (deserializes PluginError from output). `has_capability()`, `info()`, `from_loaded()`. Send+Sync. Compiles clean.