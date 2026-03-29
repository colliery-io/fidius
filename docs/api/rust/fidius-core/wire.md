# fidius-core::wire <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Wire format serialization for Fidius plugin FFI boundary.

In debug builds (`cfg(debug_assertions)`), data is serialized as JSON for
human readability. In release builds, bincode is used for compact, fast
serialization. The `WIRE_FORMAT` constant encodes which format is active
so the host can reject mismatched plugins at load time.

## Enums

### `fidius-core::wire::WireError` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Errors that can occur during wire serialization or deserialization.

#### Variants

- **`Json`** - JSON serialization/deserialization error.
- **`Bincode`** - Bincode serialization/deserialization error.



## Functions

### `fidius-core::wire::serialize`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn serialize < T : Serialize > (val : & T) -> Result < Vec < u8 > , WireError >
```

Serialize a value using the active wire format.

Returns JSON bytes in debug builds, bincode bytes in release builds.

<details>
<summary>Source</summary>

```rust
pub fn serialize<T: Serialize>(val: &T) -> Result<Vec<u8>, WireError> {
    serde_json::to_vec(val).map_err(WireError::Json)
}
```

</details>



### `fidius-core::wire::deserialize`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn deserialize < T : DeserializeOwned > (bytes : & [u8]) -> Result < T , WireError >
```

Deserialize a value from the active wire format.

Expects JSON bytes in debug builds, bincode bytes in release builds.

<details>
<summary>Source</summary>

```rust
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WireError> {
    serde_json::from_slice(bytes).map_err(WireError::Json)
}
```

</details>



### `fidius-core::wire::serialize`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn serialize < T : Serialize > (val : & T) -> Result < Vec < u8 > , WireError >
```

Serialize a value using the active wire format.

Returns JSON bytes in debug builds, bincode bytes in release builds.

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

Deserialize a value from the active wire format.

Expects JSON bytes in debug builds, bincode bytes in release builds.

<details>
<summary>Source</summary>

```rust
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WireError> {
    bincode::deserialize(bytes).map_err(WireError::Bincode)
}
```

</details>



