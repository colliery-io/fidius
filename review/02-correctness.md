# Correctness Review: Fidius Plugin Framework

**Reviewer lens**: Does this do what it claims, under all expected conditions?

**Date**: 2026-03-28

---

## Summary

The fidius framework demonstrates solid correctness for its core happy paths: interface definition, plugin implementation, FFI shim generation, registry assembly, host loading, and method calling all work as designed and are tested end-to-end. The test suite covers the primary workflows well, with a genuine full-pipeline test that scaffolds, builds, signs, loads, and calls a plugin from scratch.

However, the review identified several critical and major correctness issues, primarily in the unsafe FFI layer. The most severe is the complete absence of vtable bounds checking in `call_method`, which can cause undefined behavior on any out-of-range index. The `free_buffer` implementation uses an incorrect capacity assumption. There are also subtle issues with optional method vtable layout assumptions and several untested error paths. The codebase relies on a number of implicit invariants that are not enforced by the type system or runtime checks.

**Finding count**: 7 Critical, 6 Major, 5 Minor, 5 Observations

---

## Test Coverage Assessment

### What is tested well

- **ABI layout stability**: Struct sizes, alignments, and field offsets are asserted for `PluginRegistry` and `PluginDescriptor`. These are genuine ABI guards.
- **Wire format round-trip**: Both JSON (debug) and bincode (release) paths are covered, including `PluginError` serialization.
- **Interface hash determinism and order-independence**: Covered with regression vectors.
- **Macro code generation**: Interface and impl macros tested for vtable generation, descriptor correctness, shim invocation, and multi-plugin registries.
- **Full pipeline**: The `full_pipeline.rs` test in fidius-cli exercises the entire scaffold-to-call workflow, providing high confidence in the primary path.
- **Signing enforcement**: Strict vs. Lenient policies, correct key, wrong key, missing signature all tested.
- **Compile-fail tests**: Missing version, `&mut self`, unsupported buffer strategy all produce compile errors.
- **Package manifest parsing**: Valid, invalid, missing, extra fields, typed and untyped schemas all tested.

### What is not tested

- **vtable index out-of-bounds**: No test attempts an invalid index.
- **Concurrent plugin loading or method calling**: No thread-safety tests despite `Send + Sync` impls.
- **Plugin that panics**: No test verifies `STATUS_PANIC` handling end-to-end.
- **Plugin that returns `Result::Err`**: No test verifies `STATUS_PLUGIN_ERROR` propagation through `PluginHandle::call_method`.
- **Deserialization failure at FFI boundary**: No test sends malformed input bytes.
- **Serialization overflow**: No test for output exceeding `u32::MAX` bytes.
- **Empty registry** (zero plugins in dylib): Not tested.
- **Multiple search paths**: Only single search path tested.
- **Architecture mismatch rejection**: `check_architecture` tested at detection level but not through `load_library`.
- **`has_capability` with invalid bit**: Only tested with valid bits.
- **`discover()` with signature requirements**: Discover does not check signatures at all, but this is not tested/documented.
- **`build_package` failure paths**: No test for build failure.

### Test Quality

Tests are deterministic and well-isolated. They use real assertions, not just smoke checks. The use of `tempfile` for filesystem tests prevents cross-test interference. The `trybuild` tests verify exact compiler error messages.

Potential flakiness concern: Several tests share the same test-plugin-smoke build output directory. Parallel test execution could cause race conditions on the dylib and `.sig` files. The e2e signing tests write/delete `.sig` files in the shared build directory.

---

## Key Risk Areas

1. **Unsafe FFI boundary** (handle.rs, impl_macro.rs): Raw pointer arithmetic, unchecked indices, manual memory management.
2. **Vtable layout assumptions**: `call_method` treats the vtable as a flat array of `FfiFn`, but optional methods are `Option<FfiFn>`.
3. **Memory management**: `free_buffer` assumes `capacity == len`, generated shims use `mem::forget` on `Vec`.
4. **Registry construction**: `build_registry` leaks a `Vec` to get a static pointer; `entries.len() as u32` can silently truncate.

---

## Findings

### COR-01: vtable index out-of-bounds causes undefined behavior (Critical)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/handle.rs`, lines 103-106

**Description**: `call_method` reads a function pointer from the vtable using raw pointer arithmetic (`fn_ptrs.add(index)`) with no bounds check. There is no vtable size stored in the descriptor, no method count field, and no runtime validation. Passing an out-of-bounds index reads arbitrary memory as a function pointer and calls it.

```rust
let fn_ptr = unsafe {
    let fn_ptrs = self.vtable as *const FfiFn;
    *fn_ptrs.add(index)
};
```

