---
title: "fidius-guest::value"
description: "`Value` — a neutral, self-describing value tree for the plugin boundary."
sidebar:
  order: 24
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
`I → Value` ([`to_value`]) then `Value → O` ([`from_value`]) around the
executor.
**Layering rule:** `Value` lives in `fidius-core` and is deliberately
free of any backend dependency. Only the WASM executor (Phase 2) maps it
to `wasmtime::component::Val`; cdylib and Python never see wasmtime.
The variant set mirrors the WebAssembly Component Model value space so the
Phase-2 mapping is mechanical: distinct signed/unsigned integer widths,
`f32`/`f64`, `char`, `string`, `list<u8>` as [`Value::Bytes`], lists,
records, options, and variants.

## Structs

### <span id="struct.ValueError"></span>`struct ValueError`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


**Derives:** `Debug`, `thiserror :: Error`

Error produced while converting to or from [`Value`].

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `0` | `String` |  |



### <span id="struct.ValueSerializer"></span>`struct ValueSerializer`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>




### <span id="struct.SeqSerializer"></span>`struct SeqSerializer`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


#### Fields

| Name | Type | Description |
|------|------|-------------|
| `items` | `Vec < Value >` |  |



### <span id="struct.TupleVariantSerializer"></span>`struct TupleVariantSerializer`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


#### Fields

| Name | Type | Description |
|------|------|-------------|
| `name` | `String` |  |
| `items` | `Vec < Value >` |  |



### <span id="struct.MapSerializer"></span>`struct MapSerializer`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


#### Fields

| Name | Type | Description |
|------|------|-------------|
| `entries` | `Vec < (Value , Value) >` |  |
| `next_key` | `Option < Value >` |  |



### <span id="struct.StructSerializer"></span>`struct StructSerializer`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


#### Fields

| Name | Type | Description |
|------|------|-------------|
| `fields` | `Vec < (String , Value) >` |  |



### <span id="struct.StructVariantSerializer"></span>`struct StructVariantSerializer`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


#### Fields

| Name | Type | Description |
|------|------|-------------|
| `name` | `String` |  |
| `fields` | `Vec < (String , Value) >` |  |



### <span id="struct.SeqAccess"></span>`struct SeqAccess`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


#### Fields

| Name | Type | Description |
|------|------|-------------|
| `iter` | `std :: vec :: IntoIter < Value >` |  |



### <span id="struct.RecordAccess"></span>`struct RecordAccess`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


#### Fields

| Name | Type | Description |
|------|------|-------------|
| `iter` | `std :: vec :: IntoIter < (String , Value) >` |  |
| `value` | `Option < Value >` |  |



### <span id="struct.MapAccess"></span>`struct MapAccess`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


#### Fields

| Name | Type | Description |
|------|------|-------------|
| `iter` | `std :: vec :: IntoIter < (Value , Value) >` |  |
| `value` | `Option < Value >` |  |



### <span id="struct.SingletonMapAccess"></span>`struct SingletonMapAccess`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


Presents a `Value::Variant` as a single-entry map for `deserialize_any` consumers (e.g. deserializing into `serde_json::Value`).

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `key` | `Option < String >` |  |
| `value` | `Option < Value >` |  |



### <span id="struct.EnumAccess"></span>`struct EnumAccess`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


#### Fields

| Name | Type | Description |
|------|------|-------------|
| `name` | `String` |  |
| `value` | `Value` |  |



### <span id="struct.VariantAccess"></span>`struct VariantAccess`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


#### Fields

| Name | Type | Description |
|------|------|-------------|
| `value` | `Value` |  |



## Enums

### <span id="enum.Value"></span>`enum Value`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


A self-describing value crossing the plugin-call boundary.

Construct one from any `Serialize` type with [`to_value`] and read it back
into any `DeserializeOwned` type with [`from_value`].

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
- **`Variant`** - A tagged enum case. `value` carries the payload: [`Value::Unit`] for a
unit variant, the inner value for a newtype variant, a [`Value::List`]
for a tuple variant, or a [`Value::Record`] for a struct variant.
- **`Unit`** - The unit value — serde `()` and unit structs.



## Functions

### <span id="fn.to_value"></span>`fn to_value`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


```rust
fn to_value < T : Serialize > (value : & T) -> Result < Value , ValueError >
```

Convert any [`Serialize`] type into a [`Value`].

<details>
<summary>Source</summary>

```rust
pub fn to_value<T: Serialize>(value: &T) -> Result<Value, ValueError> {
    value.serialize(ValueSerializer)
}
```

</details>



### <span id="fn.from_value"></span>`fn from_value`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


```rust
fn from_value < T > (value : Value) -> Result < T , ValueError > where T : de :: DeserializeOwned ,
```

Convert a [`Value`] into any [`Deserialize`] type.

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



