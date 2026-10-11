---
title: "fidius-guest::wasm_descriptor"
description: "Compile-time descriptor of a fidius interface used by the WASM loader."
sidebar:
  order: 25
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


Compile-time descriptor of a fidius interface used by the WASM loader.

Like [`crate::python_descriptor`], a WASM component has no host-readable
vtable, so the host needs an out-of-band hint: which exported interface the
component implements, the method names in declaration (vtable) order with
their wire mode, and the expected interface hash. The `#[plugin_interface]`
macro emits this (Phase 3); for Phase 2 it is hand-authored alongside the
reference WIT.

## Structs

### <span id="struct.WasmInterfaceDescriptor"></span>`struct WasmInterfaceDescriptor`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


**Derives:** `Debug`, `Clone`, `Copy`

Static descriptor for one fidius interface, consumed by the WASM loader to validate and dispatch into a component.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `interface_name` | `& 'static str` | Trait name, for diagnostics. |
| `interface_export` | `& 'static str` | Fully-qualified exported interface the component must provide, e.g.
`"fidius:greeter/greeter@1.0.0"`. The host navigates to this interface's
exports to dispatch methods. |
| `interface_hash` | `u64` | Same hash the cdylib path bakes into its `PluginDescriptor`. The
component's `fidius-interface-hash` export must return this. |
| `methods` | `& 'static [WasmMethodDesc]` | Methods in declaration order — index here lines up with the cdylib
vtable index for the same trait. |



### <span id="struct.WasmMethodDesc"></span>`struct WasmMethodDesc`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


**Derives:** `Debug`, `Clone`, `Copy`

One method on the interface.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `name` | `& 'static str` | Export name within the interface (e.g. `"greet"`). |
| `wire_raw` | `bool` | Whether this method uses raw byte-passthrough wire mode (`#[wire(raw)]`). |
| `streaming` | `bool` | Whether this method is **server-streaming** (`-> fidius::Stream<T>`,
FIDIUS-I-0026). When true the export returns a `next()`-pollable resource
rather than a value, so the host routes it through the streaming path. |



