# Performance Review: Fidius Plugin Framework

**Reviewer lens**: Does this use resources appropriately for its workload?
**Finding prefix**: PRF-
**Date**: 2026-03-28

---

## Summary

Fidius is an alpha-stage plugin framework where the primary hot path is `PluginHandle::call_method` -- serialize input, call an FFI function pointer, deserialize output, free the buffer. The framework's performance profile is generally appropriate for its workload: FFI dispatch is thin, vtable lookup is O(1) pointer arithmetic, and the serialization layer delegates to well-optimized libraries (serde_json in debug, bincode in release). Most of the resource usage concerns are in cold paths (plugin loading, discovery, signing) rather than the call path.

Two significant findings relate to the hot path: per-call heap allocation on both sides of the FFI boundary (inherent to PluginAllocated strategy but worth noting for future optimization) and the `catch_unwind` cost on every shim invocation. The most impactful cold-path issue is `detect_architecture` reading entire dylib files into memory just to inspect header bytes.

Overall, the codebase avoids premature optimization -- the code is clean, readable, and appropriately simple for an alpha. No unnecessary concurrency or complex caching mechanisms exist. The main actionable items are in cold-path resource waste.

---

## Workload Assessment

**Expected workload pattern**: The framework serves two distinct phases:

1. **Plugin loading (cold path)**: Happens once or infrequently. Involves file I/O (reading dylibs, signature files), dlopen, registry validation. Acceptable to be slower.

2. **Method calling (hot path)**: Happens repeatedly at application-driven frequency. The path is: `wire::serialize` -> vtable index -> FFI call -> (inside plugin) `wire::deserialize` -> method body -> `wire::serialize` -> (back in host) `wire::deserialize` -> `free_buffer`. This is where performance matters.

**Concurrency model**: None. The framework is single-threaded from the host's perspective. `PluginHandle` is `Send + Sync` so hosts can share it across threads, but no internal concurrency exists. This is appropriate -- the framework should not impose a concurrency model on the host.

---

## Hot Path Analysis

### Method Call Path (`PluginHandle::call_method`)

**Host side** (`fidius-host/src/handle.rs:93-167`):
1. `wire::serialize(input)` -- allocates a `Vec<u8>` (heap allocation 1)
2. Vtable lookup -- `*fn_ptrs.add(index)` -- O(1), zero allocation, good
3. FFI call -- direct function pointer invocation, minimal overhead
4. `std::slice::from_raw_parts` -- zero-copy view of plugin output, good
5. `wire::deserialize(output_slice)` -- allocates deserialized value (heap allocation 2)
6. `free_buffer(out_ptr, out_len)` -- frees plugin-side allocation

**Plugin side** (generated shim, `fidius-macro/src/impl_macro.rs:178-209`):
1. `catch_unwind` wraps everything (cost: landing pad setup, ~5-15ns per call)
2. `std::slice::from_raw_parts` -- zero-copy view of input, good
3. `wire::deserialize(in_slice)` -- allocates deserialized args (heap allocation 3)
4. Method call -- user code
5. `wire::serialize(&output)` -- allocates `Vec<u8>` (heap allocation 4)
6. `std::mem::forget(output_bytes)` -- transfers ownership to host, appropriate

**Total allocations per call**: minimum 4 heap allocations (2 serialize, 2 deserialize), plus whatever the method body does. This is inherent to the PluginAllocated + serde wire format design and not avoidable without a fundamentally different approach (e.g., Arena strategy, zero-copy serialization).

---

## Findings

### PRF-001: `detect_architecture` reads entire dylib into memory [Major]

**File**: `fidius-host/src/arch.rs:68-69`
**Code**: `let bytes = std::fs::read(path).map_err(...)?;`

The function reads the entire dylib file into memory (`std::fs::read`) just to inspect the first 4-20 bytes of the header for magic number and architecture detection. Plugin dylibs can be many megabytes. This function is called on every dylib during `load_library` (which is called during both `discover()` and `load()`).

**Impact**: During `discover()`, every dylib in all search paths is fully read into memory just for architecture checking. If a directory contains 50 dylibs at 5MB each, this is 250MB of transient allocations.

