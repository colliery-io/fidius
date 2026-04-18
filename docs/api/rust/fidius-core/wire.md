# fidius-core::wire <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Wire format serialization for Fidius plugin FFI boundary.

Fidius uses bincode as the single wire format for all FFI data. Prior to
0.1.0 the format varied by build profile (JSON in debug, bincode in
release) — that was removed because profile-mixed host/plugin load
rejections caused repeated dev-loop friction with no real inspection
benefit to offset them.

## Enums

### `fidius-core::wire::WireError` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Errors that can occur during wire serialization or deserialization.

#### Variants

- **`Bincode`** - Bincode serialization/deserialization error.



## Functions

### `fidius-core::wire::serialize`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


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



### `fidius-core::wire::deserialize`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


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



