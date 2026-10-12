---
title: "fidius-guest::host_ffi"
description: "Host-function (plugin → host callback) FFI types — the stable C ABI contract for the reverse direction (FIDIUS host-callback channel)."
editUrl: false
sidebar:
  order: 16
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Host-function (plugin → host callback) FFI types — the stable C ABI contract for the reverse direction (FIDIUS host-callback channel).

The existing vtable direction lets the **host call the plugin**. This
module defines the mirror: a [`HostFunctionTable`](../../../rust/fidius-guest/host_ffi/#struct.HostFunctionTable) the host hands to a
plugin at bind time, through which **plugin code calls back into the
host** mid-execution. Arguments and returns are bincode-serialized with
the same wire conventions as the host → plugin direction (args as a
tuple, returns as the bare value, errors as [`PluginError`](../../../rust/fidius-guest/error/#struct.PluginError)).

**Shape**

One `#[repr(C)]` table per host interface, containing a single indexed
`dispatch` entry point plus identity fields (interface name, FNV-1a hash
of the method signatures, and a user-declared version). The plugin
advertises which host interfaces it wants via [`HostImportDescriptor`](../../../rust/fidius-guest/host_ffi/#struct.HostImportDescriptor)s
collected behind the optional `fidius_get_host_imports` export; the host
validates version + hash and installs the table through the descriptor's
`bind` function. Both sides check: the host refuses to bind a mismatched
import (`LoadError`-level failure), and the plugin-side `bind` shim
re-validates the table before storing it — a mismatch can therefore never
reach `dispatch`, where positional bincode would corrupt arguments.

**Memory ownership**

Output buffers are allocated by the **host** (the callee) as `Box<[u8]>`
and freed by the host via the table's `free_buffer` — the plugin calls it
after copying the bytes out, so allocations never cross allocator
boundaries. This mirrors (in reverse) the `PluginAllocated` strategy of
the forward direction.

**Threading and reentrancy (the contract)**

See the `#[fidius::host_interface]` macro docs for the full contract.
The load-bearing rules:

- Host functions are **synchronous** at the boundary. A host
  implementation that needs to await must bridge to its own runtime
  internally (e.g. `tokio::runtime::Handle::block_on`) and must tolerate
  being called from threads it does not own — including plugin-spawned
  threads and, most importantly, the host's own thread **while a
  host → plugin call is live on that stack** (host → plugin → host).
- The host must **not hold any lock across a plugin call** that a host
  function implementation could try to acquire — the callback re-enters
  the host while the plugin call is still on the stack, and that lock
  acquisition will deadlock.
- Host implementations must be `Send + Sync`: multiple plugin threads
  may call concurrently.

[`host_callback_depth`](../../../rust/fidius-guest/host_ffi/#fn.host_callback_depth) exposes a per-thread depth counter maintained by
the generated dispatch shims so host implementations can *detect*
reentrancy (e.g. `debug_assert!` invariants about which locks may be
taken at depth > 0).

## Structs

### <span id="struct.HostFunctionTable"></span><code style="font-size: 0.85em">struct HostFunctionTable</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

A set of host functions offered to a plugin, as a C-ABI table.

Built by the host side of a `#[fidius::host_interface]` (the generated
`<Trait>Binding`) and installed into the plugin through the plugin's
[`HostImportDescriptor::bind`](../../../rust/fidius-guest/host_ffi/#HostTableCell.bind). The table and everything it references
must live for the remainder of the process (the generated binding leaks
one table + one `Arc<dyn Trait>` per bind — a few dozen bytes, once per
loaded library).

**Safety**

- `interface_name` points to a static, null-terminated UTF-8 C string.
- `dispatch` and `free_buffer` are valid for the process lifetime.
- `ctx` is an opaque pointer owned by the table; the plugin passes it to
  every `dispatch` call and never dereferences or frees it.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `table_size` | `u32` | Size in bytes of this struct at host build time. Read first; any field at an offset >= `table_size` is absent in the host's build (post-1.0 forward-compat, mirroring `PluginDescriptor`). |
| `abi_version` | `u32` | Must equal [`crate::descriptor::ABI_VERSION`](../../../rust/fidius-guest/descriptor/#const.ABI_VERSION). |
| `interface_name` | `*const c_char` | Null-terminated name of the host interface trait (e.g. `"CloacinaHost"`). |
| `interface_hash` | `u64` | FNV-1a hash of the host interface's method signatures. |
| `interface_version` | `u32` | User-declared version from `#[host_interface(version = N)]`. |
| `fn_count` | `u32` | Number of functions dispatchable through this table. |
| `ctx` | `*mut c_void` | Opaque host context, passed as the first argument to every `dispatch`. |
| `dispatch` | `HostDispatchFn` | The single indexed dispatch entry point. |
| `free_buffer` | `unsafe extern "C" fn(*mut u8, usize)` | Frees a buffer previously returned through `dispatch`'s out params. Host-allocated memory is host-freed — never mixed across allocators. |

### <span id="struct.HostImportDescriptor"></span><code style="font-size: 0.85em">struct HostImportDescriptor</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

A plugin's declaration that it can consume a host interface.

Generated (one per `#[host_interface]` trait) into the interface crate and
collected behind the plugin's `fidius_get_host_imports` export. The host
reads the identity fields to gate version/hash **before** calling `bind`.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `descriptor_size` | `u32` | Size in bytes of this struct at plugin build time (forward-compat). |
| `abi_version` | `u32` | Must equal [`crate::descriptor::ABI_VERSION`](../../../rust/fidius-guest/descriptor/#const.ABI_VERSION). |
| `interface_name` | `*const c_char` | Null-terminated name of the host interface trait. |
| `interface_hash` | `u64` | FNV-1a hash of the method signatures the plugin was built against. |
| `interface_version` | `u32` | The `#[host_interface(version = N)]` the plugin was built against. |
| `bind` | `unsafe extern "C" fn(*const HostFunctionTable) -> i32` | Installs a validated [`HostFunctionTable`](../../../rust/fidius-guest/host_ffi/#struct.HostFunctionTable). Returns a `BIND_*` status. The shim re-validates hash/version/ABI defensively before storing. |

### <span id="struct.HostImportRegistry"></span><code style="font-size: 0.85em">struct HostImportRegistry</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

Registry of all host-import declarations in a plugin dylib, returned by the optional `fidius_get_host_imports` export.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `magic` | `[u8; 8]` | Must equal [`crate::descriptor::FIDIUS_MAGIC`](../../../rust/fidius-guest/descriptor/#const.FIDIUS_MAGIC). |
| `registry_version` | `u32` | Must equal [`HOST_IMPORTS_VERSION`](../../../rust/fidius-guest/host_ffi/#const.HOST_IMPORTS_VERSION). |
| `import_count` | `u32` | Number of import descriptors. |
| `imports` | `*const *const HostImportDescriptor` | Pointer to an array of `import_count` descriptor pointers. |

### <span id="struct.HostImportPtr"></span><code style="font-size: 0.85em">struct HostImportPtr</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

A `Sync` wrapper for a raw `HostImportDescriptor` pointer, for use in `static` arrays (mirrors `descriptor::DescriptorPtr`).

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `0` | `*const HostImportDescriptor` |  |

### <span id="struct.HostTableCell"></span><code style="font-size: 0.85em">struct HostTableCell</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

The once-only cell a plugin stores its bound [`HostFunctionTable`](../../../rust/fidius-guest/host_ffi/#struct.HostFunctionTable) in.

One static cell per `#[host_interface]` trait is generated into the
interface crate. `bind` installs a table exactly once (subsequent binds
report [`BIND_ERR_ALREADY_BOUND`](../../../rust/fidius-guest/host_ffi/#const.BIND_ERR_ALREADY_BOUND) rather than swapping a table out from
under in-flight calls); `get` is a relaxed-cost atomic load on the call
path.

#### Methods

##### <span id="HostTableCell.new"></span><code>new</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>
 <span class="plissken-badge plissken-badge-const" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-blue-low); color: var(--sl-color-blue-high);">const</span>

```rust
const fn new() -> Self
```

An empty (unbound) cell, for `static` initialization.

<details>
<summary>Source</summary>

```rust
    pub const fn new() -> Self {
        Self(AtomicPtr::new(std::ptr::null_mut()))
    }
```

</details>

##### <span id="HostTableCell.bind"></span><code>bind</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>
 <span class="plissken-badge plissken-badge-unsafe" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-red-low); color: var(--sl-color-red-high);">unsafe</span>

```rust
unsafe fn bind(&self, table: *const HostFunctionTable) -> i32
```

Install `table` if the cell is empty. Returns [`BIND_OK`](../../../rust/fidius-guest/host_ffi/#const.BIND_OK) on success or [`BIND_ERR_ALREADY_BOUND`](../../../rust/fidius-guest/host_ffi/#const.BIND_ERR_ALREADY_BOUND) if a table was already installed.

**Safety**

`table` must be non-null and valid for the remainder of the process.

<details>
<summary>Source</summary>

```rust
    pub unsafe fn bind(&self, table: *const HostFunctionTable) -> i32 {
        match self.0.compare_exchange(
            std::ptr::null_mut(),
            table as *mut HostFunctionTable,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => BIND_OK,
            Err(_) => BIND_ERR_ALREADY_BOUND,
        }
    }
```

</details>

##### <span id="HostTableCell.get"></span><code>get</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn get(&self) -> Option<&HostFunctionTable>
```

The bound table, or `None` if the host never bound this interface.

<details>
<summary>Source</summary>

```rust
    pub fn get(&self) -> Option<&HostFunctionTable> {
        let ptr = self.0.load(Ordering::Acquire);
        // SAFETY: a non-null pointer was installed by `bind`, whose contract
        // requires process-lifetime validity.
        unsafe { ptr.cast_const().as_ref() }
    }
```

</details>

### <span id="struct.HostBindError"></span><code style="font-size: 0.85em">struct HostBindError</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

**Derives:** `Debug`, `thiserror::Error`

Error installing a [`HostFunctionTable`](../../../rust/fidius-guest/host_ffi/#struct.HostFunctionTable) into a plugin — the typed form of a non-`BIND_OK` status from a plugin's bind shim.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `interface` | `&'static str` | The host interface (trait) name. |
| `code` | `i32` | The raw `BIND_*` status code. |
| `message` | `&'static str` | Human-readable description of the status. |

### <span id="struct.CallbackDepthGuard"></span><code style="font-size: 0.85em">struct CallbackDepthGuard</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

RAII guard incrementing [`host_callback_depth`](../../../rust/fidius-guest/host_ffi/#fn.host_callback_depth) — used by the generated host-side dispatch shims. Public so macro-generated code can name it; not intended for direct use.

#### Methods

##### <span id="CallbackDepthGuard.new"></span><code>new</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn new() -> Self
```

<details>
<summary>Source</summary>

```rust
    pub fn new() -> Self {
        CALLBACK_DEPTH.with(|d| d.set(d.get() + 1));
        Self(())
    }
```

</details>

## Enums

### <span id="enum.HostCallError"></span><code style="font-size: 0.85em">enum HostCallError</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

Error returned to plugin code when a host-function call fails.

This is the plugin-side error surface of the callback channel — every
failure mode (unbound interface, host-raised typed error, host panic,
serialization fault) arrives as a variant here, never as a panic and
never as unwinding across the FFI boundary.

#### Variants

- **`NotBound`** - The host never bound this interface for this plugin. Either the host
  application doesn't provide it, or it skipped binding after load.
- **`Host`** - The host function returned a typed error ([`PluginError`](../../../rust/fidius-guest/error/#struct.PluginError) is the
  shared error currency in both directions).
- **`HostPanic`** - The host function panicked; the panic was caught at the boundary.
- **`Serialization`** - Argument serialization failed on the plugin side, or the host
  reported it could not decode the arguments / encode the result.
- **`Deserialization`** - The host's response bytes did not decode as the expected type.
- **`InvalidIndex`** - The dispatch index was out of range for the host's table. Cannot
  happen through generated clients when the bind-time gate passed.
- **`UnknownStatus`** - The host returned a status code this plugin doesn't know.
- **`VersionMismatch`** - The host provides a different **version** of this interface than the
  plugin was built against. WASM host-call channel only — on the dylib
  path the same mismatch fails the host's bind at load, so plugin code
  sees [`HostCallError::NotBound`](../../../rust/fidius-guest/host_ffi/#enum.HostCallError) instead.
- **`HashMismatch`** - The host provides a different **signature set** (hash) of this
  interface than the plugin was built against. WASM host-call channel
  only; dylib mismatches fail the host's bind at load.

## Functions

### <span id="fn.decode_host_call_status"></span><code style="font-size: 0.85em">fn decode_​host_​call_​status</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn decode_host_call_status(
    interface: &'static str,
    index: u32,
    status: i32,
    payload: Vec<u8>,
) -> Result<Vec<u8>, HostCallError>
```

Decode a `(status, payload)` pair from the WASM `fidius:host-call` import into the plugin-side result. Compiles natively too so the mapping is unit-testable without a component build.

<details>
<summary>Source</summary>

```rust
pub fn decode_host_call_status(
    interface: &'static str,
    index: u32,
    status: i32,
    payload: Vec<u8>,
) -> Result<Vec<u8>, HostCallError> {
    use crate::status::*;
    match status {
        STATUS_OK => Ok(payload),
        STATUS_PLUGIN_ERROR => {
            let err: PluginError = crate::wire::deserialize(&payload).unwrap_or_else(|_| {
                PluginError::new(
                    "UNKNOWN",
                    "host returned an error but no decodable error data",
                )
            });
            Err(HostCallError::Host(err))
        }
        STATUS_PANIC => {
            let msg = if payload.is_empty() {
                "unknown panic".to_string()
            } else {
                crate::wire::deserialize::<String>(&payload)
                    .unwrap_or_else(|_| "unknown panic".into())
            };
            Err(HostCallError::HostPanic(msg))
        }
        STATUS_SERIALIZATION_ERROR => Err(HostCallError::Serialization(
            "host could not decode arguments or encode the result".into(),
        )),
        STATUS_INVALID_INDEX => Err(HostCallError::InvalidIndex { index }),
        HOST_CALL_STATUS_NOT_BOUND => Err(HostCallError::NotBound { interface }),
        HOST_CALL_STATUS_VERSION_MISMATCH => {
            let (plugin_expects, host_provides): (u32, u32) =
                crate::wire::deserialize(&payload).unwrap_or((0, 0));
            Err(HostCallError::VersionMismatch {
                interface,
                plugin_expects,
                host_provides,
            })
        }
        HOST_CALL_STATUS_HASH_MISMATCH => {
            let (plugin_expects, host_provides): (u64, u64) =
                crate::wire::deserialize(&payload).unwrap_or((0, 0));
            Err(HostCallError::HashMismatch {
                interface,
                plugin_expects,
                host_provides,
            })
        }
        code => Err(HostCallError::UnknownStatus { code }),
    }
}
```

</details>

### <span id="fn.bind_status_to_result"></span><code style="font-size: 0.85em">fn bind_​status_​to_​result</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn bind_status_to_result(
    interface: &'static str,
    code: i32,
) -> Result<(), HostBindError>
```

Map a `BIND_*` status to a `Result` for host-side bind entry points.

<details>
<summary>Source</summary>

```rust
pub fn bind_status_to_result(interface: &'static str, code: i32) -> Result<(), HostBindError> {
    if code == BIND_OK {
        Ok(())
    } else {
        Err(HostBindError {
            interface,
            code,
            message: bind_status_message(code),
        })
    }
}
```

</details>

### <span id="fn.bind_status_message"></span><code style="font-size: 0.85em">fn bind_​status_​message</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn bind_status_message(code: i32) -> &'static str
```

Map a `BIND_*` status to a human-readable description (for error paths).

<details>
<summary>Source</summary>

```rust
pub fn bind_status_message(code: i32) -> &'static str {
    match code {
        BIND_OK => "ok",
        BIND_ERR_NULL => "null table pointer",
        BIND_ERR_ALREADY_BOUND => "a host table is already bound for this interface",
        BIND_ERR_HASH_MISMATCH => "interface hash mismatch between plugin and host table",
        BIND_ERR_VERSION_MISMATCH => "interface version mismatch between plugin and host table",
        BIND_ERR_ABI => "fidius ABI version mismatch",
        BIND_ERR_FN_COUNT => "host table fn_count does not match the interface",
        BIND_ERR_PANIC => "plugin bind shim panicked",
        _ => "unknown bind status",
    }
}
```

</details>

### <span id="fn.host_callback_depth"></span><code style="font-size: 0.85em">fn host_​callback_​depth</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn host_callback_depth() -> usize
```

Per-thread depth of live host-function dispatches.

Incremented by the generated dispatch shim for the duration of each host
function call. `0` means "not inside a host callback"; `1` is the normal
depth while a host function runs; `> 1` means a host function (directly
or via a nested plugin call) triggered another host function on the same
thread. Host implementations can use this to assert lock-discipline
invariants — e.g. `debug_assert!(host_callback_depth() <= 1)` in code
that must never be reached reentrantly.

<details>
<summary>Source</summary>

```rust
pub fn host_callback_depth() -> usize {
    CALLBACK_DEPTH.with(|d| d.get())
}
```

</details>

### <span id="fn.call_host_fn"></span><code style="font-size: 0.85em">fn call_​host_​fn</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn call_host_fn(
    table: &HostFunctionTable,
    index: u32,
    input: &[u8],
) -> Result<Vec<u8>, HostCallError>
```

Invoke a host function through a bound table: bincode-of-args in, bincode-of-result out, with the status protocol mapped to [`HostCallError`](../../../rust/fidius-guest/host_ffi/#enum.HostCallError). Shared by every generated `<Trait>Client` method.

On `STATUS_OK` the host's output buffer is copied into a `Vec<u8>` and
released via the table's `free_buffer`; error payloads are decoded and
released the same way. Never panics on malformed host output — decoding
failures surface as [`HostCallError::Deserialization`](../../../rust/fidius-guest/host_ffi/#enum.HostCallError).

<details>
<summary>Source</summary>

```rust
pub fn call_host_fn(
    table: &HostFunctionTable,
    index: u32,
    input: &[u8],
) -> Result<Vec<u8>, HostCallError> {
    use crate::status::*;

    let mut out_ptr: *mut u8 = std::ptr::null_mut();
    let mut out_len: u32 = 0;

    // SAFETY: `table` was validated at bind time; dispatch/free_buffer are
    // process-lifetime function pointers per the HostFunctionTable contract.
    let status = unsafe {
        (table.dispatch)(
            table.ctx,
            index,
            input.as_ptr(),
            input.len() as u32,
            &mut out_ptr,
            &mut out_len,
        )
    };

    // Copy-then-free helper for host-owned output buffers.
    let take_output = |out_ptr: *mut u8, out_len: u32| -> Vec<u8> {
        if out_ptr.is_null() || out_len == 0 {
            return Vec::new();
        }
        // SAFETY: the host wrote a valid buffer of out_len bytes; we copy it
        // out and hand it straight back to the host's free_buffer.
        unsafe {
            let bytes = std::slice::from_raw_parts(out_ptr, out_len as usize).to_vec();
            (table.free_buffer)(out_ptr, out_len as usize);
            bytes
        }
    };

    match status {
        STATUS_OK => Ok(take_output(out_ptr, out_len)),
        STATUS_PLUGIN_ERROR => {
            let bytes = take_output(out_ptr, out_len);
            let err: PluginError = crate::wire::deserialize(&bytes).unwrap_or_else(|_| {
                PluginError::new(
                    "UNKNOWN",
                    "host returned an error but no decodable error data",
                )
            });
            Err(HostCallError::Host(err))
        }
        STATUS_PANIC => {
            let bytes = take_output(out_ptr, out_len);
            let msg = if bytes.is_empty() {
                "unknown panic".to_string()
            } else {
                crate::wire::deserialize::<String>(&bytes)
                    .unwrap_or_else(|_| "unknown panic".into())
            };
            Err(HostCallError::HostPanic(msg))
        }
        STATUS_SERIALIZATION_ERROR => Err(HostCallError::Serialization(
            "host could not decode arguments or encode the result".into(),
        )),
        STATUS_INVALID_INDEX => Err(HostCallError::InvalidIndex { index }),
        code => Err(HostCallError::UnknownStatus { code }),
    }
}
```

</details>

## Type aliases

| Name | Type | Description |
|------|------|-------------|
| <span id="type.HostDispatchFn"></span>`HostDispatchFn` | `unsafe extern "C" fn(*mut c_void, u32, *const u8, u32, *mut *mut u8, *mut u32) -> i32` | The dispatch signature of a host-function table: `(ctx, fn_index, in_ptr, in_len, out_ptr, out_len) -> status`. Input is the bincode of the argument tuple; on `STATUS_OK`/`STATUS_PLUGIN_ERROR`/`STATUS_PANIC` the host writes a host-owned buffer to `out_ptr`/`out_len` which the caller must release via `HostFunctionTable::free_buffer`. |

## Constants

| Name | Type | Value | Description |
|------|------|-------|-------------|
| <span id="const.HOST_IMPORTS_VERSION"></span>`HOST_IMPORTS_VERSION` | `u32` | `1` | Current layout version of the [`HostImportRegistry`](../../../rust/fidius-guest/host_ffi/#struct.HostImportRegistry) struct. |
| <span id="const.HOST_IMPORTS_SYMBOL"></span>`HOST_IMPORTS_SYMBOL` | `&str` | `"fidius_get_host_imports"` | Name of the optional symbol a plugin dylib exports to advertise the host interfaces it can consume. Emitted by `fidius_plugin_registry!()`; absent from plugins built before the host-callback channel existed — the host treats absence as "no host imports". |
| <span id="const.BIND_OK"></span>`BIND_OK` | `i32` | `0` | The table was validated and installed. |
| <span id="const.BIND_ERR_NULL"></span>`BIND_ERR_NULL` | `i32` | `-1` | The table pointer was null. |
| <span id="const.BIND_ERR_ALREADY_BOUND"></span>`BIND_ERR_ALREADY_BOUND` | `i32` | `-2` | A table for this interface was already installed (bind is once-only). |
| <span id="const.BIND_ERR_HASH_MISMATCH"></span>`BIND_ERR_HASH_MISMATCH` | `i32` | `-3` | The table's `interface_hash` does not match what the plugin was built against — the signatures drifted. |
| <span id="const.BIND_ERR_VERSION_MISMATCH"></span>`BIND_ERR_VERSION_MISMATCH` | `i32` | `-4` | The table's `interface_version` does not match what the plugin expects. |
| <span id="const.BIND_ERR_ABI"></span>`BIND_ERR_ABI` | `i32` | `-5` | The table was built by an incompatible fidius ABI. |
| <span id="const.BIND_ERR_FN_COUNT"></span>`BIND_ERR_FN_COUNT` | `i32` | `-6` | The table's `fn_count` does not match the interface's method count. |
| <span id="const.BIND_ERR_PANIC"></span>`BIND_ERR_PANIC` | `i32` | `-7` | The bind shim panicked (never expected; defensive). |
| <span id="const.BIND_ERR_WRONG_BACKEND"></span>`BIND_ERR_WRONG_BACKEND` | `i32` | `-8` | The handle's backend cannot accept this table (e.g. a wasm-only bind entry point called on a cdylib/Python-backed handle, or vice versa). |
| <span id="const.HOST_CALL_PROBE_INDEX"></span>`HOST_CALL_PROBE_INDEX` | `u32` | `u32::MAX` | Reserved dispatch index for the bind-probe: `call(..., PROBE, [])` runs the name/version/hash gate and returns `STATUS_OK` with an empty payload without dispatching a real function. Generated wasm clients use it to implement `bound()` / `is_bound()`. |
| <span id="const.HOST_CALL_STATUS_NOT_BOUND"></span>`HOST_CALL_STATUS_NOT_BOUND` | `i32` | `-7` | host-call status: no host interface with this name is bound. |
| <span id="const.HOST_CALL_STATUS_VERSION_MISMATCH"></span>`HOST_CALL_STATUS_VERSION_MISMATCH` | `i32` | `-8` | host-call status: the bound table's version differs from what the plugin was built against. Payload = bincode `(plugin_expects: u32, host_provides: u32)`. |
| <span id="const.HOST_CALL_STATUS_HASH_MISMATCH"></span>`HOST_CALL_STATUS_HASH_MISMATCH` | `i32` | `-9` | host-call status: the bound table's signature hash differs. Payload = bincode `(plugin_expects: u64, host_provides: u64)`. |
