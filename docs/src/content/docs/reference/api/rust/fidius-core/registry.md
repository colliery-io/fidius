---
title: "fidius-core::registry"
description: "Plugin registry assembly for multi-plugin dylibs."
sidebar:
  order: 9
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


Plugin registry assembly for multi-plugin dylibs.

Each `#[plugin_impl]` submits its `PluginDescriptor` pointer via `inventory::submit!`.
Plugin crates call `fidius_plugin_registry!()` once in their lib.rs to emit the
`fidius_get_registry` export function that the host calls via `dlsym`.

## Structs

### <span id="struct.DescriptorEntry"></span>`struct DescriptorEntry`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


A submitted descriptor pointer. Used with `inventory` for distributed collection.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `descriptor` | `& 'static PluginDescriptor` |  |



## Functions

### <span id="fn.build_registry"></span>`fn build_registry`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


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



### <span id="fn.get_registry"></span>`fn get_registry`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


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



