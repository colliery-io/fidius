<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# ABI Specification

Wire protocol, binary layout, and ABI contract between Fidius hosts and plugins.

**Source:** `fidius-core/src/descriptor.rs`, `fidius-core/src/status.rs`, `fidius-core/src/wire.rs`, `fidius-core/src/hash.rs`, `fidius-core/tests/layout_and_roundtrip.rs`

---

## Magic Bytes

```rust
pub const FIDIUS_MAGIC: [u8; 8] = *b"FIDIUS\0\0";
```

The first 8 bytes of every `PluginRegistry`. Used by the host to verify the registry pointer is valid before reading further fields.

---

## Version Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `REGISTRY_VERSION` | `1` | Layout version of the `PluginRegistry` struct. |
| `ABI_VERSION` | `1` | Layout version of the `PluginDescriptor` struct. |

Both are `u32`. The host rejects registries or descriptors with mismatched versions.

---

## PluginRegistry Layout

> **Note:** All sizes and offsets in this section assume a 64-bit platform
> (pointer size = 8 bytes). On 32-bit platforms, pointer fields are 4 bytes and
> total struct sizes differ.

`#[repr(C)]`, 24 bytes, 8-byte aligned (on 64-bit platforms).

| Offset | Size | Field | Type | Description |
|--------|------|-------|------|-------------|
| 0 | 8 | `magic` | `[u8; 8]` | Must equal `b"FIDIUS\0\0"`. |
| 8 | 4 | `registry_version` | `u32` | Must equal `REGISTRY_VERSION` (currently `1`). |
| 12 | 4 | `plugin_count` | `u32` | Number of descriptor pointers in the array. |
| 16 | 8 | `descriptors` | `*const *const PluginDescriptor` | Pointer to array of `plugin_count` descriptor pointers. |

Each `PluginRegistry` is constructed once per dylib by `build_registry()` and cached in a `OnceLock`. The `descriptors` field points to a leaked `Vec` of pointers collected via the `inventory` crate.

---

## PluginDescriptor Layout

`#[repr(C)]`, 72 bytes, 8-byte aligned (on 64-bit platforms).

