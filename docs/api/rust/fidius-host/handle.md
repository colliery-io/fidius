# fidius-host::handle <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


PluginHandle — type-safe proxy for calling plugin methods via FFI.

## Structs

### `fidius-host::handle::PluginHandle`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


A handle to a loaded plugin, ready for calling methods.

Holds an `Arc<Library>` to keep the dylib loaded as long as any handle exists.
Call methods via `call_method()` which handles serialization, FFI, and cleanup.
`PluginHandle` is `Send + Sync`. Plugin methods take `&self` (enforced by
the macro), so concurrent calls from multiple threads are safe as long as
the plugin implementation is thread-safe internally.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `_library` | `Option < Arc < Library > >` | Keeps the library alive for dylib-loaded plugins. `None` for in-process
handles built via [`PluginHandle::from_descriptor`] — in-process plugins
live in the current binary's address space and don't need Arc-tracking. |
| `vtable` | `* const c_void` | Pointer to the `#[repr(C)]` vtable struct in the loaded library. |
| `descriptor` | `* const PluginDescriptor` | Pointer to the full descriptor in library memory. Used by metadata
accessors to read `method_metadata` / `trait_metadata`. Valid for the
handle's lifetime via `_library` Arc (or forever for in-process). |
| `free_buffer` | `Option < unsafe extern "C" fn (* mut u8 , usize) >` | Free function for plugin-allocated output buffers. |
| `capabilities` | `u64` | Capability bitfield for optional method support. |
| `method_count` | `u32` | Total number of methods in the vtable. |
| `info` | `PluginInfo` | Owned plugin metadata. |

#### Methods

##### `new` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">pub(crate)</span>


```rust
fn new (library : Arc < Library > , vtable : * const c_void , descriptor : * const PluginDescriptor , free_buffer : Option < unsafe extern "C" fn (* mut u8 , usize) > , capabilities : u64 , method_count : u32 , info : PluginInfo ,) -> Self
```

Create a new PluginHandle. Crate-private — use `from_loaded()` instead.

<details>
<summary>Source</summary>

```rust
    pub(crate) fn new(
        library: Arc<Library>,
        vtable: *const c_void,
        descriptor: *const PluginDescriptor,
        free_buffer: Option<unsafe extern "C" fn(*mut u8, usize)>,
        capabilities: u64,
        method_count: u32,
        info: PluginInfo,
    ) -> Self {
        Self {
            _library: Some(library),
            vtable,
            descriptor,
            free_buffer,
            capabilities,
            method_count,
            info,
        }
    }
```

</details>



##### `from_loaded` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn from_loaded (plugin : crate :: loader :: LoadedPlugin) -> Self
```

Create a PluginHandle from a LoadedPlugin.

<details>
<summary>Source</summary>

```rust
    pub fn from_loaded(plugin: crate::loader::LoadedPlugin) -> Self {
        Self {
            _library: Some(plugin.library),
            vtable: plugin.vtable,
            descriptor: plugin.descriptor,
            free_buffer: plugin.free_buffer,
            capabilities: plugin.info.capabilities,
            method_count: plugin.method_count,
            info: plugin.info,
        }
    }
```

</details>



##### `from_descriptor` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn from_descriptor (desc : & 'static PluginDescriptor) -> Result < Self , LoadError >
```

Create a PluginHandle from a plugin descriptor already registered in the current process's inventory (via a `#[plugin_impl]` linked into the current binary as a normal rlib). No dylib is loaded — the descriptor's vtable points at code in the current binary.

Used by the generated `Client::in_process(plugin_name)` constructor.
Host applications normally use [`PluginHandle::from_loaded`] instead.

<details>
<summary>Source</summary>

```rust
    pub fn from_descriptor(desc: &'static PluginDescriptor) -> Result<Self, LoadError> {
        let info = PluginInfo {
            name: unsafe { desc.plugin_name_str() }.to_string(),
            interface_name: unsafe { desc.interface_name_str() }.to_string(),
            interface_hash: desc.interface_hash,
            interface_version: desc.interface_version,
            capabilities: desc.capabilities,
            buffer_strategy: desc
                .buffer_strategy_kind()
                .map_err(|v| LoadError::UnknownBufferStrategy { value: v })?,
        };
        Ok(Self {
            _library: None,
            vtable: desc.vtable,
            descriptor: desc as *const PluginDescriptor,
            free_buffer: desc.free_buffer,
            capabilities: desc.capabilities,
            method_count: desc.method_count,
            info,
        })
    }
```

</details>



##### `find_in_process_descriptor` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn find_in_process_descriptor (plugin_name : & str ,) -> Result < & 'static PluginDescriptor , LoadError >
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



##### `call_method` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn call_method < I : Serialize , O : DeserializeOwned > (& self , index : usize , input : & I ,) -> Result < O , CallError >
```

Call a plugin method by vtable index.

Serializes the input, calls the FFI function pointer at the given index,
checks the status code, deserializes the output, and frees the plugin-allocated buffer.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `index` | `-` | The method index in the vtable (0-based, in declaration order) |
| `input` | `-` | The input argument to serialize and pass to the plugin |


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



##### `call_plugin_allocated` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn call_plugin_allocated < O : DeserializeOwned > (& self , index : usize , input_bytes : & [u8] ,) -> Result < O , CallError >
```

PluginAllocated path: plugin allocates an output buffer via `Box::into_raw(Box<[u8]>)`, host deserializes and calls free_buffer.

<details>
<summary>Source</summary>

