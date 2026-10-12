---
title: "fidius-core::host_registry"
description: "Host-import registry assembly (plugin → host callback channel)."
editUrl: false
sidebar:
  order: 6
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Host-import registry assembly (plugin → host callback channel).

Each `#[fidius::host_interface]` trait submits a [`HostImportDescriptor`](../../../rust/fidius-guest/host_ffi/#struct.HostImportDescriptor)
pointer via `inventory::submit!` (from the interface crate, gated to
non-wasm builds). `fidius_plugin_registry!()` emits the optional
`fidius_get_host_imports` export that collects them, so a plugin dylib
that links a host interface automatically advertises it. Plugins with no
host interfaces export an **empty** registry — and plugins built before
this channel existed export no such symbol at all; the host treats both
identically (nothing to bind).

## Structs

### <span id="struct.HostImportEntry"></span><code style="font-size: 0.85em">struct HostImportEntry</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

A submitted host-import descriptor pointer, collected via `inventory`.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `descriptor` | `&'static HostImportDescriptor` |  |

## Functions

### <span id="fn.get_host_import_registry"></span><code style="font-size: 0.85em">fn get_​host_​import_​registry</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn get_host_import_registry() -> &'static HostImportRegistry
```

Get or build the host-import registry (cached after first call).

<details>
<summary>Source</summary>

```rust
pub fn get_host_import_registry() -> &'static HostImportRegistry {
    static REGISTRY: std::sync::OnceLock<HostImportRegistry> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(build_host_import_registry)
}
```

</details>
