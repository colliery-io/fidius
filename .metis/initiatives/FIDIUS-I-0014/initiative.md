---
id: arena-buffer-strategy-implement-or
level: initiative
title: "Arena Buffer Strategy — Implement or Remove Unused Strategies"
short_code: "FIDIUS-I-0014"
created_at: 2026-04-17T13:23:32.689827+00:00
updated_at: 2026-04-18T01:32:59.209431+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
initiative_id: arena-buffer-strategy-implement-or
---

# Arena Buffer Strategy — Implement or Remove Unused Strategies Initiative

## Context

`BufferStrategyKind` in `fidius-core/src/descriptor.rs:39-48` declares three variants: `CallerAllocated`, `PluginAllocated`, `Arena`. The spec (`FIDIUS-S-0001`), CLI inspect output, descriptor serialization, and compile-fail test (`fidius-macro/tests/compile_fail/unsupported_buffer.rs`) all treat this as a first-class three-variant choice.

**In reality, only `PluginAllocated` is implemented.** `fidius-macro/src/interface.rs:42-54` hard-rejects the other two:

```rust
BufferStrategyAttr::CallerAllocated => {
    return Err(syn::Error::new_spanned(
        &ir.original_trait.ident,
        "CallerAllocated buffer strategy is not yet supported",
    ));
}
BufferStrategyAttr::Arena => {
    return Err(syn::Error::new_spanned(
        &ir.original_trait.ident,
        "Arena buffer strategy is not yet supported",
    ));
}
```

Consequences:
- The `buffer_strategy` descriptor field is always `1` — pure header bloat
- `fidius inspect` reports a value the user can't actually choose
- The host's `expected_strategy` validation is a real check against a single possible value
- Readers of the spec/ABI assume all three work, then hit the macro error

**The review identified this as the highest-impact perf opportunity** — `Arena` enables zero per-call heap allocation on the plugin side. For hot loops (e.g., image filters processing frames), this is the difference between "plugin framework" and "plugin framework you'd actually use on a hot path." `CallerAllocated` adds less value in practice (plugin can still alloc internally; the only difference is who owns the output buffer) and carries a retry-on-too-small codepath that doubles host complexity.

## Decision (Settled)

**Option A: Ship Arena, drop CallerAllocated.** Implement Arena end-to-end; remove `CallerAllocated` from `BufferStrategyKind` and the ABI. Two variants only going forward: `PluginAllocated` (status quo) and `Arena` (new, zero-alloc hot path).

## Goals & Non-Goals

**Goals:**
- Plugin authors can specify `#[plugin_interface(buffer = Arena)]` and it compiles
- Host provides an `ArenaPool` with thread-local or per-call arena buffers
- Plugin-side shim writes serialized output into the arena (not heap)
- One FFI call path for a hot plugin method produces zero heap allocations on the plugin side (verified via allocator counter test)
- Retry-on-too-small works: if the arena is too small, plugin returns `STATUS_BUFFER_TOO_SMALL` with required size; host grows arena and retries
- `CallerAllocated` is removed from `BufferStrategyKind` and the ABI

**Non-Goals:**
- Mix-and-match strategies within a single interface
- Cross-plugin arena sharing (each plugin has its own arena)
- Arena allocator for the plugin's own internal state (just for output)

## Detailed Design (Option A)

### Arena vtable signature

Arena strategy changes the vtable function signature — instead of allocating output, the plugin writes into a caller-supplied buffer:

```rust
unsafe extern "C" fn(
    in_ptr: *const u8, in_len: u32,
    arena_ptr: *mut u8, arena_cap: u32,
    out_offset: *mut u32,
    out_len: *mut u32,
) -> i32
```

- Input pointer+len: same as PluginAllocated
- `arena_ptr`, `arena_cap`: host-provided arena buffer and its capacity
- `out_offset`: plugin writes the offset into the arena where output starts
- `out_len`: plugin writes the output length
- Return: `STATUS_OK`, `STATUS_BUFFER_TOO_SMALL` (with needed size in `out_len`), or other status codes

### Plugin-side shim codegen (Arena variant)

```rust
unsafe extern "C" fn __fidius_shim_Foo_bar(
    in_ptr: *const u8, in_len: u32,
    arena_ptr: *mut u8, arena_cap: u32,
    out_offset: *mut u32,
    out_len: *mut u32,
) -> i32 {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let in_slice = std::slice::from_raw_parts(in_ptr, in_len as usize);
        let (arg1, arg2) = match wire::deserialize(in_slice) { ... };
        let output = instance.bar(arg1, arg2);

        // Serialize into a thread-local buffer to know required size
        let bytes = match wire::serialize(&output) { ... };
        if bytes.len() > arena_cap as usize {
            unsafe { *out_len = bytes.len() as u32; }
            return STATUS_BUFFER_TOO_SMALL;
        }

        // Copy into arena
        let arena_slice = std::slice::from_raw_parts_mut(arena_ptr, arena_cap as usize);
        arena_slice[..bytes.len()].copy_from_slice(&bytes);
        unsafe {
            *out_offset = 0;
            *out_len = bytes.len() as u32;
        }
        STATUS_OK
    }));
    // ... panic handling
}
```