```rust
    fn call_plugin_allocated<O: DeserializeOwned>(
        &self,
        index: usize,
        input_bytes: &[u8],
    ) -> Result<O, CallError> {
        let fn_ptr = unsafe {
            let fn_ptrs = self.vtable as *const FfiFn;
            *fn_ptrs.add(index)
        };

        let mut out_ptr: *mut u8 = std::ptr::null_mut();
        let mut out_len: u32 = 0;

        let status = unsafe {
            fn_ptr(
                input_bytes.as_ptr(),
                input_bytes.len() as u32,
                &mut out_ptr,
                &mut out_len,
            )
        };

        match status {
            STATUS_OK => {}
            STATUS_BUFFER_TOO_SMALL => return Err(CallError::BufferTooSmall),
            STATUS_SERIALIZATION_ERROR => {
                return Err(CallError::Serialization("FFI serialization failed".into()))
            }
            STATUS_PLUGIN_ERROR => {
                if !out_ptr.is_null() && out_len > 0 {
                    let output_slice =
                        unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
                    let plugin_err: PluginError = wire::deserialize(output_slice)
                        .map_err(|e| CallError::Deserialization(e.to_string()))?;

                    if let Some(free) = self.free_buffer {
                        unsafe { free(out_ptr, out_len as usize) };
                    }

                    return Err(CallError::Plugin(plugin_err));
                }
                return Err(CallError::Plugin(PluginError::new(
                    "UNKNOWN",
                    "plugin returned error but no error data",
                )));
            }
            STATUS_PANIC => {
                let msg = if !out_ptr.is_null() && out_len > 0 {
                    let slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
                    let msg = wire::deserialize::<String>(slice)
                        .unwrap_or_else(|_| "unknown panic".into());
                    if let Some(free) = self.free_buffer {
                        unsafe { free(out_ptr, out_len as usize) };
                    }
                    msg
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

        let output_slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
        let result: Result<O, CallError> =
            wire::deserialize(output_slice).map_err(|e| CallError::Deserialization(e.to_string()));

        if let Some(free) = self.free_buffer {
            unsafe { free(out_ptr, out_len as usize) };
        }

        result
    }
```

</details>



##### `call_arena` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn call_arena < O : DeserializeOwned > (& self , index : usize , input_bytes : & [u8] ,) -> Result < O , CallError >
```

Arena path: host supplies a buffer from the thread-local pool. If the plugin reports `STATUS_BUFFER_TOO_SMALL`, grow the buffer to the requested size and retry exactly once (second too-small would indicate a misbehaving plugin — bail with `CallError::BufferTooSmall`).

<details>
<summary>Source</summary>

```rust
    fn call_arena<O: DeserializeOwned>(
        &self,
        index: usize,
        input_bytes: &[u8],
    ) -> Result<O, CallError> {
        let fn_ptr = unsafe {
            let fn_ptrs = self.vtable as *const ArenaFn;
            *fn_ptrs.add(index)
        };

        let mut arena = acquire_arena(DEFAULT_ARENA_CAPACITY);
        let mut out_offset: u32 = 0;
        let mut out_len: u32 = 0;
        let mut retried = false;

        let status = loop {
            let s = unsafe {
                fn_ptr(
                    input_bytes.as_ptr(),
                    input_bytes.len() as u32,
                    arena.as_mut_ptr(),
                    arena.len() as u32,
                    &mut out_offset,
                    &mut out_len,
                )
            };
            if s == STATUS_BUFFER_TOO_SMALL && !retried {
                // Plugin wrote the needed size into out_len. Grow and retry once.
                let needed = out_len as usize;
                grow_arena(&mut arena, needed);
                retried = true;
                continue;
            }
            break s;
        };

        match status {
            STATUS_OK => {
                let start = out_offset as usize;
                let end = start + out_len as usize;
                if end > arena.len() {
                    release_arena(arena);
                    return Err(CallError::Serialization(
                        "plugin reported out_offset/out_len outside arena".into(),
                    ));
                }
                let result = wire::deserialize(&arena[start..end])
                    .map_err(|e| CallError::Deserialization(e.to_string()));
                release_arena(arena);
                result
            }
            STATUS_BUFFER_TOO_SMALL => {
                release_arena(arena);
                Err(CallError::BufferTooSmall)
            }
            STATUS_SERIALIZATION_ERROR => {
                release_arena(arena);
                Err(CallError::Serialization("FFI serialization failed".into()))
            }
            STATUS_PLUGIN_ERROR => {
                let start = out_offset as usize;
                let end = start + out_len as usize;
                let plugin_err = if out_len > 0 && end <= arena.len() {
                    wire::deserialize::<PluginError>(&arena[start..end]).unwrap_or_else(|_| {
                        PluginError::new("UNKNOWN", "plugin returned malformed error")
                    })
                } else {
                    PluginError::new("UNKNOWN", "plugin returned error but no error data")
                };
                release_arena(arena);
                Err(CallError::Plugin(plugin_err))
            }
            STATUS_PANIC => {
                // Arena strategy's panic path returns out_len = 0 (the arena
                // might be too small for the panic message). Host can't
                // recover a message; report an opaque panic.
                release_arena(arena);
                Err(CallError::Panic(
                    "plugin panicked (message not transmitted via Arena strategy)".into(),
                ))
            }
            code => {
                release_arena(arena);
                Err(CallError::UnknownStatus { code })
            }
        }
    }
```

</details>



##### `has_capability` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn has_capability (& self , bit : u32) -> bool
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



##### `info` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn info (& self) -> & PluginInfo
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



##### `method_metadata` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn method_metadata (& self , method_id : u32) -> Vec < (& str , & str) >
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



##### `trait_metadata` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn trait_metadata (& self) -> Vec < (& str , & str) >
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





