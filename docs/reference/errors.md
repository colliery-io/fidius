<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# Errors Reference

All error types in the Fidius plugin framework, with causes and resolutions.

**Source:** `fidius-host/src/error.rs`, `fidius-core/src/wire.rs`, `fidius-core/src/error.rs`

---

## LoadError

Errors that can occur when loading a plugin. Defined in `fidius_host::error`.

Derives `Debug` and `thiserror::Error`.

```rust
pub enum LoadError {
    LibraryNotFound { path: String },
    SymbolNotFound { path: String },
    InvalidMagic,
    IncompatibleRegistryVersion { got: u32, expected: u32 },
    IncompatibleAbiVersion { got: u32, expected: u32 },
    InterfaceHashMismatch { got: u64, expected: u64 },
    WireFormatMismatch { got: u8, expected: u8 },
    BufferStrategyMismatch { got: u8, expected: u8 },
    ArchitectureMismatch { expected: String, got: String },
    SignatureInvalid { path: String },
    SignatureRequired { path: String },
    PluginNotFound { name: String },
    LibLoading(libloading::Error),
    Io(std::io::Error),
}
```

### Variant Details

#### `LibraryNotFound`

```
library not found: {path}
```

| | |
|---|---|
| **Trigger** | `dlopen` fails with an error message containing "No such file" or "not found". |
| **Fields** | `path: String` -- display path of the dylib. |
| **Resolution** | Verify the dylib exists at the specified path. Check search paths configured on `PluginHostBuilder`. |

#### `SymbolNotFound`

```
symbol 'fidius_get_registry' not found in {path}
```

| | |
|---|---|
| **Trigger** | `dlsym("fidius_get_registry")` fails. The library was opened but does not export the expected symbol. |
| **Fields** | `path: String` -- display path of the dylib. |
| **Resolution** | Ensure the plugin crate calls `fidius_core::fidius_plugin_registry!()` in its `lib.rs` and is compiled as a `cdylib`. |

#### `InvalidMagic`

```
invalid magic bytes (expected FIDIUS\0\0)
```

| | |
|---|---|
| **Trigger** | The `magic` field of the `PluginRegistry` does not equal `b"FIDIUS\0\0"`. |
| **Fields** | None. |
| **Resolution** | The dylib is not a Fidius plugin, or its registry is corrupt. Rebuild the plugin. |

#### `IncompatibleRegistryVersion`

```
incompatible registry version: got {got}, expected {expected}
```

| | |
|---|---|
| **Trigger** | `registry.registry_version != REGISTRY_VERSION`. The plugin was built against a different `fidius-core` version with a different registry layout. |
| **Fields** | `got: u32`, `expected: u32`. |
| **Resolution** | Rebuild the plugin against the same `fidius-core` version as the host. |

#### `IncompatibleAbiVersion`

```
incompatible ABI version: got {got}, expected {expected}
```

| | |
|---|---|
| **Trigger** | `descriptor.abi_version != ABI_VERSION`. The plugin's descriptor layout does not match the host's expectation. |
| **Fields** | `got: u32`, `expected: u32`. |
| **Resolution** | Rebuild the plugin against the same `fidius-core` version as the host. |

#### `InterfaceHashMismatch`

```
interface hash mismatch: got {got:#x}, expected {expected:#x}
```

| | |
|---|---|
| **Trigger** | The plugin's `interface_hash` does not match the expected hash set on `PluginHostBuilder::interface_hash()`. The plugin was compiled against a different version of the interface trait (method signatures changed). |
| **Fields** | `got: u64`, `expected: u64` (displayed as hex). |
| **Resolution** | Rebuild the plugin against the current version of the interface crate. Only changes to required method signatures affect the hash. |

#### `WireFormatMismatch`

```
wire format mismatch: got {got}, expected {expected}
```