**Impact**: Calling `handle.call_method(999, &input)` causes undefined behavior. Since the index is a plain `usize` from user code, this is easy to trigger accidentally (e.g., wrong method ordering between interface versions).

**Recommendation**: Add a `method_count` field to `PluginDescriptor` and validate `index < method_count` before the unsafe pointer read. Alternatively, generate typed wrapper methods that use compile-time-known indices.

---

### COR-02: Optional method vtable slots assumed to be same size as bare fn pointers (Critical)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/handle.rs`, lines 103-106

**Description**: The vtable struct generated by `#[plugin_interface]` uses `Option<unsafe extern "C" fn(...)>` for optional methods and bare `unsafe extern "C" fn(...)` for required methods. `call_method` casts the entire vtable to `*const FfiFn` (a bare function pointer type) and indexes into it as a flat array. This relies on the nullable pointer optimization guaranteeing that `Option<fn>` has the same layout as a bare function pointer. While this optimization is guaranteed by Rust for `Option<fn>` (function pointers are non-null), calling an optional method that is `None` would call a null function pointer, because `call_method` does not check `has_capability` before invoking.

**Impact**: If a host calls an optional method that a plugin does not implement, it will call a null function pointer, causing a segfault. There is no automatic check in `call_method` and the `CallError::NotImplemented` variant exists but is never returned by any code path.

**Recommendation**: Either check `has_capability` inside `call_method` for optional method indices, or read through the typed vtable struct rather than raw pointer arithmetic. At minimum, document the invariant that callers must check `has_capability` first.

---

### COR-03: `free_buffer` uses incorrect Vec capacity (Critical)

**File**: `/Users/dstorey/Desktop/fides/fidius-macro/src/impl_macro.rs`, lines 106-110

**Description**: The generated `free_buffer` function reconstructs a `Vec` from the raw pointer using `Vec::from_raw_parts(ptr, len, len)`, setting capacity equal to length. However, the shim that allocates the buffer does `let ptr = output_bytes.as_ptr()` / `std::mem::forget(output_bytes)`, where `output_bytes` is a `Vec<u8>` whose capacity may exceed its length (as is common with `Vec`). Dropping a `Vec` with the wrong capacity is undefined behavior because the allocator may attempt to deallocate a different size than was originally allocated.

```rust
// Generated shim (allocation side):
let len = output_bytes.len();
let ptr = output_bytes.as_ptr() as *mut u8;
std::mem::forget(output_bytes);
unsafe {
    *out_ptr = ptr;
    *out_len = len as u32;
}

// Generated free_buffer:
unsafe extern "C" fn __fidius_free_buffer_...(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        drop(unsafe { Vec::from_raw_parts(ptr, len, len) });  // capacity != original
    }
}
```

**Impact**: Undefined behavior on every plugin method call where the serialization `Vec` has excess capacity. In practice, many allocators tolerate this, but it is unsound and can cause heap corruption with some allocators or under sanitizers.

**Recommendation**: Either (a) call `output_bytes.shrink_to_fit()` before `forget` to ensure `capacity == len`, or (b) pass both `len` and `capacity` through the FFI boundary (add a capacity out-parameter), or (c) use `into_raw_parts()` and pass both `len` and `capacity` separately.

---

### COR-04: Output size truncation from `usize` to `u32` (Major)

**File**: `/Users/dstorey/Desktop/fides/fidius-macro/src/impl_macro.rs`, line 200

**Description**: The generated shim casts `output_bytes.len()` (a `usize`) to `u32` when writing to `*out_len`. On 64-bit systems, if the serialized output exceeds `u32::MAX` (~4 GB), this silently truncates the length. The host then reads a partial buffer and attempts deserialization, which will either fail with a confusing deserialization error or, worse, produce incorrect data.

```rust
*out_len = len as u32;
```

Similarly, the input length is `u32`, capping inputs at 4 GB.

**Impact**: Silent data corruption for large payloads. Low probability in current usage patterns, but the framework does not document this limit.

**Recommendation**: Add a size check before the cast: `assert!(len <= u32::MAX as usize, "output too large for FFI")`. Document the 4 GB limit in the interface.

---

### COR-05: `discover()` silently skips signature verification (Major)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/host.rs`, lines 128-167

**Description**: `PluginHost::discover()` does not check signatures, even when `require_signature` is true. It only validates interface hash/wire/strategy. The `load()` method checks signatures, but `discover()` returns `PluginInfo` for unsigned/tampered plugins. A host using `discover()` to list available plugins will show plugins that `load()` would reject.

**Impact**: Inconsistent behavior between discovery and loading. A host that shows discovered plugins to users creates false expectations about what can be loaded.

**Recommendation**: Apply the same signature check in `discover()` as in `load()`, or document the inconsistency clearly.

---

