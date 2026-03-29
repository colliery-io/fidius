<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# Buffer Strategies Explained

How memory ownership works across the FFI boundary, why there are three
strategies, and why PluginAllocated is the default.

## The Core Problem

When a host calls a plugin method through the FFI boundary, the method
produces output bytes (serialized return value). Someone has to allocate
memory for those bytes. The question is: who allocates, who owns the
allocation, and who frees it? This is the buffer strategy.

## The Three Strategies

Fidius defines three buffer strategies in `fidius-core/src/descriptor.rs`:

```rust
#[repr(u8)]
pub enum BufferStrategyKind {
    CallerAllocated = 0,
    PluginAllocated = 1,
    Arena = 2,
}
```

The strategy is set **per-trait** in the `#[plugin_interface]` attribute:

```rust
#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait MyPlugin: Send + Sync { ... }
```

### CallerAllocated (strategy = 0)

The host pre-allocates an output buffer and passes it to the plugin. The
plugin writes its output into the provided buffer.

```
FFI signature:
  unsafe extern "C" fn(
      in_ptr: *const u8, in_len: u32,     // serialized input
      out_ptr: *mut u8, out_cap: u32,      // host-provided buffer
      out_len: *mut u32,                   // actual bytes written
  ) -> i32
```

| Aspect | Detail |
|--------|--------|
| **Who allocates** | Host (caller) |
| **Who writes** | Plugin |
| **Who frees** | Host (owns the buffer) |
| **On overflow** | Returns `STATUS_BUFFER_TOO_SMALL` (-1); `out_len` contains the required size |

**Trade-offs:**

- Zero allocation on the plugin side -- the buffer is already there.
- Host must guess an initial buffer size. If the guess is wrong, the call
  must be retried with a larger buffer, doubling latency.
- Works well when output sizes are predictable and bounded.

### PluginAllocated (strategy = 1)

The plugin allocates its own output buffer (via `Vec<u8>`) and hands the
pointer back to the host. The host reads the data, then calls the descriptor's
`free_buffer` function to deallocate.

```
FFI signature:
  unsafe extern "C" fn(
      in_ptr: *const u8, in_len: u32,     // serialized input
      out_ptr: *mut *mut u8,              // plugin sets this to allocated pointer
      out_len: *mut u32,                  // plugin sets this to length
  ) -> i32
```

| Aspect | Detail |
|--------|--------|
| **Who allocates** | Plugin (via `Vec<u8>`, then `mem::forget`) |
| **Who writes** | Plugin |
| **Who frees** | Host, via `free_buffer(ptr, len)` from the descriptor |
| **On overflow** | N/A -- the plugin allocates exactly what it needs |

**Trade-offs:**

- Always succeeds in a single call. No retry loop.
- Requires a `free_buffer` function pointer in the descriptor (the plugin must
  provide a way to deallocate from its own allocator).
- Slightly more overhead than CallerAllocated when output sizes are small and
  predictable.

### Arena (strategy = 2)

The host provides a pre-allocated arena. The plugin writes output into the
arena. Data is valid only until the next call.

```
FFI signature:
  unsafe extern "C" fn(
      in_ptr: *const u8, in_len: u32,     // serialized input
      arena_ptr: *mut u8, arena_cap: u32,  // host-provided arena
      out_offset: *mut u32,               // offset into arena where output starts
      out_len: *mut u32,                  // output length
  ) -> i32
```

| Aspect | Detail |
|--------|--------|
| **Who allocates** | Host (arena, once) |
| **Who writes** | Plugin (into the arena) |
| **Who frees** | No per-call free. Arena is reused. |
| **On overflow** | Returns `STATUS_BUFFER_TOO_SMALL` (-1); `out_len` contains the required size |

**Trade-offs:**

- Amortized zero allocation: the arena is allocated once and reused.
- Best throughput for high-frequency calls with bounded output.
- Data is only valid until the next call -- the host must copy if it needs to
  retain the output.
- Same size-guessing problem as CallerAllocated.