Note: this still allocates a `Vec<u8>` for serialization. True zero-alloc requires `serde` writers that write directly into the arena — can be done with `bincode::serialize_into(&mut &mut arena[..], &output)`. Do this in a follow-up task within the initiative.

### Host-side: ArenaPool

```rust
// fidius-host/src/arena.rs
pub struct ArenaPool {
    buffers: Mutex<Vec<Vec<u8>>>,
    initial_capacity: usize,
}

impl ArenaPool {
    pub fn new(initial_capacity: usize) -> Self { ... }
    pub fn with_arena<R>(&self, f: impl FnOnce(&mut [u8]) -> R) -> R { ... }
}
```

`PluginHandle::call_method` for Arena strategy:
1. Acquire an arena buffer from the pool (or create one at initial_capacity)
2. Call vtable fn with arena ptr + cap
3. On `STATUS_BUFFER_TOO_SMALL`, grow the buffer to needed size and retry
4. On `STATUS_OK`, deserialize from `arena[offset..offset+len]`
5. Return buffer to pool

### Files to modify (Option A)

- `fidius-core/src/descriptor.rs` — remove `CallerAllocated` variant, bump `ABI_VERSION` to 3
- `fidius-macro/src/ir.rs` — update `BufferStrategyAttr` enum, parsing
- `fidius-macro/src/interface.rs` — generate Arena vtable signature, remove reject for Arena, keep reject for (removed) CallerAllocated or remove that branch
- `fidius-macro/src/impl_macro.rs` — branch on strategy in shim codegen (Arena vs PluginAllocated)
- `fidius-host/src/arena.rs` — new module
- `fidius-host/src/handle.rs` — Arena call path with retry
- `fidius-host/src/loader.rs` — Arena descriptor validation
- `fidius-host/src/host.rs` — `ArenaPool` wiring into builder
- `fidius-macro/tests/compile_fail/unsupported_buffer.rs` — remove (or repurpose to test a different invalid value)
- `fidius-macro/tests/` — new tests: arena compile, arena smoke via vtable, retry-on-too-small
- `fidius-host/tests/integration.rs` — end-to-end Arena test with a hot-loop benchmark
- `tests/test-plugin-smoke/src/lib.rs` — could add an Arena-strategy companion to exercise the path
- Scaffold updates in `fidius-cli`
- Spec document (`FIDIUS-S-0001`) — the buffer strategy section

## Alternatives Considered

- **Option B (drop both):** Cleaner but forecloses a real perf path. The review explicitly identified Arena as the biggest perf lever — dropping it because it's currently unused leaves value on the table.
- **Option C (ship Arena, keep CallerAllocated as placeholder):** Keeps the very problem this initiative was created to solve. No.
- **Implement all three:** Doubles test matrix and codegen paths. CallerAllocated's value proposition (caller owns the buffer) is weak when the plugin can already use `PluginAllocated` + a pool on its side.
- **Make it an allocator abstraction (e.g., pass an `Allocator` trait object):** Over-engineered; C ABI has no way to express this cleanly.

## Implementation Plan (Option A — pending user decision)

1. Bump `ABI_VERSION` to 3; remove `CallerAllocated` from `BufferStrategyKind`
2. Update `BufferStrategyAttr` in ir.rs to match
3. Update `generate_interface` in interface.rs — emit Arena vtable signature when `buffer = Arena`
4. Update `generate_shims` in impl_macro.rs — emit Arena shim when strategy is Arena
5. Implement `fidius-host/src/arena.rs` with `ArenaPool`
6. Update `PluginHandle::call_method` to dispatch on strategy (PluginAllocated vs Arena)
7. Implement retry-on-too-small in the Arena call path
8. Add compile tests for Arena interfaces
9. Add smoke test: Arena plugin loads and round-trips
10. Add perf test: zero plugin-side heap alloc verified via custom `#[global_allocator]`
11. Add Arena example to test-plugin-smoke or a new fixture
12. Update CLI scaffolds with `--buffer arena` option
13. Update spec doc

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- `#[plugin_interface(buffer = Arena)]` compiles
- A smoke plugin with Arena strategy loads and round-trips an `add(a, b)` call
- Retry-on-too-small works — start with a 16-byte arena, call a method whose output is 100 bytes, verify exactly one retry happens and the call succeeds
- `cargo test` with an alloc-counting global allocator shows zero plugin-side allocs during the hot loop (excluding serialization helper that can be optimized in a follow-up)
- `CallerAllocated` is not present anywhere in the codebase
- ABI_VERSION == 3; old plugins are rejected with `IncompatibleAbiVersion`
- Spec document updated to reflect two strategies only