**Recommendation**: Read only the first ~20 bytes using `File::open` + `Read::read_exact` with a small stack buffer:
```rust
let mut buf = [0u8; 20];
let mut file = std::fs::File::open(path)?;
let n = file.read(&mut buf)?;
```

### PRF-002: Signing reads entire dylib twice during `load()` [Minor]

**File**: `fidius-host/src/host.rs:189-199`, `fidius-host/src/signing.rs:61`

When `require_signature` is enabled and `load()` is called, the dylib file is read entirely into memory by `verify_signature` (line 61: `let dylib_bytes = std::fs::read(dylib_path)?`), and then separately the same file is `dlopen`-ed by `load_library`. The `detect_architecture` call inside `load_library` reads it a third time (see PRF-001). So with signatures enabled, the dylib is fully read into memory at least twice (once for signing, once for arch detection) plus the OS-level mmap for dlopen.

**Impact**: For cold path only (plugin loading). Doubled memory pressure during load. Not critical for typical usage but wasteful.

**Recommendation**: After fixing PRF-001 (arch detection reads only header), the remaining full read for signature verification is justified since Ed25519 needs to hash the entire file. No further action needed beyond PRF-001.

### PRF-003: `discover()` loads and validates every dylib, discarding all results [Minor]

**File**: `fidius-host/src/host.rs:128-167`

`discover()` calls `load_library()` on every dylib in all search paths. `load_library` performs `dlopen` (which maps the library into memory and runs its constructors), validates descriptors, and returns `LoadedLibrary`. The `LoadedLibrary` (including its `Arc<Library>`) is then dropped, causing `dlclose`. If the host later calls `load()`, the same library is opened again from scratch.

**Impact**: Discovery is O(n * load_cost) where `load_cost` includes dlopen + constructor execution + dlclose. For large plugin directories this is unnecessarily expensive. However, this is a cold path.

**Recommendation**: Consider caching discovered libraries or offering a `discover_and_retain()` method. Alternatively, document that `discover()` is expensive and should be called sparingly. Not urgent for an alpha.

### PRF-004: Per-call `catch_unwind` overhead in generated shims [Observation]

**File**: `fidius-macro/src/impl_macro.rs:185`

Every generated FFI shim wraps the entire method body in `std::panic::catch_unwind(AssertUnwindSafe(...))`. This is the correct safety measure for FFI boundaries -- unwinding across `extern "C"` is undefined behavior. The cost is approximately 5-15ns per call on modern x86_64 with zero-cost exception handling (landing pad setup, no cost on the non-panicking path with DWARF unwinding).

**Impact**: Negligible for most workloads. The serialization overhead (microseconds) dominates. Correctly prioritizes safety over performance.

**Recommendation**: None. This is the right tradeoff.

### PRF-005: Multi-thread tokio runtime per plugin dylib [Minor]

**File**: `fidius-core/src/async_runtime.rs:25-31`

Each plugin dylib with the `async` feature gets its own lazily-initialized multi-thread tokio runtime (`Builder::new_multi_thread().enable_all()`). A multi-thread runtime spawns worker threads equal to the number of CPU cores by default. If a host loads 5 async plugin dylibs, this creates 5 separate runtimes with potentially 5 * N_CORES worker threads.

**Impact**: Thread proliferation. On an 8-core machine with 5 async plugins, this could spawn 40 runtime worker threads, most sitting idle. Each thread consumes ~8MB of stack space by default.

**Recommendation**: Consider using `new_current_thread()` instead, since the shim calls `block_on()` synchronously anyway -- a multi-thread runtime provides no benefit when the calling pattern is always `block_on(single_future)`. Alternatively, document the thread cost and consider allowing host-provided runtime injection.

### PRF-006: `free_buffer` reconstructs Vec with `len == capacity` assumption [Observation]

**File**: `fidius-macro/src/impl_macro.rs:107-110`

```rust
unsafe extern "C" fn __fidius_free_buffer(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        drop(unsafe { Vec::from_raw_parts(ptr, len, len) });
    }
}
```