| | |
|---|---|
| **Trigger** | The plugin's `wire_format` does not match the expected format. Typically caused by mixing debug and release builds. |
| **Fields** | `got: u8`, `expected: u8`. Values: `0` = Json, `1` = Bincode. |
| **Resolution** | Compile both host and plugin in the same mode (both debug or both release). |

#### `BufferStrategyMismatch`

```
buffer strategy mismatch: got {got}, expected {expected}
```

| | |
|---|---|
| **Trigger** | The plugin's `buffer_strategy` does not match the expected strategy. |
| **Fields** | `got: u8`, `expected: u8`. Values: `0` = CallerAllocated, `1` = PluginAllocated, `2` = Arena. |
| **Resolution** | Ensure the plugin implements the same interface with the same `buffer` attribute. |

#### `ArchitectureMismatch`

```
architecture mismatch: expected {expected}, got {got}
```

| | |
|---|---|
| **Trigger** | The dylib binary format or CPU architecture does not match the host. Detected by reading the binary header before `dlopen`. |
| **Fields** | `expected: String`, `got: String`. |
| **Resolution** | Cross-compile the plugin for the host's target architecture. |

#### `SignatureInvalid`

```
signature verification failed for {path}
```

| | |
|---|---|
| **Trigger** | The `.sig` file exists but the Ed25519 signature does not verify against any trusted key. The dylib may have been tampered with, or was signed with an untrusted key. |
| **Fields** | `path: String`. |
| **Resolution** | Re-sign the plugin with a trusted key, or add the signing key to the host's trusted keys. |

#### `SignatureRequired`

```
signature required but no .sig file found for {path}
```

| | |
|---|---|
| **Trigger** | `require_signature` is `true` on the host, but no `.sig` file was found adjacent to the dylib. |
| **Fields** | `path: String`. |
| **Resolution** | Sign the plugin with `fidius sign --key <secret_key> <dylib>`. |

#### `PluginNotFound`

```
plugin '{name}' not found
```

| | |
|---|---|
| **Trigger** | `PluginHost::load(name)` searched all configured paths and found no plugin with the given name. |
| **Fields** | `name: String`. |
| **Resolution** | Verify the plugin name matches the impl type name used with `#[plugin_impl]`. Check that the dylib is in one of the configured search paths. |

#### `LibLoading`

```
libloading error: {0}
```

| | |
|---|---|
| **Trigger** | A `libloading::Error` that does not match the "not found" pattern. May indicate permission issues, missing dependencies, or corrupt binaries. |
| **Fields** | The inner `libloading::Error`. |
| **Resolution** | Check file permissions, system library dependencies, and binary integrity. |

#### `Io`

```
io error: {0}
```

| | |
|---|---|
| **Trigger** | An `std::io::Error` from filesystem operations (reading directories, reading signature files). |
| **Fields** | The inner `std::io::Error`. |
| **Resolution** | Check filesystem permissions and that search path directories exist. |

---

## CallError

Errors that can occur when calling a plugin method via `PluginHandle::call_method`. Defined in `fidius_host::error`.

Derives `Debug` and `thiserror::Error`.

```rust
pub enum CallError {
    Serialization(String),
    Deserialization(String),
    Plugin(PluginError),
    Panic,
    BufferTooSmall,
    NotImplemented { bit: u32 },
}
```

### Variant Details

#### `Serialization`

```
serialization error: {0}
```

| | |
|---|---|
| **Trigger** | Input serialization failed (before FFI call), or the plugin returned `STATUS_SERIALIZATION_ERROR` (`-2`), or an unknown status code was received. |
| **Fields** | `String` -- error description. |
| **Resolution** | Ensure the input type implements `Serialize` correctly and matches the type expected by the plugin method. |

#### `Deserialization`

```
deserialization error: {0}
```

| | |
|---|---|
| **Trigger** | Output deserialization failed after a successful FFI call (`STATUS_OK`), or `PluginError` deserialization failed when handling `STATUS_PLUGIN_ERROR`. |
| **Fields** | `String` -- error description. |
| **Resolution** | Ensure the output type parameter `O` matches the type the plugin method actually returns. Verify host and plugin use the same wire format. |