## Comparison Table

```
                     CallerAllocated     PluginAllocated       Arena
                     ───────────────     ───────────────       ─────
Allocs per call      0 (host pre-alloc)  1 (plugin Vec)        0 (reuse arena)
Retry on overflow    Yes                 No                    Yes
free_buffer needed   No                  Yes                   No
Output lifetime      Owned by host       Until free_buffer     Until next call
Best for             Bounded outputs     General purpose       High-frequency
```

## Why PluginAllocated Is the Default

PluginAllocated is the only strategy implemented in the MVP, and the macro
rejects CallerAllocated and Arena with compile-time errors:

```rust
// fidius-macro/src/interface.rs
match ir.attrs.buffer_strategy {
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
    BufferStrategyAttr::PluginAllocated => {}
}
```

PluginAllocated was chosen as the default for these reasons:

1. **Single-call guarantee.** The plugin allocates exactly the right amount
   of memory. No guessing, no retries. This simplifies host code and makes
   latency predictable.

2. **Safe deallocation.** The `free_buffer` function in the descriptor
   (`unsafe extern "C" fn(*mut u8, usize)`) reconstructs the `Vec` from the
   raw parts and drops it. This ensures the plugin's allocator handles the
   deallocation, avoiding cross-allocator bugs.

   ```rust
   // Generated by #[plugin_impl]:
   unsafe extern "C" fn __fidius_free_buffer_BlurFilter(ptr: *mut u8, len: usize) {
       if !ptr.is_null() && len > 0 {
           drop(unsafe { Vec::from_raw_parts(ptr, len, len) });
       }
   }
   ```

3. **No size coupling.** The host does not need to know anything about output
   sizes. This is important because serialized output sizes depend on the
   wire format (JSON is larger than bincode), the data content, and the plugin
   implementation -- none of which the host can predict.

4. **Correctness by construction.** The generated shim calls
   `wire::serialize(&output)` which produces a `Vec<u8>`, then does
   `mem::forget(output_bytes)` to hand the allocation to the host. The host
   reads the data and calls `free_buffer`. There is exactly one allocation
   and exactly one deallocation per call.

## Why This Is a Per-Trait Decision

The buffer strategy is set once on the `#[plugin_interface]` attribute, not
per-method. This is deliberate:

- **Vtable uniformity.** Every function pointer in the vtable has the same
  signature. The host code for calling methods (`PluginHandle::call_method`)
  uses a single `FfiFn` type alias. If strategies varied per method, the vtable
  would need mixed signatures and the calling code would need per-method
  dispatch.

- **Descriptor simplicity.** The `buffer_strategy` field is a single `u8` in
  the `PluginDescriptor`. The host checks it once at load time. Per-method
  strategies would require a per-method metadata array.

- **Cognitive simplicity.** Interface authors make one decision: "this
  interface uses PluginAllocated." Plugin authors do not need to think about
  buffer management at all -- the generated shims handle everything.

## When You Would Choose Each Strategy

*Note: CallerAllocated and Arena are designed but not yet implemented.*

**PluginAllocated** -- the right choice for most interfaces. Use when:

- Output sizes are unpredictable or vary widely
- Call frequency is moderate (not millions per second)
- You want the simplest possible plugin authoring experience

**CallerAllocated** -- use when:

- Output sizes are bounded and well-known (e.g., always returns a 256-byte
  struct)
- You want to avoid per-call allocations on the plugin side
- You are willing to handle the retry-on-overflow pattern in the host

**Arena** -- use when:

- You are calling the same method at very high frequency
- Output is consumed immediately and does not need to outlive the next call
- You want amortized zero-allocation throughput
- You are willing to copy data if you need to retain it

---

*Related documentation:*

- [Architecture Overview](architecture.md) -- the full pipeline and where buffer strategies fit
- [Wire Format](wire-format.md) -- how data is serialized before being written to buffers
- [Interface Evolution](interface-evolution.md) -- buffer strategy mismatches at load time
