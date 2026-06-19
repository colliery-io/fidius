# fidius-host::types <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Owned metadata types for loaded plugins.

## Structs

### `fidius-host::types::PluginInfo`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`, `Clone`

Owned metadata for a discovered or loaded plugin.

All data copied from FFI descriptor — no raw pointers. `capabilities` and
`buffer_strategy` are cdylib-specific concepts; for python plugins they
take their default values (0 / `PluginAllocated`) and have no runtime
meaning.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `name` | `String` | Human-readable plugin name (e.g., "BlurFilter"). |
| `interface_name` | `String` | Interface trait name (e.g., "ImageFilter"). |
| `interface_hash` | `u64` | FNV-1a hash of required method signatures. |
| `interface_version` | `u32` | User-specified interface version. |
| `capabilities` | `u64` | Capability bitfield (optional method support). Cdylib only. |
| `buffer_strategy` | `BufferStrategyKind` | Buffer management strategy. Cdylib only. |
| `runtime` | `PluginRuntimeKind` | Runtime kind. New in 0.2 — defaults to `Cdylib` for backward
compatibility with code that constructs `PluginInfo` directly. |

#### Methods

##### `is_cdylib` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn is_cdylib (& self) -> bool
```

True if this is a cdylib-backed plugin.

<details>
<summary>Source</summary>

```rust
    pub fn is_cdylib(&self) -> bool {
        matches!(self.runtime, PluginRuntimeKind::Cdylib)
    }
```

</details>



##### `is_python` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn is_python (& self) -> bool
```

True if this is a Python plugin.

<details>
<summary>Source</summary>

```rust
    pub fn is_python(&self) -> bool {
        matches!(self.runtime, PluginRuntimeKind::Python)
    }
```

</details>



##### `is_wasm` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn is_wasm (& self) -> bool
```

True if this is a WASM component plugin.

<details>
<summary>Source</summary>

```rust
    pub fn is_wasm(&self) -> bool {
        matches!(self.runtime, PluginRuntimeKind::Wasm)
    }
```

</details>





## Enums

### `fidius-host::types::PluginRuntimeKind` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Plugin runtime kind. Mirrors `fidius_core::package::PackageRuntime` and surfaces it in the host-facing `PluginInfo`. Re-exported here so host callers don't need a transitive `fidius-core` use.

#### Variants

- **`Cdylib`** - Cdylib + `PluginRegistry` (the original fidius substrate).
- **`Python`** - `.py` package loaded via `fidius-python`'s embedded interpreter.
Only produced when the `python` feature is enabled on `fidius-host`.
- **`Wasm`** - Signed `.wasm` **component** (Component Model + WIT). Surfaced by
discovery; the loader (`WasmComponentExecutor`) lands in FIDIUS-I-0021
Phase 2. Reserved now so routing and `PluginInfo` model all three
backends.



### `fidius-host::types::LoadPolicy` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Controls how strictly the host validates plugins.

#### Variants

- **`Strict`** - Reject any validation failure, require signatures if configured.
- **`Lenient`** - Warn on unsigned plugins but allow loading.