#### `Plugin`

```
plugin error: {0}
```

| | |
|---|---|
| **Trigger** | The plugin method returned `STATUS_PLUGIN_ERROR` (`-3`). The output buffer contained a serialized `PluginError` which was successfully deserialized. |
| **Fields** | `PluginError` -- contains `code`, `message`, and optional `details`. |
| **Resolution** | Application-specific. Inspect `PluginError::code` and `PluginError::message` for the plugin's error details. |

#### `Panic`

```
plugin panicked during method call
```

| | |
|---|---|
| **Trigger** | The plugin returned `STATUS_PANIC` (`-4`). A panic occurred inside the plugin method and was caught by `catch_unwind` in the FFI shim. |
| **Fields** | None. |
| **Resolution** | Debug the plugin. The panic message is not reliably available across FFI. |

#### `BufferTooSmall`

```
buffer too small
```

| | |
|---|---|
| **Trigger** | The plugin returned `STATUS_BUFFER_TOO_SMALL` (`-1`). Only relevant for `CallerAllocated` and `Arena` buffer strategies (not currently used with `PluginAllocated`). |
| **Fields** | None. |
| **Resolution** | Retry with a larger buffer (for CallerAllocated/Arena strategies). |

#### `NotImplemented`

```
method not implemented (capability bit {bit} not set)
```

| | |
|---|---|
| **Trigger** | An optional method was called but the plugin does not implement it (the corresponding capability bit is not set). This variant is available for host-side guard code; the vtable would contain `None` for unimplemented optional methods. |
| **Fields** | `bit: u32` -- the capability bit index. |
| **Resolution** | Check `PluginHandle::has_capability(bit)` before calling optional methods. Use a plugin that implements the required optional method. |

---

## WireError

Errors from wire format serialization/deserialization. Defined in `fidius_core::wire`.

Derives `Debug` and `thiserror::Error`.

```rust
pub enum WireError {
    Json(serde_json::Error),
    Bincode(bincode::Error),
}
```

### Variant Details

#### `Json`

```
json wire error: {0}
```

| | |
|---|---|
| **Trigger** | `serde_json` serialization or deserialization failed. Active in debug builds. |
| **Fields** | Inner `serde_json::Error`. |

#### `Bincode`

```
bincode wire error: {0}
```

| | |
|---|---|
| **Trigger** | `bincode` serialization or deserialization failed. Active in release builds. |
| **Fields** | Inner `bincode::Error`. |

---

## PluginError

Business logic error returned by plugin method implementations. Defined in `fidius_core::error`. Serialized across the FFI boundary.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}
```

| Field | Description |
|-------|-------------|
| `code` | Machine-readable error code (e.g., `"INVALID_INPUT"`, `"NOT_FOUND"`). |
| `message` | Human-readable error message. |
| `details` | Optional structured details as a JSON string. |

### Display Format

```
[{code}] {message}
```

### Constructors

```rust
pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self
```

Create a `PluginError` without details.

```rust
pub fn with_details(
    code: impl Into<String>,
    message: impl Into<String>,
    details: serde_json::Value,
) -> Self
```

Create a `PluginError` with structured details. The `serde_json::Value` is serialized to a JSON string for storage (ensuring it serializes correctly under both JSON and bincode wire formats).

### Accessors

```rust
pub fn details_value(&self) -> Option<serde_json::Value>
```

Parse the `details` field back into a `serde_json::Value`. Returns `None` if details is absent or fails to parse.

---

## See Also

- [Host API Reference](./host-api.md) -- where these errors are returned
- [ABI Specification](./abi-specification.md) -- status codes that map to `CallError` variants
- [#[plugin_impl] Reference](./plugin-impl-macro.md) -- shim code that produces status codes
