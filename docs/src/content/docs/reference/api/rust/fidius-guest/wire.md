---
title: "fidius-guest::wire"
description: "Wire format serialization for Fidius plugin FFI boundary."
sidebar:
  order: 26
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


Wire format serialization for Fidius plugin FFI boundary.

Fidius uses bincode as the single wire format for all FFI data. Prior to
0.1.0 the format varied by build profile (JSON in debug, bincode in
release) — that was removed because profile-mixed host/plugin load
rejections caused repeated dev-loop friction with no real inspection
benefit to offset them.

## Enums

### <span id="enum.WireError"></span>`enum WireError`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


Errors that can occur during wire serialization or deserialization.

#### Variants

- **`Bincode`** - Bincode serialization/deserialization error.



## Functions

### <span id="fn.serialize"></span>`fn serialize`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


```rust
fn serialize < T : Serialize > (val : & T) -> Result < Vec < u8 > , WireError >
```

Serialize a value as bincode for transport across the FFI boundary.

<details>
<summary>Source</summary>

```rust
pub fn serialize<T: Serialize>(val: &T) -> Result<Vec<u8>, WireError> {
    bincode::serialize(val).map_err(WireError::Bincode)
}
```

</details>



### <span id="fn.deserialize"></span>`fn deserialize`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


```rust
fn deserialize < T : DeserializeOwned > (bytes : & [u8]) -> Result < T , WireError >
```

Deserialize a value from bincode bytes received across the FFI boundary.

<details>
<summary>Source</summary>

```rust
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WireError> {
    bincode::deserialize(bytes).map_err(WireError::Bincode)
}
```

</details>



### <span id="fn.serialized_size"></span>`fn serialized_size`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


```rust
fn serialized_size < T : Serialize > (val : & T) -> Result < u64 , WireError >
```

The exact serialized size of `val` in bytes, without allocating. Matches [`serialize`]'s output length. Used to size/validate a buffer before [`serialize_into`] (e.g. the cdylib streaming arena path, FIDIUS-T-0138).

<details>
<summary>Source</summary>

```rust
pub fn serialized_size<T: Serialize>(val: &T) -> Result<u64, WireError> {
    bincode::serialized_size(val).map_err(WireError::Bincode)
}
```

</details>



### <span id="fn.serialize_into"></span>`fn serialize_into`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


```rust
fn serialize_into < T : Serialize > (buf : & mut [u8] , val : & T) -> Result < () , WireError >
```

Serialize `val` directly into a caller-provided buffer — no intermediate `Vec` allocation. `buf` must be at least [`serialized_size`] bytes (bincode errors otherwise). Byte-identical to [`serialize`].

<details>
<summary>Source</summary>

```rust
pub fn serialize_into<T: Serialize>(buf: &mut [u8], val: &T) -> Result<(), WireError> {
    bincode::serialize_into(buf, val).map_err(WireError::Bincode)
}
```

</details>



