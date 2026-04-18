<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# Buffer Strategies Explained

How memory ownership works across the FFI boundary, the two supported
strategies, and how to choose.

## The Core Problem

When a host calls a plugin method through the FFI boundary, the method
produces output bytes (serialized return value). Someone has to allocate
memory for those bytes. The question is: who allocates, who owns the
allocation, and who frees it? That's the buffer strategy.

## Two Strategies

Fidius supports two buffer strategies (since 0.1.0 — `CallerAllocated` was
removed; its value proposition was subsumed by Arena):

```rust
#[repr(u8)]
pub enum BufferStrategyKind {
    PluginAllocated = 1,
    Arena = 2,
}
```

The strategy is set **per-trait** on the `#[plugin_interface]` attribute
and must be matched on the corresponding `#[plugin_impl]`:

```rust
#[fidius::plugin_interface(version = 1, buffer = Arena)]
pub trait HotPath: Send + Sync {
    fn process(&self, input: String) -> String;
}

#[plugin_impl(HotPath, buffer = Arena)]
impl HotPath for MyImpl { /* ... */ }
```

### PluginAllocated (default)

The plugin allocates its own output buffer (via `Box<[u8]>`) and hands
the pointer back to the host. The host reads the data, then calls the
descriptor's `free_buffer` to deallocate.

```
FFI signature:
  unsafe extern "C" fn(
      in_ptr: *const u8, in_len: u32,
      out_ptr: *mut *mut u8,              // plugin sets to allocated pointer
      out_len: *mut u32,                  // plugin sets to length
  ) -> i32
```

| Aspect | Detail |
|--------|--------|
| **Who allocates** | Plugin (`Box<[u8]>`, then `Box::into_raw`) |
| **Who writes** | Plugin |
| **Who frees** | Host, via `descriptor.free_buffer(ptr, len)` |
| **On overflow** | N/A — plugin allocates exactly what it needs |
| **Panic message** | Transmitted — plugin allocates a buffer for it |

**Best for**: general-purpose plugins, unbounded or variable output
sizes, plugins with rich panic messages.

### Arena

The host maintains a thread-local arena pool. Before each call, a buffer
is acquired from the pool and passed to the plugin. The plugin writes
its output into the buffer. After the call, the host deserializes the
result and releases the buffer back to the pool.

```
FFI signature:
  unsafe extern "C" fn(
      in_ptr: *const u8, in_len: u32,
      arena_ptr: *mut u8, arena_cap: u32,
      out_offset: *mut u32,               // offset into arena where output starts
      out_len: *mut u32,                  // output length
  ) -> i32
```

| Aspect | Detail |
|--------|--------|
| **Who allocates** | Host (thread-local arena pool) |
| **Who writes** | Plugin (into host-provided arena) |
| **Who frees** | No per-call free — arena is reused |
| **On overflow** | Plugin returns `STATUS_BUFFER_TOO_SMALL` with needed size in `out_len`; host grows arena and retries once |
| **Panic message** | Not transmitted (arena may be too small); opaque `CallError::Panic` |

**Retry flow:**

1. Host calls plugin with current arena (default 4 KB initial).
2. Plugin writes required size to `out_len`, returns `STATUS_BUFFER_TOO_SMALL`.
3. Host grows the arena to at least `out_len` bytes and retries exactly once.
4. On second success, deserialize from `arena[out_offset..out_offset + out_len]`.
5. Second `STATUS_BUFFER_TOO_SMALL` returns `CallError::BufferTooSmall` (plugin misbehaving).

**Best for**: high-frequency calls where per-call allocation cost
matters, outputs that fit comfortably within a reasonable bounded size,
workloads tolerant of losing panic messages for speed.

## Comparison

```
                     PluginAllocated       Arena
                     ────────────────      ─────
Allocs per call      1 (plugin Box)        0 (reuse pooled arena)
Retry on overflow    No                    Yes (host grows arena once)
free_buffer needed   Yes                   No
Output lifetime      Until free_buffer     Until next call on same thread
Panic messages       Preserved             Opaque
Typical overhead     Medium                Low (amortized)
Best for             General purpose       Hot loops, perf-critical
```

## Why per-trait, not per-method

The strategy is set once per interface. This keeps vtable fn signatures
uniform (one fn type per vtable, not mixed), keeps the descriptor simple
(a single `u8` field, not per-method metadata), and keeps authoring
decisions to one choice per interface.

If you have mixed-cost methods, split them into two traits with
different strategies. The host can load both and call through typed
Clients.

## When to pick which

**Default to PluginAllocated.** It's simpler, preserves panic messages,
and the allocation cost is usually well under other framework overheads
(serialization, FFI call, etc.).

**Switch to Arena when you've measured plugin-side allocation as a
bottleneck.** Concretely: you have a call rate >10k/sec, your output is
bounded and typically <4 KB, and you've confirmed via profiling that the
plugin's per-call `Box` allocation dominates. Otherwise Arena's costs
(lost panic messages, retry path complexity, thread-local pool
coordination) are not worth it.

**Do not mix strategies in one interface** — fidius rejects that at the
macro level. Split the interface into two traits if you need both.

## Implementation notes

- **Arena pool is thread-local.** Multi-threaded hosts get one pool per
  thread, no mutex contention.
- **Arena pool doesn't shrink.** Buffers grow on demand and stay at peak
  size for the thread's lifetime.
- **Panic messages under Arena are intentionally opaque.** The shim's
  catch_unwind handler can't transmit a message without potentially
  writing past the arena's capacity — it returns `out_len = 0` and
  `STATUS_PANIC`. If panic debugging matters, use PluginAllocated.
- **The `buffer` attribute must match between interface and impl.**
  `#[plugin_impl(Trait, buffer = Arena)]` is required when the interface
  uses Arena. A mismatch produces a vtable fn-pointer type error at
  compile time.
