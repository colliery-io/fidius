---
title: "fidius-host::executor::cdylib"
description: "`CdylibExecutor` — the cdylib execution backend: vtable/FFI dispatch with the bincode wire format."
editUrl: false
sidebar:
  order: 33
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

`CdylibExecutor` — the cdylib execution backend: vtable/FFI dispatch with the bincode wire format.

This is the original `PluginHandle` dispatch logic, moved behind the
[`crate::executor::PluginExecutor`](../../../../rust/fidius-host/executor/#trait.PluginExecutor) seam (FIDIUS-I-0021). It keeps its own
generic `call_method<I, O>` so the cdylib typed path serializes the concrete
type with bincode **directly** — `Value` is never involved, so the bytes the
plugin decodes are byte-identical to pre-refactor. The public-facing
[`crate::handle::PluginHandle`](../../../../rust/fidius-host/handle/#struct.PluginHandle) wraps this in an enum alongside the Python
(and future WASM) backends.

## Structs

### <span id="struct.CdylibExecutor"></span><code style="font-size: 0.85em">struct CdylibExecutor</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

A handle to a loaded plugin, ready for calling methods.

Holds an `Arc<Library>` to keep the dylib loaded as long as any handle exists.
Call methods via `call_method()` which handles serialization, FFI, and cleanup.

`CdylibExecutor` is `Send + Sync`. Plugin methods take `&self` (enforced by
the macro), so concurrent calls from multiple threads are safe as long as
the plugin implementation is thread-safe internally.

#### Methods

##### <span id="CdylibExecutor.from_loaded"></span><code>from_​loaded</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn from_loaded(plugin: crate::loader::LoadedPlugin) -> Self
```

Create a CdylibExecutor from a LoadedPlugin.

<details>
<summary>Source</summary>

```rust
    pub fn from_loaded(plugin: crate::loader::LoadedPlugin) -> Self {
        Self::from_loaded_with_config(plugin, &[])
    }
```

</details>

##### <span id="CdylibExecutor.from_loaded_with_config"></span><code>from_​loaded_​with_​config</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn from_loaded_with_config(
    plugin: crate::loader::LoadedPlugin,
    cfg: &[u8],
) -> Self
```

Like `Self::from_loaded` but binds serialized `cfg` config bytes at construction — the DYNAMIC *configured* path (FIDIUS-A-0006 / CI.2). A `#[plugin_impl(Trait, config = C)]` loaded from a dylib whose `configure` constructor receives the bound config once; methods then close over it without re-passing. `cfg` is bincode of the plugin's config type; empty (`&[]`) reproduces `Self::from_loaded`'s singleton construction. This is the dynamic-load analogue of `Self::from_descriptor_with_config`.

<details>
<summary>Source</summary>

```rust
    pub fn from_loaded_with_config(plugin: crate::loader::LoadedPlugin, cfg: &[u8]) -> Self {
        let instance = unsafe { construct_instance(plugin.descriptor, cfg) };
        let destroy = unsafe { (*plugin.descriptor).destroy };
        Self {
            _library: Some(plugin.library),
            vtable: plugin.vtable,
            descriptor: plugin.descriptor,
            free_buffer: plugin.free_buffer,
            capabilities: plugin.info.capabilities,
            method_count: plugin.method_count,
            info: plugin.info,
            instance,
            destroy,
        }
    }
```

</details>

##### <span id="CdylibExecutor.from_descriptor"></span><code>from_​descriptor</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn from_descriptor(
    desc: &'static PluginDescriptor,
) -> Result<Self, LoadError>
```

Create a CdylibExecutor from a plugin descriptor already registered in the current process's inventory (via a `#[plugin_impl]` linked into the current binary as a normal rlib). No dylib is loaded — the descriptor's vtable points at code in the current binary.

Used by the generated `Client::in_process(plugin_name)` constructor.
Host applications normally use [`CdylibExecutor::from_loaded`](../../../../rust/fidius-host/executor/cdylib/#CdylibExecutor.from_loaded) instead.

<details>
<summary>Source</summary>

```rust
    pub fn from_descriptor(desc: &'static PluginDescriptor) -> Result<Self, LoadError> {
        Self::from_descriptor_with_config(desc, &[])
    }
```

</details>

##### <span id="CdylibExecutor.from_descriptor_with_config"></span><code>from_​descriptor_​with_​config</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn from_descriptor_with_config(
    desc: &'static PluginDescriptor,
    cfg: &[u8],
) -> Result<Self, LoadError>
```

Like `Self::from_descriptor` but constructs the instance from serialized config bytes (FIDIUS-A-0006 / CI.2) — the in-process *configured* path. `cfg` is bincode of the plugin's config type (empty = the singleton).

<details>
<summary>Source</summary>

```rust
    pub fn from_descriptor_with_config(
        desc: &'static PluginDescriptor,
        cfg: &[u8],
    ) -> Result<Self, LoadError> {
        let info = PluginInfo {
            name: unsafe { desc.plugin_name_str() }.to_string(),
            interface_name: unsafe { desc.interface_name_str() }.to_string(),
            interface_hash: desc.interface_hash,
            interface_version: desc.interface_version,
            capabilities: desc.capabilities,
            buffer_strategy: desc
                .buffer_strategy_kind()
                .map_err(|v| LoadError::UnknownBufferStrategy { value: v })?,
            runtime: crate::types::PluginRuntimeKind::Cdylib,
        };
        let descriptor = desc as *const PluginDescriptor;
        let instance = unsafe { construct_instance(descriptor, cfg) };
        Ok(Self {
            _library: None,
            vtable: desc.vtable,
            descriptor,
            free_buffer: desc.free_buffer,
            capabilities: desc.capabilities,
            method_count: desc.method_count,
            info,
            instance,
            destroy: desc.destroy,
        })
    }
```

</details>

##### <span id="CdylibExecutor.find_in_process_descriptor"></span><code>find_​in_​process_​descriptor</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn find_in_process_descriptor(
    plugin_name: &str,
) -> Result<&'static PluginDescriptor, LoadError>
```

Look up a descriptor in the current process's inventory registry by `plugin_name` (the Rust struct name that was passed to `#[plugin_impl]`). Returns `LoadError::PluginNotFound` if no descriptor has that name.

The returned reference has `'static` lifetime because descriptors
emitted by `#[plugin_impl]` live in the binary's `.rodata`.

<details>
<summary>Source</summary>

```rust
    pub fn find_in_process_descriptor(
        plugin_name: &str,
    ) -> Result<&'static PluginDescriptor, LoadError> {
        let reg = fidius_core::registry::get_registry();
        for i in 0..reg.plugin_count as usize {
            let desc_ptr = unsafe { *reg.descriptors.add(i) };
            let desc = unsafe { &*desc_ptr };
            if unsafe { desc.plugin_name_str() } == plugin_name {
                return Ok(desc);
            }
        }
        Err(LoadError::PluginNotFound {
            name: plugin_name.to_string(),
        })
    }
```

</details>

##### <span id="CdylibExecutor.call_method"></span><code>call_​method</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn call_method<I: Serialize, O: DeserializeOwned>(
    &self,
    index: usize,
    input: &I,
) -> Result<O, CallError>
```

Call a plugin method by vtable index.

Serializes the input, calls the FFI function pointer at the given index,
checks the status code, deserializes the output, and frees the plugin-allocated buffer.

**No timeout**

This call runs synchronously on the calling thread and has no built-in
timeout or cancellation. A misbehaving plugin will block the caller
indefinitely. See the `fidius` crate top-level docs ("What fidius
does not provide") for the rationale and the recommended consumer-side
mitigation.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `index` | `usize` | The method index in the vtable (0-based, in declaration order) |
| `input` | `&I` | The input argument to serialize and pass to the plugin |

<details>
<summary>Source</summary>

```rust
    pub fn call_method<I: Serialize, O: DeserializeOwned>(
        &self,
        index: usize,
        input: &I,
    ) -> Result<O, CallError> {
        // Bounds check: ensure index is within the vtable
        if index >= self.method_count as usize {
            return Err(CallError::InvalidMethodIndex {
                index,
                count: self.method_count,
            });
        }

        let input_bytes =
            wire::serialize(input).map_err(|e| CallError::Serialization(e.to_string()))?;

        match self.info.buffer_strategy {
            BufferStrategyKind::PluginAllocated => self.call_plugin_allocated(index, &input_bytes),
            BufferStrategyKind::Arena => self.call_arena(index, &input_bytes),
        }
    }
