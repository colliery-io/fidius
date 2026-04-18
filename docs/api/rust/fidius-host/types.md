# fidius-host::types <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Owned metadata types for loaded plugins.

## Structs

### `fidius-host::types::PluginInfo`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`, `Clone`

Owned metadata for a discovered or loaded plugin.

All data copied from FFI descriptor — no raw pointers.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `name` | `String` | Human-readable plugin name (e.g., "BlurFilter"). |
| `interface_name` | `String` | Interface trait name (e.g., "ImageFilter"). |
| `interface_hash` | `u64` | FNV-1a hash of required method signatures. |
| `interface_version` | `u32` | User-specified interface version. |
| `capabilities` | `u64` | Capability bitfield (optional method support). |
| `buffer_strategy` | `BufferStrategyKind` | Buffer management strategy. |



## Enums

### `fidius-host::types::LoadPolicy` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Controls how strictly the host validates plugins.

#### Variants

- **`Strict`** - Reject any validation failure, require signatures if configured.
- **`Lenient`** - Warn on unsigned plugins but allow loading.



