---
title: "fidius-core::registry"
description: "Plugin registry assembly for multi-plugin dylibs."
editUrl: false
sidebar:
  order: 8
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Plugin registry assembly for multi-plugin dylibs.

Each `#[plugin_impl]` submits its `PluginDescriptor` pointer via `inventory::submit!`.
Plugin crates call `fidius_plugin_registry!()` once in their lib.rs to emit the
`fidius_get_registry` export function that the host calls via `dlsym`.

## Structs

### <span id="struct.DescriptorEntry"></span><code style="font-size: 0.85em">struct DescriptorEntry</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

A submitted descriptor pointer. Used with `inventory` for distributed collection.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `descriptor` | `&'static PluginDescriptor` |  |

## Functions

### <span id="fn.get_registry"></span><code style="font-size: 0.85em">fn get_​registry</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn get_registry() -> &'static PluginRegistry
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
