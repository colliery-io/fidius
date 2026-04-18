# fidius-host::loader <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Core plugin loading and descriptor validation.

## Structs

### `fidius-host::loader::LoadedLibrary`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


A loaded plugin library with validated descriptors.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `library` | `Arc < Library >` | The dynamically loaded library. Must stay alive while any PluginHandle exists. |
| `plugins` | `Vec < LoadedPlugin >` | Validated plugin descriptors with owned metadata. |



### `fidius-host::loader::LoadedPlugin`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


A single validated plugin from a loaded library.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `info` | `PluginInfo` | Owned metadata copied from the FFI descriptor. |
| `vtable` | `* const c_void` | Raw vtable pointer (points into the loaded library's memory). |
| `free_buffer` | `Option < unsafe extern "C" fn (* mut u8 , usize) >` | Free function for plugin-allocated buffers. |
| `method_count` | `u32` | Total number of methods in the vtable. |
| `descriptor` | `* const PluginDescriptor` | Raw pointer to the plugin's descriptor in library memory. Kept so the
host can read metadata fields (`method_metadata`, `trait_metadata`)
without re-walking the registry. Valid for the lifetime of `library`. |
| `library` | `Arc < Library >` | Reference to the library to keep it alive. |



## Functions

### `fidius-host::loader::load_library`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn load_library (path : & Path) -> Result < LoadedLibrary , LoadError >
```

Load a plugin library from a path.

Opens the dylib, calls `fidius_get_registry()`, validates the registry
and all descriptors, copies FFI data to owned types.

<details>
<summary>Source</summary>

```rust
pub fn load_library(path: &Path) -> Result<LoadedLibrary, LoadError> {
    let path_str = path.display().to_string();

    #[cfg(feature = "tracing")]
    tracing::debug!(path = %path_str, "loading library");

    // Check architecture before dlopen
    crate::arch::check_architecture(path)?;

    // dlopen
    let library = unsafe { Library::new(path) }.map_err(|e| {
        if e.to_string().contains("No such file") || e.to_string().contains("not found") {
            LoadError::LibraryNotFound {
                path: path_str.clone(),
            }
        } else {
            LoadError::LibLoading(e)
        }
    })?;

    // dlsym("fidius_get_registry")
    let get_registry: libloading::Symbol<unsafe extern "C" fn() -> *const PluginRegistry> =
        unsafe { library.get(b"fidius_get_registry") }.map_err(|_| LoadError::SymbolNotFound {
            path: path_str.clone(),
        })?;

    // Call to get the registry pointer
    let registry = unsafe { &*get_registry() };

    // Validate magic
    if registry.magic != FIDIUS_MAGIC {
        return Err(LoadError::InvalidMagic);
    }

    // Validate registry version
    if registry.registry_version != REGISTRY_VERSION {
        return Err(LoadError::IncompatibleRegistryVersion {
            got: registry.registry_version,
            expected: REGISTRY_VERSION,
        });
    }

    let library = Arc::new(library);

    // Iterate descriptors and validate each
    let mut plugins = Vec::with_capacity(registry.plugin_count as usize);
    for i in 0..registry.plugin_count {
        let desc = unsafe { &**registry.descriptors.add(i as usize) };
        let plugin = validate_descriptor(desc, &library)?;
        plugins.push(plugin);
    }

    Ok(LoadedLibrary { library, plugins })
}
```

</details>



### `fidius-host::loader::validate_descriptor`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn validate_descriptor (desc : & PluginDescriptor , library : & Arc < Library > ,) -> Result < LoadedPlugin , LoadError >
```

Validate a single descriptor and copy to owned types.

<details>
<summary>Source</summary>

```rust
fn validate_descriptor(
    desc: &PluginDescriptor,
    library: &Arc<Library>,
) -> Result<LoadedPlugin, LoadError> {
    // Check ABI version
    if desc.abi_version != ABI_VERSION {
        return Err(LoadError::IncompatibleAbiVersion {
            got: desc.abi_version,
            expected: ABI_VERSION,
        });
    }

    // Copy FFI strings to owned
    let interface_name = unsafe { desc.interface_name_str() }.to_string();
    let plugin_name = unsafe { desc.plugin_name_str() }.to_string();

    let info = PluginInfo {
        name: plugin_name,
        interface_name,
        interface_hash: desc.interface_hash,
        interface_version: desc.interface_version,
        capabilities: desc.capabilities,
        buffer_strategy: desc
            .buffer_strategy_kind()
            .map_err(|v| LoadError::UnknownBufferStrategy { value: v })?,
    };

    Ok(LoadedPlugin {
        info,
        vtable: desc.vtable,
        free_buffer: desc.free_buffer,
        method_count: desc.method_count,
        descriptor: desc as *const PluginDescriptor,
        library: Arc::clone(library),
    })
}
```

</details>



### `fidius-host::loader::validate_against_interface`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn validate_against_interface (plugin : & LoadedPlugin , expected_hash : Option < u64 > , expected_strategy : Option < BufferStrategyKind > ,) -> Result < () , LoadError >
```

Validate a loaded plugin against expected interface parameters.

<details>
<summary>Source</summary>

```rust
pub fn validate_against_interface(
    plugin: &LoadedPlugin,
    expected_hash: Option<u64>,
    expected_strategy: Option<BufferStrategyKind>,
) -> Result<(), LoadError> {
    if let Some(hash) = expected_hash {
        if plugin.info.interface_hash != hash {
            return Err(LoadError::InterfaceHashMismatch {
                got: plugin.info.interface_hash,
                expected: hash,
            });
        }
    }

    if let Some(strategy) = expected_strategy {
        if plugin.info.buffer_strategy != strategy {
            return Err(LoadError::BufferStrategyMismatch {
                got: plugin.info.buffer_strategy,
                expected: strategy,
            });
        }
    }

    Ok(())
}
```

</details>