| Offset | Size | Field | Type | Description |
|--------|------|-------|------|-------------|
| 0 | 4 | `abi_version` | `u32` | Must equal `ABI_VERSION` (currently `1`). |
| 4 | 4 | _(padding)_ | | Alignment padding before pointer. |
| 8 | 8 | `interface_name` | `*const c_char` | Null-terminated UTF-8 C string. Interface trait name. |
| 16 | 8 | `interface_hash` | `u64` | FNV-1a hash of required method signatures. |
| 24 | 4 | `interface_version` | `u32` | User-specified version from `#[plugin_interface(version = N)]`. |
| 28 | 4 | _(padding)_ | | Alignment padding before `u64`. |
| 32 | 8 | `capabilities` | `u64` | Bitfield: bit N set means optional method N is implemented. |
| 40 | 1 | `wire_format` | `u8` | `0` = Json, `1` = Bincode. See [Wire Format](#wireformat). |
| 41 | 1 | `buffer_strategy` | `u8` | `0` = CallerAllocated, `1` = PluginAllocated, `2` = Arena. |
| 42 | 6 | _(padding)_ | | Alignment padding before pointer. |
| 48 | 8 | `plugin_name` | `*const c_char` | Null-terminated UTF-8 C string. Plugin implementation name. |
| 56 | 8 | `vtable` | `*const c_void` | Opaque pointer to the interface-specific `#[repr(C)]` vtable. |
| 64 | 8 | `free_buffer` | `Option<unsafe extern "C" fn(*mut u8, usize)>` | Buffer deallocation function. Must be `Some` for `PluginAllocated`. |

### PluginDescriptor Helper Methods

`PluginDescriptor` provides the following convenience methods:

| Method | Return type | Description |
|--------|-------------|-------------|
| `interface_name_str()` | `&str` | Reads `interface_name` as a `CStr` and converts to `&str`. |
| `plugin_name_str()` | `&str` | Reads `plugin_name` as a `CStr` and converts to `&str`. |
| `buffer_strategy_kind()` | `BufferStrategyKind` | Converts the `buffer_strategy` `u8` to the enum. |
| `wire_format_kind()` | `WireFormat` | Converts the `wire_format` `u8` to the enum. |
| `has_capability(bit: u32)` | `bool` | Returns `true` if the given capability bit is set. |

---

## BufferStrategyKind

`#[repr(u8)]`, 1 byte.

| Discriminant | Variant | Description |
|--------------|---------|-------------|
| `0` | `CallerAllocated` | Host allocates output buffer. Returns `-1` with needed size if too small. |
| `1` | `PluginAllocated` | Plugin allocates output. Host frees via `free_buffer`. |
| `2` | `Arena` | Host provides pre-allocated arena. Data valid until next call. |

Only `PluginAllocated` is currently supported by the macro.

---

## WireFormat

`#[repr(u8)]`, 1 byte.

| Discriminant | Variant | Description |
|--------------|---------|-------------|
| `0` | `Json` | JSON via `serde_json`. Used in debug builds (`cfg(debug_assertions)`). |
| `1` | `Bincode` | bincode. Used in release builds (`cfg(not(debug_assertions))`). |

---

Layout sizes and offsets are regression-tested in `fidius-core/tests/layout_and_roundtrip.rs` to catch accidental ABI drift.

---

## Wire Format Selection

The wire format is determined at compile time. For a detailed explanation of the debug/release behavior and the rationale behind `PluginError.details`, see [Wire Format and Debug/Release Behavior](../explanation/wire-format.md).

```rust
#[cfg(debug_assertions)]
pub const WIRE_FORMAT: WireFormat = WireFormat::Json;

#[cfg(not(debug_assertions))]
pub const WIRE_FORMAT: WireFormat = WireFormat::Bincode;
```

The host rejects plugins whose `wire_format` field does not match its own `WIRE_FORMAT`. Both host and plugin must be compiled in the same mode (both debug or both release).

### Serialization

```rust
// Debug: serde_json::to_vec(val)
// Release: bincode::serialize(val)
pub fn serialize<T: Serialize>(val: &T) -> Result<Vec<u8>, WireError>
```

### Deserialization

```rust
// Debug: serde_json::from_slice(bytes)
// Release: bincode::deserialize(bytes)
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WireError>
```

---

## VTable Function Pointer Signatures

### PluginAllocated (currently the only supported strategy)

```c
int32_t method(
    const uint8_t* in_ptr,    // serialized input (tuple-encoded arguments)
    uint32_t       in_len,    // input byte count
    uint8_t**      out_ptr,   // [out] pointer to plugin-allocated output
    uint32_t*      out_len    // [out] output byte count
) -> int32_t;                 // status code
```

Rust type:

```rust
unsafe extern "C" fn(*const u8, u32, *mut *mut u8, *mut u32) -> i32
```

Required methods use this type directly. Optional methods use `Option<unsafe extern "C" fn(...)>`.

### Argument Encoding

All method arguments are **tuple-encoded** at the FFI boundary. The input buffer
contains the serialized form of a tuple containing all arguments:

| Arg count | Trait signature | Serialized input type | Example |
|-----------|----------------|----------------------|---------|
| 0 | `fn status(&self) -> String` | `()` | `serialize(&())` |
| 1 | `fn process(&self, input: String) -> String` | `(String,)` | `serialize(&("hello".to_string(),))` |
| 2 | `fn add(&self, a: i64, b: i64) -> i64` | `(i64, i64)` | `serialize(&(3i64, 7i64))` |
| N | `fn foo(&self, a: A, b: B, c: C) -> R` | `(A, B, C)` | `serialize(&(a, b, c))` |

This encoding is uniform — there are no special cases. The `#[plugin_impl]`
macro generates the deserialization code automatically. Host-side callers using
`call_method` must pass the tuple-encoded input:

```rust
// Zero args
let result: String = handle.call_method(0, &()).unwrap();

// One arg
let result: String = handle.call_method(1, &("hello".to_string(),)).unwrap();

// Two args
let result: i64 = handle.call_method(2, &(3i64, 7i64)).unwrap();
```

**Breaking change (v0.0.5):** Prior to 0.0.5, single-argument methods used bare
value encoding (not tuple-wrapped). All methods now use tuple encoding uniformly.

### Free Buffer

```c
void free_buffer(uint8_t* ptr, size_t len);
```

Rust type:

```rust
unsafe extern "C" fn(*mut u8, usize)
```

Called by the host after reading the output buffer. Reconstructs a `Vec<u8>` from `(ptr, len, len)` and drops it.

---

## Status Codes

All `i32`. Returned by vtable function pointers.

| Code | Constant | Meaning |
|------|----------|---------|
| `0` | `STATUS_OK` | Success. Output buffer contains the serialized result. |
| `-1` | `STATUS_BUFFER_TOO_SMALL` | Output buffer too small (CallerAllocated/Arena only). `out_len` contains the required size. |
| `-2` | `STATUS_SERIALIZATION_ERROR` | Serialization or deserialization failed at the FFI boundary. |
| `-3` | `STATUS_PLUGIN_ERROR` | Plugin returned an error. Output buffer contains a serialized `PluginError`. |
| `-4` | `STATUS_PANIC` | Panic caught via `catch_unwind` at the `extern "C"` boundary. |

---

## Load Sequence

The host loads a plugin dylib through the following sequence:

1. **Architecture check** -- Read binary header to verify format (ELF, Mach-O, PE) and architecture (x86_64, aarch64) match the host platform.
2. **dlopen** -- Open the shared library via `libloading::Library::new()`.
3. **dlsym** -- Look up the symbol `fidius_get_registry` (signature: `extern "C" fn() -> *const PluginRegistry`).
4. **Call registry function** -- Invoke `fidius_get_registry()` to obtain the registry pointer.
5. **Validate magic** -- Compare `registry.magic` with `FIDIUS_MAGIC`. Reject on mismatch.
6. **Validate registry version** -- Compare `registry.registry_version` with `REGISTRY_VERSION`. Reject on mismatch.
7. **Iterate descriptors** -- For each of `registry.plugin_count` descriptors:
   - **Validate ABI version** -- Compare `descriptor.abi_version` with `ABI_VERSION`. Reject on mismatch.
   - **Copy strings** -- Read `interface_name` and `plugin_name` as `CStr`, convert to owned `String`.
   - **Copy metadata** -- Build `PluginInfo` from descriptor fields.
8. **Interface validation** (optional) -- If the host has expected values for `interface_hash`, `wire_format`, or `buffer_strategy`, compare each against the plugin's values.
9. **Signature verification** (optional) -- If `require_signature` is set, verify the `.sig` file against trusted Ed25519 public keys.

---

## Interface Hashing Algorithm

FNV-1a 64-bit, used to detect ABI drift at load time.

### Constants

| Name | Value |
|------|-------|
| FNV offset basis | `0xcbf29ce484222325` |
| FNV prime | `0x100000001b3` |

Both `fnv1a` and `interface_hash` are public API, defined in `fidius_core::hash` and re-exported from the `fidius::` facade crate.

### `fnv1a(bytes: &[u8]) -> u64`

```
hash = FNV_OFFSET_BASIS
for each byte in bytes:
    hash = hash XOR byte
    hash = hash * FNV_PRIME  (wrapping multiplication)
return hash
```

This function is `const fn` and can be evaluated at compile time.

### `interface_hash(signatures: &[&str]) -> u64`

1. Copy the input slice and sort lexicographically (ensures order-independence).
2. Join sorted signatures with `"\n"` as separator.
3. Return `fnv1a(combined.as_bytes())`.

### Signature String Format

Each required method produces a canonical signature string:

```
name:arg_type_1,arg_type_2->return_type
```

- The `self` parameter is excluded.
- Types are the `TokenStream::to_string()` representation of the `syn::Type`.
- Optional methods are **excluded** from the interface hash.

### Properties

- **Deterministic:** Same trait definition always produces the same hash.
- **Order-independent:** Method declaration order does not affect the hash (signatures are sorted).
- **Case-sensitive:** `String` and `string` produce different hashes.
- **Optional-method-stable:** Adding optional methods does not change the hash.

---

## Registry Export Symbol

Each plugin cdylib exports a single entry point:

```rust
#[no_mangle]
pub extern "C" fn fidius_get_registry() -> *const PluginRegistry
```

Generated by `fidius_core::fidius_plugin_registry!()`. The registry is built on first call from descriptors collected via `inventory` and cached in a `OnceLock`.

---

## See Also

- [#[plugin_interface] Reference](../api/rust/fidius-macro/interface.md) -- macro that generates vtable and constants
- [#[plugin_impl] Reference](../api/rust/fidius-macro/impl_macro.md) -- macro that generates shims and descriptors
- [Host API Reference](../api/rust/fidius-host.md) -- host-side loading API
- [Errors Reference](./errors.md) -- error types for load and call failures
