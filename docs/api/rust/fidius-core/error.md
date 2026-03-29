# fidius-core::error <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Error types for the Fidius plugin framework.

## Structs

### `fidius-core::error::PluginError`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`, `Clone`, `Serialize`, `Deserialize`, `PartialEq`

Error returned by plugin method implementations to signal business logic failures.

Serialized across the FFI boundary via the wire format. The host deserializes
this from the output buffer when the FFI shim returns `STATUS_PLUGIN_ERROR`.
The `details` field is stored as a JSON string (not `serde_json::Value`)
so that it serializes correctly under both JSON and bincode wire formats.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `code` | `String` | Machine-readable error code (e.g., `"INVALID_INPUT"`, `"NOT_FOUND"`). |
| `message` | `String` | Human-readable error message. |
| `details` | `Option < String >` | Optional structured details as a JSON string. |

#### Methods

##### `new` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn new (code : impl Into < String > , message : impl Into < String >) -> Self
```

Create a new `PluginError` without details.

<details>
<summary>Source</summary>

```rust
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }
```

</details>



##### `with_details` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn with_details (code : impl Into < String > , message : impl Into < String > , details : serde_json :: Value ,) -> Self
```

Create a new `PluginError` with structured details.

The `serde_json::Value` is serialized to a JSON string for storage.

<details>
<summary>Source</summary>

```rust
    pub fn with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: Some(details.to_string()),
        }
    }
```

</details>



##### `details_value` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn details_value (& self) -> Option < serde_json :: Value >
```

Parse the `details` field back into a `serde_json::Value`.

Returns `None` if details is absent or fails to parse.

<details>
<summary>Source</summary>

```rust
    pub fn details_value(&self) -> Option<serde_json::Value> {
        self.details
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
    }
```

</details>





