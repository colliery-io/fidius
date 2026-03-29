---
id: plugin-impl-macro-ffi-shims-and
level: task
title: "plugin_impl macro — FFI shims and descriptor emission"
short_code: "FIDES-T-0008"
created_at: 2026-03-29T00:53:34.351153+00:00
updated_at: 2026-03-29T01:04:35.452052+00:00
parent: FIDES-I-0002
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDES-I-0002
---

# plugin_impl macro — FFI shims and descriptor emission

## Parent Initiative

[[FIDES-I-0002]]

## Objective

Implement the `#[plugin_impl(TraitName)]` attribute macro. When applied to an impl block, it emits the original impl plus: `extern "C"` shim functions (one per trait method) that handle serialization/deserialization/catch_unwind, a static instance of the impl struct, a populated vtable static, and a `PluginDescriptor` static. For single-plugin dylibs, also emit the `FIDES_PLUGIN_REGISTRY` (multi-plugin is T-0009).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `#[plugin_impl(TraitName)]` compiles on an impl block
- [ ] Generates one `extern "C"` shim per trait method with the PluginAllocated signature
- [ ] Each shim: deserializes input via `fides_core::wire::deserialize`, calls the real method on a static instance, serializes output via `fides_core::wire::serialize`, returns status code
- [ ] Each shim wrapped in `std::panic::catch_unwind` — panics return `STATUS_PANIC`
- [ ] Plugin-allocated output: shim allocates via `Vec<u8>`, leaks it, writes pointer+len to out params. `free_buffer` reclaims via `Vec::from_raw_parts`.
- [ ] Generates a `static {IMPL}_VTABLE: {Trait}_VTable` with all function pointers populated (optional methods that are implemented → `Some(fn_ptr)`, missing → `None`)
- [ ] Generates a `static {IMPL}_DESCRIPTOR: PluginDescriptor` with all fields populated from the interface constants
- [ ] Generates `static {IMPL}_INSTANCE: {ImplType}` (constructed via `Default` or zero-sized)
- [ ] For single-plugin case: emits `#[no_mangle] pub static FIDES_PLUGIN_REGISTRY: PluginRegistry` pointing to the descriptor
- [ ] The original impl block is preserved in the output
- [ ] `Result<T, PluginError>` return types: serialize `Ok(T)` on success, serialize `PluginError` and return `STATUS_PLUGIN_ERROR` on `Err`

## Implementation Notes

### Technical Approach

File: `fides-macro/src/impl_macro.rs`

Parse `impl TraitName for ImplType { ... }`. For each method in the impl block, generate a corresponding `extern "C"` shim function with a unique name (e.g., `__fides_{impl}_{method}`).

The shim pattern for PluginAllocated:
```rust
unsafe extern "C" fn __fides_blur_process(
    in_ptr: *const u8, in_len: u32,
    out_ptr: *mut *mut u8, out_len: *mut u32,
) -> i32 {
    std::panic::catch_unwind(|| {
        let input = fides_core::wire::deserialize(slice::from_raw_parts(in_ptr, in_len as usize))?;
        let result = INSTANCE.process(input);
        // handle Result<T, PluginError>
        let output_bytes = fides_core::wire::serialize(&result)?;
        let len = output_bytes.len();
        let ptr = output_bytes.as_ptr();
        std::mem::forget(output_bytes);
        *out_ptr = ptr as *mut u8;
        *out_len = len as u32;
        Ok(STATUS_OK)
    }).unwrap_or(STATUS_PANIC)
}
```

### Dependencies
- FIDES-T-0006 (IR), FIDES-T-0007 (vtable struct + constants exist)

## Status Updates

- **2026-03-29**: Implemented in `fides-macro/src/impl_macro.rs`. Generates: extern "C" shims with catch_unwind + wire serialize/deserialize, static instance, vtable static, descriptor static via builder fn, free_buffer fn, single-plugin FIDES_PLUGIN_REGISTRY. Added DescriptorPtr wrapper to fides-core for Sync safety. 3 integration tests pass including full FFI round-trip (serialize input → call vtable → deserialize output → verify "Hello, World!").