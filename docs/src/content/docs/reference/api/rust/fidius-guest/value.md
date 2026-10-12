---
title: "fidius-guest::value"
description: "`Value` — a neutral, self-describing value tree for the plugin boundary."
editUrl: false
sidebar:
  order: 23
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

`Value` — a neutral, self-describing value tree for the plugin boundary.

Every fidius execution backend dispatches typed method calls through a
single `PluginExecutor::call(method, Value) -> Value` seam (see
`fidius-host`). `Value` is the lingua franca each backend maps to its
native representation:

- **cdylib**: `Value` → bincode → vtable FFI → bincode → `Value`
- **python**: `Value` → `PyObject` → call → `PyObject` → `Value`
- **wasm**: `Value` → `wasmtime` `component::Val` (Canonical ABI) → `Value`

The host's generic `call_method<I, O>` stays caller-identical by going
`I → Value` ([`to_value`](../../../rust/fidius-guest/value/#fn.to_value)) then `Value → O` ([`from_value`](../../../rust/fidius-guest/value/#fn.from_value)) around the
executor.

**Layering rule:** `Value` lives in `fidius-core` and is deliberately
free of any backend dependency. Only the WASM executor (Phase 2) maps it
to `wasmtime::component::Val`; cdylib and Python never see wasmtime.

The variant set mirrors the WebAssembly Component Model value space so the
Phase-2 mapping is mechanical: distinct signed/unsigned integer widths,
`f32`/`f64`, `char`, `string`, `list<u8>` as [`Value::Bytes`](../../../rust/fidius-guest/value/#enum.Value), lists,
records, options, and variants.

## Structs

### <span id="struct.ValueError"></span><code style="font-size: 0.85em">struct ValueError</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

**Derives:** `Debug`, `thiserror::Error`

Error produced while converting to or from [`Value`](../../../rust/fidius-guest/value/#enum.Value).

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `0` | `String` |  |

## Enums

### <span id="enum.Value"></span><code style="font-size: 0.85em">enum Value</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

A self-describing value crossing the plugin-call boundary.

Construct one from any `Serialize` type with [`to_value`](../../../rust/fidius-guest/value/#fn.to_value) and read it back
into any `DeserializeOwned` type with [`from_value`](../../../rust/fidius-guest/value/#fn.from_value).

#### Variants

- **`Bool`** - Boolean.
- **`S8`** - Signed 8-bit integer.
- **`S16`** - Signed 16-bit integer.
- **`S32`** - Signed 32-bit integer.
- **`S64`** - Signed 64-bit integer.
- **`U8`** - Unsigned 8-bit integer.
- **`U16`** - Unsigned 16-bit integer.
- **`U32`** - Unsigned 32-bit integer.
- **`U64`** - Unsigned 64-bit integer.
- **`F32`** - 32-bit float.
- **`F64`** - 64-bit float.
- **`Char`** - Unicode scalar value.
- **`String`** - UTF-8 string.
- **`Bytes`** - Opaque byte string (`list<u8>` in WIT terms).
- **`Option`** - Optional value (`none`/`some`).
- **`List`** - Ordered sequence — serde seqs, tuples, and tuple structs land here.
- **`Record`** - Named fields — serde structs and string-keyed maps land here.
  Field order is preserved (insertion order).
- **`Map`** - General key/value map for non-string keys.
- **`Variant`** - A tagged enum case. `value` carries the payload: [`Value::Unit`](../../../rust/fidius-guest/value/#enum.Value) for a
  unit variant, the inner value for a newtype variant, a [`Value::List`](../../../rust/fidius-guest/value/#enum.Value)
  for a tuple variant, or a [`Value::Record`](../../../rust/fidius-guest/value/#enum.Value) for a struct variant.
- **`Unit`** - The unit value — serde `()` and unit structs.

## Functions

### <span id="fn.to_value"></span><code style="font-size: 0.85em">fn to_​value</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn to_value<T: Serialize>(value: &T) -> Result<Value, ValueError>
```

Convert any `Serialize` type into a [`Value`](../../../rust/fidius-guest/value/#enum.Value).

<details>
<summary>Source</summary>

```rust
pub fn to_value<T: Serialize>(value: &T) -> Result<Value, ValueError> {
    value.serialize(ValueSerializer)
}
```

</details>

### <span id="fn.from_value"></span><code style="font-size: 0.85em">fn from_​value</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn from_value<T>(value: Value) -> Result<T, ValueError>
where
    T: de::DeserializeOwned,
```

Convert a [`Value`](../../../rust/fidius-guest/value/#enum.Value) into any `Deserialize` type.

<details>
<summary>Source</summary>

```rust
pub fn from_value<T>(value: Value) -> Result<T, ValueError>
where
    T: de::DeserializeOwned,
{
    T::deserialize(value)
}
```

</details>