```

</details>

##### <span id="CdylibExecutor.call_method_raw"></span><code>call_​method_​raw</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn call_method_raw(
    &self,
    index: usize,
    input: &[u8],
) -> Result<Vec<u8>, CallError>
```

Call a plugin method whose argument and successful return value are raw bytes — bypassing bincode on both sides. Used by methods declared with `#[wire(raw)]` on the interface trait.

Errors and panic messages still use bincode (small typed payloads).
Returns the success bytes on `Ok`, or a `CallError::Plugin(_)` whose
inner `PluginError` was bincode-decoded from the plugin's error payload.

Same no-timeout caveat as `Self::call_method`.

<details>
<summary>Source</summary>

```rust
    pub fn call_method_raw(&self, index: usize, input: &[u8]) -> Result<Vec<u8>, CallError> {
        if index >= self.method_count as usize {
            return Err(CallError::InvalidMethodIndex {
                index,
                count: self.method_count,
            });
        }
        match self.info.buffer_strategy {
            BufferStrategyKind::PluginAllocated => self.call_plugin_allocated_raw(index, input),
            BufferStrategyKind::Arena => self.call_arena_raw(index, input),
        }
    }
```

</details>

##### <span id="CdylibExecutor.call_client_streaming_raw"></span><code>call_​client_​streaming_​raw</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>
 <span class="plissken-badge plissken-badge-unsafe" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-red-low); color: var(--sl-color-red-high);">unsafe</span>

