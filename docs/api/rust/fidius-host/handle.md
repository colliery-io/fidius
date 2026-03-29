# fidius-host::handle <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


PluginHandle — type-safe proxy for calling plugin methods via FFI.

## Structs

### `fidius-host::handle::PluginHandle`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


A handle to a loaded plugin, ready for calling methods.

Holds an `Arc<Library>` to keep the dylib loaded as long as any handle exists.
Call methods via `call_method()` which handles serialization, FFI, and cleanup.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `_library` | `Arc < Library >` | Keeps the library alive. |
| `vtable` | `* const c_void` | Pointer to the `#[repr(C)]` vtable struct in the loaded library. |
| `free_buffer` | `Option < unsafe extern "C" fn (* mut u8 , usize) >` | Free function for plugin-allocated output buffers. |
| `capabilities` | `u64` | Capability bitfield for optional method support. |
| `info` | `PluginInfo` | Owned plugin metadata. |

#### Methods

##### `new` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn new (library : Arc < Library > , vtable : * const c_void , free_buffer : Option < unsafe extern "C" fn (* mut u8 , usize) > , capabilities : u64 , info : PluginInfo ,) -> Self
```

Create a new PluginHandle from a loaded plugin.

<details>
<summary>Source</summary>

```rust
    pub fn new(
        library: Arc<Library>,
        vtable: *const c_void,
        free_buffer: Option<unsafe extern "C" fn(*mut u8, usize)>,
        capabilities: u64,
        info: PluginInfo,
    ) -> Self {
        Self {
            _library: library,
            vtable,
            free_buffer,
            capabilities,
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
            _library: plugin.library,
            vtable: plugin.vtable,
            free_buffer: plugin.free_buffer,
            capabilities: plugin.info.capabilities,
            info: plugin.info,
        }
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
        // Serialize input
        let input_bytes =
            wire::serialize(input).map_err(|e| CallError::Serialization(e.to_string()))?;

        // Get the function pointer from the vtable
        let fn_ptr = unsafe {
            let fn_ptrs = self.vtable as *const FfiFn;
            *fn_ptrs.add(index)
        };

        // Call the FFI function
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

        // Handle status codes
        match status {
            STATUS_OK => {}
            STATUS_BUFFER_TOO_SMALL => return Err(CallError::BufferTooSmall),
            STATUS_SERIALIZATION_ERROR => {
                return Err(CallError::Serialization("FFI serialization failed".into()))
            }
            STATUS_PLUGIN_ERROR => {
                // Output buffer contains a serialized PluginError
                if !out_ptr.is_null() && out_len > 0 {
                    let output_slice =
                        unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
                    let plugin_err: PluginError = wire::deserialize(output_slice)
                        .map_err(|e| CallError::Deserialization(e.to_string()))?;

                    // Free the buffer
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
            STATUS_PANIC => return Err(CallError::Panic),
            _ => {
                return Err(CallError::Serialization(format!(
                    "unknown status code: {status}"
                )))
            }
        }

        // Deserialize output
        let output_slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
        let result: Result<O, CallError> =
            wire::deserialize(output_slice).map_err(|e| CallError::Deserialization(e.to_string()));

        // Free the plugin-allocated buffer
        if let Some(free) = self.free_buffer {
            unsafe { free(out_ptr, out_len as usize) };
        }

        result
    }
```

</details>



##### `has_capability` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn has_capability (& self , bit : u32) -> bool
```

Check if an optional method is supported (capability bit is set).

<details>
<summary>Source</summary>

```rust
    pub fn has_capability(&self, bit: u32) -> bool {
        assert!(bit < 64, "capability bit must be < 64");
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





