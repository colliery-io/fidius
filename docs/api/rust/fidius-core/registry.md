# fidius-core::registry <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Plugin registry assembly for multi-plugin dylibs.

Each `#[plugin_impl]` submits its `PluginDescriptor` pointer via `inventory::submit!`.
Plugin crates call `fidius_plugin_registry!()` once in their lib.rs to emit the
`fidius_get_registry` export function that the host calls via `dlsym`.

## Structs

### `fidius-core::registry::DescriptorEntry`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


A submitted descriptor pointer. Used with `inventory` for distributed collection.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `descriptor` | `& 'static PluginDescriptor` |  |



## Functions

### `fidius-core::registry::build_registry`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn build_registry () -> PluginRegistry
```

Build the plugin registry from all submitted descriptors.

Allocates a `Vec` of descriptor pointers and leaks it to get a `'static` pointer.
Called once; the result is cached in `OnceLock`.

<details>
<summary>Source</summary>

```rust
fn build_registry() -> PluginRegistry {
    let entries: Vec<*const PluginDescriptor> = inventory::iter::<DescriptorEntry>()
        .map(|e| e.descriptor as *const PluginDescriptor)
        .collect();

    let count = entries.len() as u32;
    let ptr = entries.as_ptr();
    std::mem::forget(entries);

    PluginRegistry {
        magic: FIDIUS_MAGIC,
        registry_version: REGISTRY_VERSION,
        plugin_count: count,
        descriptors: ptr,
    }
}
```

</details>



### `fidius-core::registry::get_registry`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn get_registry () -> & 'static PluginRegistry
```

Get or build the plugin registry.

Returns a `'static` reference to the registry. Built on first call,
cached for subsequent calls.

<details>
<summary>Source</summary>

```rust
pub fn get_registry() -> &'static PluginRegistry {
    static REGISTRY: std::sync::OnceLock<PluginRegistry> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(build_registry)
}
```

</details>