```rust
unsafe fn call_client_streaming_raw(
    &self,
    index: usize,
    handle: *mut fidius_core::stream_ffi::FidiusStreamHandle,
    input_bytes: &[u8],
) -> Result<Vec<u8>, CallError>
```

Client-streaming raw call (FIDIUS-I-0030 CS2.2). The vtable slot is a `ClientStreamFn` that also takes the host's producer `handle`, from which the plugin pulls its `Stream<T>` argument. `input_bytes` is the bincode of the **non-stream** args; the bincode of the method's result is returned. The plugin's consumer frees `handle` via its `drop_fn` — the host must not.

**Safety**

`handle` must be a valid, exclusively-owned producer handle (e.g. from
[`crate::client_stream::host_producer_handle`](../../../../rust/fidius-host/client_stream/#fn.host_producer_handle)); it is consumed by the call.

<details>
<summary>Source</summary>

```rust
    pub unsafe fn call_client_streaming_raw(
        &self,
        index: usize,
        handle: *mut fidius_core::stream_ffi::FidiusStreamHandle,
        input_bytes: &[u8],
    ) -> Result<Vec<u8>, CallError> {
        if index >= self.method_count as usize {
            return Err(CallError::InvalidMethodIndex {
                index,
                count: self.method_count,
            });
        }
        type ClientStreamFn = unsafe extern "C" fn(
            *mut c_void,
            *mut fidius_core::stream_ffi::FidiusStreamHandle,
            *const u8,
            u32,
            *mut *mut u8,
            *mut u32,
        ) -> i32;
        let fn_ptr = match unsafe { *(self.vtable as *const Option<ClientStreamFn>).add(index) } {
            Some(f) => f,
            None => return Err(CallError::NotImplemented { bit: index as u32 }),
        };

        let mut out_ptr: *mut u8 = std::ptr::null_mut();
        let mut out_len: u32 = 0;
        let status = unsafe {
            fn_ptr(
                self.instance,
                handle,
                input_bytes.as_ptr(),
                input_bytes.len() as u32,
                &mut out_ptr,
                &mut out_len,
            )
        };

        match status {
            STATUS_OK => {}
            STATUS_SERIALIZATION_ERROR => {
                return Err(CallError::Serialization("FFI serialization failed".into()))
            }
            STATUS_PLUGIN_ERROR => {
                let err = if !out_ptr.is_null() && out_len > 0 {
                    let slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
                    let pe: PluginError = wire::deserialize(slice)
                        .unwrap_or_else(|_| PluginError::new("UNKNOWN", "plugin error"));
                    if let Some(free) = self.free_buffer {
                        unsafe { free(out_ptr, out_len as usize) };
                    }
                    pe
                } else {
                    PluginError::new("UNKNOWN", "plugin returned error but no data")
                };
                return Err(CallError::Plugin(err));
            }
            STATUS_PANIC => {
                let msg = if !out_ptr.is_null() && out_len > 0 {
                    let slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
                    let m = wire::deserialize::<String>(slice)
                        .unwrap_or_else(|_| "unknown panic".into());
                    if let Some(free) = self.free_buffer {
                        unsafe { free(out_ptr, out_len as usize) };
                    }
                    m
                } else {
                    "unknown panic".into()
                };
                return Err(CallError::Panic(msg));
            }
            _ => return Err(CallError::UnknownStatus { code: status }),
        }

        if out_ptr.is_null() {
            return Err(CallError::Serialization(
                "plugin returned null output buffer".into(),
            ));
        }
        let result = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) }.to_vec();
        if let Some(free) = self.free_buffer {
            unsafe { free(out_ptr, out_len as usize) };
        }
        Ok(result)
    }
```

</details>

##### <span id="CdylibExecutor.call_streaming_raw"></span><code>call_​streaming_​raw</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn call_streaming_raw(
    &self,
    index: usize,
    input_bytes: &[u8],
    decode_item: fn(&[u8]) -> Result<fidius_core::Value, CallError>,
) -> Result<crate::stream::ChunkStream, CallError>
```

Start a server-streaming cdylib call (FIDIUS-I-0026 CS.1). `input_bytes` is the **concrete bincode** of the args tuple (the cdylib path never goes through `Value` — same as `call_method`), so the caller serialises with `wire::serialize` directly.

Calls the streaming method's vtable slot (an *init* shim with the ordinary
`FfiFn` shape) to obtain a `FidiusStreamHandle`, then pumps `next()` on a
dedicated thread (cdylib is synchronous) into a bounded channel →
`ChunkStream`. The pump owns **one reusable buffer** the guest writes each
item into (FIDIUS-T-0138 arena-style `next`) — so there's no per-item heap
alloc and no `free_buffer` crossing, just one `next` call per item. Each
item crosses as **concrete bincode** and is turned into a `Value` by the
caller-supplied `decode_item` (`wire::deserialize::<O>` + `to_value`,
FIDIUS-T-0137). Dropping the stream runs the guest's `drop_fn` (cancel).

<details>
<summary>Source</summary>

```rust
    pub fn call_streaming_raw(
        &self,
        index: usize,
        input_bytes: &[u8],
        decode_item: fn(&[u8]) -> Result<fidius_core::Value, CallError>,
    ) -> Result<crate::stream::ChunkStream, CallError> {
        if index >= self.method_count as usize {
            return Err(CallError::InvalidMethodIndex {
                index,
                count: self.method_count,
            });
        }

        // init: the streaming method's vtable slot (FfiFn shape) → handle. Guard
        // the null slot of an unimplemented optional method.
        let init = match unsafe { *(self.vtable as *const Option<FfiFn>).add(index) } {
            Some(f) => f,
            None => return Err(CallError::NotImplemented { bit: index as u32 }),
        };
        let mut out_ptr: *mut u8 = std::ptr::null_mut();
        let mut out_len: u32 = 0;
        let status = unsafe {
            init(
                self.instance,
                input_bytes.as_ptr(),
                input_bytes.len() as u32,
                &mut out_ptr,
                &mut out_len,
            )
        };
        match status {
            STATUS_OK => {}
            STATUS_SERIALIZATION_ERROR => {
                return Err(CallError::Serialization(
                    "stream init: argument decode failed".into(),
                ))
            }
            STATUS_PANIC => return Err(CallError::Panic("plugin panicked in stream init".into())),
            code => return Err(CallError::UnknownStatus { code }),
        }
        if out_ptr.is_null() {
            return Err(CallError::Backend {
                runtime: "cdylib".into(),
                message: "stream init returned a null handle".into(),
            });
        }

        // Pump the returned handle into a `ChunkStream` (shared with the
        // bidirectional path, which reaches the same output-stream handle).
        Ok(pump_stream_handle(
            out_ptr,
            decode_item,
            self._library.clone(),
        ))
    }
```

</details>

##### <span id="CdylibExecutor.call_bidi_streaming_raw"></span><code>call_​bidi_​streaming_​raw</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>
 <span class="plissken-badge plissken-badge-unsafe" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-red-low); color: var(--sl-color-red-high);">unsafe</span>

```rust
unsafe fn call_bidi_streaming_raw(
    &self,
    index: usize,
    handle: *mut fidius_core::stream_ffi::FidiusStreamHandle,
    input_bytes: &[u8],
    decode_item: fn(&[u8]) -> Result<fidius_core::Value, CallError>,
) -> Result<crate::stream::ChunkStream, CallError>
```

Bidirectional streaming (FIDIUS-I-0032 / ADR-0010): call a method whose vtable slot is a `ClientStreamFn` returning the OUTPUT stream handle. `handle` is the host's INPUT producer (the plugin pulls its `Stream<In>` from it); `input_bytes` is the bincode of the non-stream args. Returns a `ChunkStream` over the plugin's `Stream<Out>` — pulling it drives the plugin, which re-enters `handle.next()` on demand (the synchronous lazy-pull composition).

**Safety**

`handle` must be a valid, exclusively-owned producer handle; it is consumed
by the call (the plugin's output stream frees it via its `drop_fn`).

<details>
<summary>Source</summary>

```rust
    pub unsafe fn call_bidi_streaming_raw(
        &self,
        index: usize,
        handle: *mut fidius_core::stream_ffi::FidiusStreamHandle,
        input_bytes: &[u8],
        decode_item: fn(&[u8]) -> Result<fidius_core::Value, CallError>,
    ) -> Result<crate::stream::ChunkStream, CallError> {
        if index >= self.method_count as usize {
            return Err(CallError::InvalidMethodIndex {
                index,
                count: self.method_count,
            });
        }
        // Same FFI shape as client-streaming: instance + input handle + args + out.
        type ClientStreamFn = unsafe extern "C" fn(
            *mut c_void,
            *mut fidius_core::stream_ffi::FidiusStreamHandle,
            *const u8,
            u32,
            *mut *mut u8,
            *mut u32,
        ) -> i32;
        let init = match unsafe { *(self.vtable as *const Option<ClientStreamFn>).add(index) } {
            Some(f) => f,
            None => return Err(CallError::NotImplemented { bit: index as u32 }),
        };
        let mut out_ptr: *mut u8 = std::ptr::null_mut();
        let mut out_len: u32 = 0;
        let status = unsafe {
            init(
                self.instance,
                handle,
                input_bytes.as_ptr(),
                input_bytes.len() as u32,
                &mut out_ptr,
                &mut out_len,
            )
        };
        match status {
            STATUS_OK => {}
            STATUS_SERIALIZATION_ERROR => {
                return Err(CallError::Serialization(
                    "bidi stream init: argument decode failed".into(),
                ))
            }
            STATUS_PANIC => {
                return Err(CallError::Panic(
                    "plugin panicked in bidi stream init".into(),
                ))
            }
            code => return Err(CallError::UnknownStatus { code }),
        }
        if out_ptr.is_null() {
            return Err(CallError::Backend {
                runtime: "cdylib".into(),
                message: "bidi stream init returned a null output handle".into(),
            });
        }
        Ok(pump_stream_handle(
            out_ptr,
            decode_item,
            self._library.clone(),
        ))
    }
```

</details>

##### <span id="CdylibExecutor.has_capability"></span><code>has_​capability</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn has_capability(&self, bit: u32) -> bool
```

Check if an optional method is supported (capability bit is set).

Returns `false` for bit indices >= 64 rather than panicking.

<details>
<summary>Source</summary>

```rust
    pub fn has_capability(&self, bit: u32) -> bool {
        if bit >= 64 {
            return false;
        }
        self.capabilities & (1u64 << bit) != 0
    }
```

</details>

##### <span id="CdylibExecutor.info"></span><code>info</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn info(&self) -> &PluginInfo
```

Access the plugin's owned metadata.

<details>
<summary>Source</summary>

```rust
    pub fn info(&self) -> &PluginInfo {
        &self.info
    }
```

</details>

##### <span id="CdylibExecutor.method_metadata"></span><code>method_​metadata</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn method_metadata(&self, method_id: u32) -> Vec<(&str, &str)>
```

Returns the static key/value metadata declared on the given method via `#[method_meta(...)]` attributes on the trait, in declaration order.

Returns an empty `Vec` if:
- `method_id >= method_count` (out of range)
- the interface declared no method metadata on any method
- this specific method has no metadata declared

The returned `&str` slices borrow from the loaded library's `.rodata`
(for dylib-loaded handles) or from the current binary's `.rodata`
(for in-process handles). The handle's lifetime bounds them safely.

<details>
<summary>Source</summary>

```rust
    pub fn method_metadata(&self, method_id: u32) -> Vec<(&str, &str)> {
        if method_id >= self.method_count {
            return Vec::new();
        }
        // SAFETY: descriptor pointer is valid for the handle's lifetime.
        let desc = unsafe { &*self.descriptor };
        if desc.method_metadata.is_null() {
            return Vec::new();
        }
        // SAFETY: when method_metadata is non-null, it points at an array
        // of method_count entries (codegen invariant).
        let entries =
            unsafe { std::slice::from_raw_parts(desc.method_metadata, self.method_count as usize) };
        let entry = &entries[method_id as usize];
        if entry.kvs.is_null() || entry.kv_count == 0 {
            return Vec::new();
        }
        // SAFETY: kvs points at an array of kv_count MetaKv entries.
        let kvs = unsafe { std::slice::from_raw_parts(entry.kvs, entry.kv_count as usize) };
        kvs.iter()
            .map(|kv| {
                // SAFETY: both pointers are static, null-terminated UTF-8
                // per the ABI contract enforced by the macro.
                let k = unsafe { std::ffi::CStr::from_ptr(kv.key) }
                    .to_str()
                    .expect("metadata key is not valid UTF-8");
                let v = unsafe { std::ffi::CStr::from_ptr(kv.value) }
                    .to_str()
                    .expect("metadata value is not valid UTF-8");
                (k, v)
            })
            .collect()
    }
```

</details>

##### <span id="CdylibExecutor.trait_metadata"></span><code>trait_​metadata</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn trait_metadata(&self) -> Vec<(&str, &str)>
```

Returns the static key/value metadata declared on the trait via `#[trait_meta(...)]` attributes, in declaration order.

Returns an empty `Vec` if no trait-level metadata was declared.

<details>
<summary>Source</summary>

```rust
    pub fn trait_metadata(&self) -> Vec<(&str, &str)> {
        // SAFETY: descriptor pointer is valid for the handle's lifetime.
        let desc = unsafe { &*self.descriptor };
        if desc.trait_metadata.is_null() || desc.trait_metadata_count == 0 {
            return Vec::new();
        }
        // SAFETY: trait_metadata points at an array of trait_metadata_count entries.
        let kvs = unsafe {
            std::slice::from_raw_parts(desc.trait_metadata, desc.trait_metadata_count as usize)
        };
        kvs.iter()
            .map(|kv| {
                let k = unsafe { std::ffi::CStr::from_ptr(kv.key) }
                    .to_str()
                    .expect("trait metadata key is not valid UTF-8");
                let v = unsafe { std::ffi::CStr::from_ptr(kv.value) }
                    .to_str()
                    .expect("trait metadata value is not valid UTF-8");
                (k, v)
            })
            .collect()
    }
```

</details>
