---
title: "fidius-guest::python_descriptor"
description: "Compile-time descriptor of a fidius interface used by the Python loader."
sidebar:
  order: 19
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


Compile-time descriptor of a fidius interface used by the Python loader.

The cdylib path doesn't need any of this: the dylib carries its own
`PluginRegistry` + vtable that the host introspects at load time. A
Python plugin doesn't have a vtable; the host needs an out-of-band hint
about which method names exist on the trait, in what order, and with
which wire mode. The `#[plugin_interface]` macro emits a
`PythonInterfaceDescriptor` const into its companion module to provide
exactly that.
The descriptor is `'static`-shaped (string slices, slice of structs) so
it can sit in the binary's `.rodata` and be referenced freely.

## Structs

### <span id="struct.PythonInterfaceDescriptor"></span>`struct PythonInterfaceDescriptor`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


**Derives:** `Debug`, `Clone`, `Copy`

Static descriptor for one fidius interface, consumed by the Python loader to validate and dispatch into a Python plugin.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `interface_name` | `& 'static str` | Trait name, used for diagnostics only. |
| `interface_hash` | `u64` | Same hash the cdylib path baked into its `PluginDescriptor`. The
Python plugin's `__interface_hash__` constant must match this. |
| `methods` | `& 'static [PythonMethodDesc]` | Methods in declaration order — the index here lines up with the
vtable index the cdylib path uses for the same trait. The Python
loader looks up callables in this order so `call_method(i, ...)`
dispatches to the right Python function. |



### <span id="struct.PythonMethodDesc"></span>`struct PythonMethodDesc`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


**Derives:** `Debug`, `Clone`, `Copy`

One method on the interface.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `name` | `& 'static str` | Function name to look up in the Python plugin module. |
| `wire_raw` | `bool` | Whether this method uses raw byte-passthrough wire mode
(`#[wire(raw)]`). Determines whether the dispatcher routes through
`call_method_raw` (raw bytes both sides) or `call_method` (typed
args via JSON conversion). |



