---
id: box-lt-u8-gt-ffi-buffer-contract
level: initiative
title: "Box&lt;[u8]&gt; FFI Buffer Contract — Remove shrink_to_fit Hazard"
short_code: "FIDIUS-I-0013"
created_at: 2026-04-17T13:23:31.169733+00:00
updated_at: 2026-04-17T17:44:42.014869+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: S
initiative_id: box-lt-u8-gt-ffi-buffer-contract
---

# Box&lt;[u8]&gt; FFI Buffer Contract — Remove shrink_to_fit Hazard Initiative

## Context

Plugin-side shims today follow this pattern to hand output bytes to the host (`fidius-macro/src/impl_macro.rs:247-256`):

```rust
output_bytes.shrink_to_fit();
let len = output_bytes.len();
let ptr = output_bytes.as_ptr() as *mut u8;
std::mem::forget(output_bytes);
unsafe {
    *out_ptr = ptr;
    *out_len = len as u32;
}
```

The host's `free_buffer` (the generated `__fidius_free_buffer_{Plugin}` function) reconstructs a `Vec` to drop it:

```rust
drop(unsafe { Vec::from_raw_parts(ptr, len, len) });
```

This is **only sound because `shrink_to_fit` has made cap == len**. `Vec::from_raw_parts` with a mismatched capacity is immediate undefined behavior. The contract is implicit, scattered across two codegen sites, and one missing `shrink_to_fit` call is silent UB.

This came in with FIDIUS-T-0037 (R-01: free_buffer capacity mismatch) as a fix for a real bug. The fix works, but the underlying contract stays fragile.

`Box<[u8]>` (aka boxed slice) has cap == len **by construction** — the type cannot represent a slice with slack capacity. `Vec::into_boxed_slice()` does the shrink in one call. `Box::into_raw` gives `*mut [u8]` from which we read ptr + len. `Box::from_raw(slice::from_raw_parts_mut(ptr, len))` reconstructs it. No cap==len invariant to violate.

## Goals & Non-Goals

**Goals:**
- Eliminate the implicit cap==len invariant between shim codegen and `free_buffer` codegen
- Remove explicit `shrink_to_fit` from the shim (the `into_boxed_slice` conversion handles it)
- No change to the FFI signature `(*mut u8, usize)` for `free_buffer` or the vtable function signatures

**Non-Goals:**
- Change the descriptor layout or ABI version
- Pool or reuse output buffers (that's a separate perf initiative)
- Change input buffer handling (input is host-owned, already correct)

## Detailed Design

### Shim output path (new)

```rust
let boxed: Box<[u8]> = output_bytes.into_boxed_slice();
let len = boxed.len();
let ptr = Box::into_raw(boxed) as *mut u8;
unsafe {
    *out_ptr = ptr;
    *out_len = len as u32;
}
```

`Box::into_raw` returns `*mut [u8]` (a fat pointer). Cast to `*mut u8` keeps the data pointer; `len()` is captured before the cast.

### free_buffer (new)

```rust
unsafe extern "C" fn __fidius_free_buffer_Plugin(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            let slice = std::slice::from_raw_parts_mut(ptr, len);
            drop(Box::from_raw(slice as *mut [u8]));
        }
    }
}
```

No capacity field, no mismatch window.

### Apply to both success and panic paths

The shim has two output paths in `impl_macro.rs:245-283`:
1. Success path: serialized output bytes
2. Panic path: serialized panic message string

Both use the same `shrink_to_fit + forget` pattern. Both must migrate.

## Alternatives Considered

- **Keep `Vec<u8>` + assert cap == len in `free_buffer`.** Makes the contract explicit but still relies on every future shim codegen change remembering it. `Box<[u8]>` is self-enforcing.
- **Pass capacity through the FFI.** Requires descriptor change (breaking), adds a u32 per call. The pointer+length surface is sufficient; capacity is an allocator concern that should stay on the plugin side.
- **Use a custom allocator handle.** Overkill for this problem.

## Implementation Plan

1. Update success-path codegen in `generate_shims` to emit `into_boxed_slice` + `Box::into_raw`
2. Update panic-path codegen (same function) the same way
3. Update `generate_plugin_impl` free_buffer codegen to use `Box::from_raw(slice::from_raw_parts_mut(...))`
4. Grep the codebase for any remaining `Vec::from_raw_parts` or `shrink_to_fit` near the FFI boundary and confirm they're gone
5. Add a regression test: a shim that deliberately returns output from a non-shrunk Vec path (e.g., a `Vec::with_capacity(1024)` with 10 bytes written) and verify no UB under Miri
6. Run the full test suite under `cargo miri test` for the macro test suite

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- `rg 'shrink_to_fit' fidius-macro/src/` returns no matches in codegen (test code excluded)
- `rg 'Vec::from_raw_parts' fidius-macro/src/ fidius-host/src/` returns no matches
- All existing tests pass (layout, smoke, integration, e2e, multi-arg, multi-plugin, async, crate_path)
- Miri-tested codegen passes without UB flags
- Generated `free_buffer` uses `Box::from_raw` pattern