### COR-06: `STATUS_PANIC` does not set output buffer, host reads null pointer (Major)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/handle.rs`, lines 148, 156-158

**Description**: When a plugin method panics, `catch_unwind` catches it and returns `STATUS_PANIC`. The host checks for `STATUS_PANIC` and returns `Err(CallError::Panic)` -- so far, so good. But if the panic occurs *after* the output buffer has been set (e.g., during serialization of a successful result), the output pointer may have been written. More importantly, on the `STATUS_OK` path, the host unconditionally reads from `out_ptr` regardless of whether it's null:

```rust
// After the match on status (STATUS_OK falls through):
let output_slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
```

If a method returns `STATUS_OK` but does not set `out_ptr` (which shouldn't happen with correct generated code but could happen with hand-written plugins), this creates a slice from a null pointer, which is undefined behavior.

**Impact**: Low likelihood with generated code, but the host API accepts arbitrary vtables. A manually constructed or malicious plugin could trigger UB in the host.

**Recommendation**: Add a null check for `out_ptr` on the `STATUS_OK` path.

---

### COR-07: `build_registry` truncates plugin count from `usize` to `u32` (Major)

**File**: `/Users/dstorey/Desktop/fides/fidius-core/src/registry.rs`, line 39

**Description**: `let count = entries.len() as u32;` silently truncates if there are more than 2^32 descriptor entries. While extremely unlikely in practice, this is an unsound truncation in the same pattern as COR-04.

**Impact**: Negligible in practice, but a correctness defect.

**Recommendation**: Add an assertion: `assert!(entries.len() <= u32::MAX as usize)`.

---

### COR-08: `detect_architecture` reads entire file into memory (Major)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/arch.rs`, line 69

**Description**: `std::fs::read(path)` reads the entire dylib file into memory just to inspect the first 20 bytes of the header. Plugin dylibs can be tens of megabytes or more.

```rust
let bytes = std::fs::read(path).map_err(|_| LoadError::LibraryNotFound {
    path: path.display().to_string(),
})?;
```

This function is called for *every* dylib during `discover()` and `load()`, meaning scanning a directory with many large dylibs incurs massive unnecessary I/O and memory allocation.

**Impact**: Performance-correctness issue: OOM possible on resource-constrained systems when scanning directories with large dylibs.

**Recommendation**: Use `std::fs::File::open` + `Read::read_exact` to read only the header bytes needed (20 bytes max).

---

### COR-09: `signing::verify_signature` reads entire dylib into memory (Major)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/signing.rs`, line 61

**Description**: Similar to COR-08, `verify_signature` reads the entire dylib into memory with `std::fs::read`. Combined with COR-08, loading a single signed plugin reads the entire dylib file twice (once for arch detection, once for signature verification), and then `libloading` maps it a third time.

**Impact**: 3x memory overhead for loading signed plugins.

**Recommendation**: Cache the file bytes between arch detection and signature verification, or use memory-mapped I/O for signing.

---

### COR-10: `init_interface` uses same dep string for both `fidius` and `fidius-core` (Minor)

**File**: `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs`, lines 93-103

**Description**: `init_interface` resolves the `fidius` dependency using `resolve_dep("fidius", version)` and then uses that same resolved string for `fidius-core`:

```rust
let fidius_dep = resolve_dep("fidius", version);
let cargo_toml = format!(
    ...
    fidius = {fidius_dep}
    fidius-core = {fidius_dep}
    ...
);
```

If `fidius` resolves to a path like `{ path = "/some/path" }`, `fidius-core` gets the same path, which is incorrect -- `fidius` and `fidius-core` are different crates at different paths.

**Impact**: Scaffolded interface crate will have incorrect `fidius-core` dependency when using local path resolution.

**Recommendation**: Resolve `fidius-core` separately, or derive its path from the `fidius` path.

---

### COR-11: `init_plugin` hardcodes `fidius-core = { version = "0.1" }` (Minor)

**File**: `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs`, line 164

**Description**: The scaffolded plugin's `Cargo.toml` always writes `fidius-core = { version = "0.1" }`, regardless of the actual version (currently `0.0.0-alpha.1`). This version requirement will fail to resolve.

**Impact**: Scaffolded plugins cannot build without manual intervention.

**Recommendation**: Use the same dependency resolution logic as the interface dep, or use the actual version.

---

### COR-12: `detect_architecture` maps IO error to `LibraryNotFound` (Minor)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/arch.rs`, lines 69-71

**Description**: `std::fs::read(path).map_err(|_| LoadError::LibraryNotFound { ... })` discards the original IO error (e.g., permission denied, disk error) and replaces it with `LibraryNotFound`, which is misleading.

**Impact**: Incorrect error diagnosis. A permission-denied error looks like "library not found."

**Recommendation**: Preserve the original IO error: use `LoadError::Io(e)` for non-`NotFound` errors.

---

### COR-13: `has_capability` panics instead of returning error (Minor)

**File**: `/Users/dstorey/Desktop/fides/fidius-core/src/descriptor.rs`, line 194, `/Users/dstorey/Desktop/fides/fidius-host/src/handle.rs`, line 171

**Description**: `has_capability(bit)` uses `assert!(bit < 64)` which panics on invalid input. For a host-side API, panicking on invalid input is inappropriate -- it should return `false` or `Err`.

**Impact**: Host application crashes on programmer error instead of graceful failure.

**Recommendation**: Return `false` for bits >= 64, or change to `Result`.

---

### COR-14: `buffer_strategy_kind()` and `wire_format_kind()` panic on unknown values (Minor)

**File**: `/Users/dstorey/Desktop/fides/fidius-core/src/descriptor.rs`, lines 174-190

**Description**: These methods use `panic!` on unknown discriminant values. When reading descriptors from a loaded dylib, the u8 values come from potentially untrusted data. A corrupted or future-version plugin with unknown strategy/format values will crash the host.

**Impact**: Host process crash from loading a malformed plugin, instead of graceful error.

**Recommendation**: Return `Result` or an `Unknown` variant, allowing the loader to reject the plugin gracefully.

---

### COR-15: E2E signing tests share mutable state in test-plugin-smoke build directory (Observation)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/tests/e2e.rs`

**Description**: Multiple tests in `e2e.rs` write and delete `.sig` files in the shared `tests/test-plugin-smoke/target/debug/` directory. When Cargo runs tests in parallel, these tests can interfere with each other (e.g., one test's `cleanup_sig` removing a `.sig` file that another test just created).

**Impact**: Potential test flakiness under parallel execution.

**Recommendation**: Use per-test temporary directories by copying the dylib to a temp location before signing.

---

### COR-16: No test for `Result`-returning plugin methods through the host (Observation)

**Description**: The test-plugin-smoke fixture uses methods that return plain values (`AddOutput`, `MulOutput`), not `Result<T, PluginError>`. The `STATUS_PLUGIN_ERROR` path in `call_method` is exercised by no integration test. The code for serializing errors in the generated shim (the `returns_result` branch in `generate_shims`) is not tested end-to-end.

**Impact**: Untested error propagation path. If there's a serialization mismatch for `PluginError`, it won't be caught.

**Recommendation**: Add a test fixture with a method that returns `Result<T, PluginError>` and test both `Ok` and `Err` paths through `PluginHandle::call_method`.

---

### COR-17: `package_sign` signs `package.toml` using the dylib signing function (Observation)

**File**: `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs`, lines 391-397

**Description**: `package_sign` calls `sign(key_path, &manifest_path)` which produces a `.sig` file named `package.toml.sig` (the `sign` function appends `.sig` after the existing extension). The `package_verify` function calls `verify(key_path, &manifest_path)` which looks for `package.toml.sig`. This works but note that the `sign` function constructs the sig path as `path.with_extension("toml.sig")`, which replaces `.toml` with `.toml.sig`, producing `package.toml.sig` -- correct by coincidence. If the manifest file had no extension (just `package`), `with_extension` would produce `package.sig`, but looking at the actual flow, `package.toml` always has the `.toml` extension so this works.

**Impact**: None currently, but fragile sig-path construction.

---

### COR-18: `verify` function calls `process::exit(1)` instead of returning error (Observation)

**File**: `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs`, line 271

**Description**: The `verify` command calls `std::process::exit(1)` on verification failure instead of returning `Err(...)`. This bypasses the normal error handling in `main()` and prevents cleanup (e.g., destructors, temp file cleanup).

```rust
Err(_) => {
    eprintln!("Signature INVALID: {}", dylib_path.display());
    std::process::exit(1);
}
```

**Impact**: Inconsistent error handling. All other commands return `Err`, but `verify` hard-exits.

**Recommendation**: Return `Err("Signature INVALID: ...")` like other commands.

---

### COR-19: Hash known vectors test verifies determinism but not specific values (Observation)

**File**: `/Users/dstorey/Desktop/fides/fidius-core/tests/layout_and_roundtrip.rs`, lines 194-222

**Description**: The `hash_known_vectors` test comments say "Hardcode after first run -- these are the golden values" but then only asserts determinism (same input produces same output) and distinctness, not specific hash values. If the FNV-1a implementation changes, these tests would still pass as long as the new implementation is also deterministic. True regression vectors should assert exact numeric values.

**Impact**: Hash algorithm changes would not be detected by this test, potentially causing silent ABI incompatibility between old and new plugins.

**Recommendation**: Compute the actual hash values once and hardcode them as expected values in the assertions.