The free function reconstructs a `Vec` with `capacity == len`. On the plugin side, the serialized output `Vec` is `forget`-ed after recording `len = output_bytes.len()`. However, `Vec::len()` and `Vec::capacity()` are different -- the serialized `Vec` may have excess capacity from the serializer's internal growth strategy. Passing `len` as both length and capacity to `from_raw_parts` is technically correct for deallocation purposes (the allocator only needs the pointer, and using `len` as capacity means it will only drop `len` elements), but if the `Vec` had a larger capacity than length, the excess memory between `len` and actual capacity becomes leaked (it was allocated but never freed, since the allocator sees a smaller capacity).

**Impact**: Minor memory leakage per call. The leaked amount per call is typically small (serializers often fill close to capacity), but it accumulates over many calls without ever being reclaimed.

**Recommendation**: Shrink the `Vec` before forgetting it on the plugin side (`output_bytes.shrink_to_fit()`) or pass both length and capacity across the FFI boundary. The latter requires changing the FFI signature, so the simpler fix is adding `shrink_to_fit()`:
```rust
let mut output_bytes = wire::serialize(&output)?;
output_bytes.shrink_to_fit();
let len = output_bytes.len();
let ptr = output_bytes.as_ptr() as *mut u8;
std::mem::forget(output_bytes);
```

### PRF-007: Registry `build_registry` leaks a Vec intentionally [Observation]

**File**: `fidius-core/src/registry.rs:34-49`

```rust
fn build_registry() -> PluginRegistry {
    let entries: Vec<*const PluginDescriptor> = ...;
    let count = entries.len() as u32;
    let ptr = entries.as_ptr();
    std::mem::forget(entries);
    ...
}
```

The registry is built once (via `OnceLock`) and the backing `Vec` is intentionally leaked to produce a `'static` pointer. This is a standard pattern for plugin frameworks and FFI. The leak is bounded (once per dylib lifetime, proportional to number of plugins which is tiny).

**Impact**: None. Correct and appropriate use of intentional leak.

**Recommendation**: None. Consider adding `Box::leak` instead of `forget` for clarity, though the current approach is fine:
```rust
let entries = entries.into_boxed_slice();
let ptr = Box::leak(entries).as_ptr();
```

### PRF-008: No vtable bounds checking on `call_method` index [Observation]

**File**: `fidius-host/src/handle.rs:103-106`

```rust
let fn_ptr = unsafe {
    let fn_ptrs = self.vtable as *const FfiFn;
    *fn_ptrs.add(index)
};
```

There is no bounds check on `index` before dereferencing the vtable pointer. An out-of-bounds index reads past the vtable struct, interpreting arbitrary memory as a function pointer. This is primarily a correctness/safety issue (covered in other review lenses), but it has a performance dimension: adding a bounds check would cost a single comparison + branch, which is negligible compared to serialization costs.

**Impact**: No performance impact from the missing check. Adding one would have no measurable cost.

**Recommendation**: Add a bounds check (this is primarily a safety recommendation, not performance).

### PRF-009: Bincode v1 vs v2 [Observation]

**File**: `fidius-core/Cargo.toml` (bincode 1 dependency), `fidius-core/src/wire.rs`

The project uses bincode v1, which is in maintenance mode. Bincode v2 offers meaningful performance improvements (faster varint encoding, better buffer reuse, support for borrowed deserialization). For the hot-path wire format in release builds, this is the serialization library that matters most.

**Impact**: Minor. Bincode v1 is already fast. V2 improvements are incremental.

**Recommendation**: Consider upgrading to bincode v2 when stabilizing beyond alpha. The API differs significantly, so this is not urgent.

### PRF-010: `interface_hash` allocates unnecessarily [Observation]

**File**: `fidius-core/src/hash.rs:47-52`

```rust
pub fn interface_hash(signatures: &[&str]) -> u64 {
    let mut sorted: Vec<&str> = signatures.to_vec();
    sorted.sort();
    let combined = sorted.join("\n");
    fnv1a(combined.as_bytes())
}
```

This function allocates a `Vec` (for sorting) and a `String` (for joining). It is called at compile time by the proc macro, not at runtime. There is no runtime performance impact.

**Impact**: None at runtime. Compile-time only.

**Recommendation**: None. The `fnv1a` function itself is `const fn` and efficient. The non-const `interface_hash` wrapper is only used at macro expansion time.
