---
title: "fidius-guest::error"
description: "Error types for the Fidius plugin framework."
sidebar:
  order: 13
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


Error types for the Fidius plugin framework.

## Structs

### <span id="struct.PluginError"></span>`struct PluginError`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


**Derives:** `Debug`, `Clone`, `Serialize`, `Deserialize`, `PartialEq`

Error returned by plugin method implementations to signal business logic failures.

Serialized as bincode across the FFI boundary. The host deserializes
this from the output buffer when the FFI shim returns `STATUS_PLUGIN_ERROR`.
The `details` field is stored as a JSON string (not `serde_json::Value`)
because bincode cannot deserialize a free-form `serde_json::Value`
(it lacks `deserialize_any`). Storing it as a `String` keeps the error
struct bincode-compatible.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `code` | `String` | Machine-readable error code (e.g., `"INVALID_INPUT"`, `"NOT_FOUND"`). |
| `message` | `String` | Human-readable error message. |
| `details` | `Option < String >` | Optional structured details as a JSON string. |

#### Methods

##### <span id="PluginError.new"></span>`new`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


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



##### <span id="PluginError.with_details"></span>`with_details`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


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



##### <span id="PluginError.details_value"></span>`details_value`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


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





