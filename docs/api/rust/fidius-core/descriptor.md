# fidius-core::descriptor <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


FFI descriptor and registry types for the Fidius plugin framework.

These types form the stable C ABI contract between host and plugin.
All types use `#[repr(C)]` layout and are read directly from dylib memory.

## Structs

### `fidius-core::descriptor::PluginRegistry`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Top-level registry exported by every Fidius plugin dylib.

Each dylib exports exactly one `FIDIUS_PLUGIN_REGISTRY` static symbol
pointing to this struct. The registry contains pointers to one or more
`PluginDescriptor`s (supporting multiple plugins per dylib).

# Safety
- `descriptors` must point to a valid array of `plugin_count` pointers. - Each pointer in the array must point to a valid `PluginDescriptor`. - All pointed-to data must have `'static` lifetime (typically link-time constants).

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `magic` | `[u8 ; 8]` | Magic bytes — must equal `FIDIUS_MAGIC` (`b"FIDIUS\0\0"`). |
| `registry_version` | `u32` | Layout version of this struct. Must equal `REGISTRY_VERSION`. |
| `plugin_count` | `u32` | Number of plugin descriptors in this registry. |
| `descriptors` | `* const * const PluginDescriptor` | Pointer to an array of `plugin_count` descriptor pointers. |



### `fidius-core::descriptor::PluginDescriptor`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Metadata descriptor for a single plugin within a dylib.

Contains all information the host needs to validate and call the plugin
without executing any plugin code. All string fields are pointers to
static, null-terminated C strings embedded in the dylib.

# Safety
- `interface_name` and `plugin_name` must point to valid, null-terminated, UTF-8 C strings with `'static` lifetime. - `vtable` must point to a valid `#[repr(C)]` vtable struct matching the interface identified by `interface_name` and `interface_hash`. - When `buffer_strategy == PluginAllocated`, `free_buffer` must be `Some`. - All pointed-to data must outlive any `PluginHandle` derived from this descriptor.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `abi_version` | `u32` | Descriptor struct layout version. Must equal `ABI_VERSION`. |
| `interface_name` | `* const c_char` | Null-terminated name of the trait this plugin implements (e.g., `"ImageFilter"`). |
| `interface_hash` | `u64` | FNV-1a hash of the required method signatures. Detects ABI drift. |
| `interface_version` | `u32` | User-specified interface version from `#[plugin_interface(version = N)]`. |
| `capabilities` | `u64` | Bitfield where bit N indicates optional method N is implemented.
Supports up to 64 optional methods per interface. |
| `wire_format` | `u8` | Wire serialization format this plugin was compiled with. |
| `buffer_strategy` | `u8` | Buffer management strategy this plugin's vtable expects. |
| `plugin_name` | `* const c_char` | Null-terminated human-readable name for this plugin implementation. |
| `vtable` | `* const c_void` | Opaque pointer to the interface-specific `#[repr(C)]` vtable struct. |
| `free_buffer` | `Option < unsafe extern "C" fn (* mut u8 , usize) >` | Deallocation function for plugin-allocated buffers.
Must be `Some` when `buffer_strategy == PluginAllocated`.
The host calls this after reading output data to free the plugin's allocation. |

#### Methods

##### `interface_name_str` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>
 <span class="plissken-badge plissken-badge-unsafe" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #f44336; color: white;">unsafe</span>


```rust
unsafe fn interface_name_str (& self) -> & str
```

Read the `interface_name` field as a Rust `&str`.

# Safety
`interface_name` must point to a valid, null-terminated, UTF-8 C string that outlives the returned reference.

<details>
<summary>Source</summary>

```rust
    pub unsafe fn interface_name_str(&self) -> &str {
        let cstr = unsafe { std::ffi::CStr::from_ptr(self.interface_name) };
        cstr.to_str().expect("interface_name is not valid UTF-8")
    }
```

</details>



##### `plugin_name_str` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>
 <span class="plissken-badge plissken-badge-unsafe" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #f44336; color: white;">unsafe</span>


```rust
unsafe fn plugin_name_str (& self) -> & str
```

Read the `plugin_name` field as a Rust `&str`.

# Safety
`plugin_name` must point to a valid, null-terminated, UTF-8 C string that outlives the returned reference.

<details>
<summary>Source</summary>

```rust
    pub unsafe fn plugin_name_str(&self) -> &str {
        let cstr = unsafe { std::ffi::CStr::from_ptr(self.plugin_name) };
        cstr.to_str().expect("plugin_name is not valid UTF-8")
    }
```

</details>



##### `buffer_strategy_kind` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn buffer_strategy_kind (& self) -> BufferStrategyKind
```

Returns the `buffer_strategy` field as a `BufferStrategyKind`.

<details>
<summary>Source</summary>

```rust
    pub fn buffer_strategy_kind(&self) -> BufferStrategyKind {
        match self.buffer_strategy {
            0 => BufferStrategyKind::CallerAllocated,
            1 => BufferStrategyKind::PluginAllocated,
            2 => BufferStrategyKind::Arena,
            _ => panic!("invalid buffer_strategy value: {}", self.buffer_strategy),
        }
    }
```

</details>



##### `wire_format_kind` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn wire_format_kind (& self) -> WireFormat
```

Returns the `wire_format` field as a `WireFormat`.

<details>
<summary>Source</summary>

```rust
    pub fn wire_format_kind(&self) -> WireFormat {
        match self.wire_format {
            0 => WireFormat::Json,
            1 => WireFormat::Bincode,
            _ => panic!("invalid wire_format value: {}", self.wire_format),
        }
    }
```

</details>



##### `has_capability` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn has_capability (& self , bit : u32) -> bool
```

Check if the given optional method capability bit is set.

<details>
<summary>Source</summary>

```rust
    pub fn has_capability(&self, bit: u32) -> bool {
        assert!(bit < 64, "capability bit must be < 64");
        self.capabilities & (1u64 << bit) != 0
    }
```

</details>





### `fidius-core::descriptor::DescriptorPtr`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


A `Sync` wrapper for a raw pointer to a `PluginDescriptor`.

Used in static contexts where a `*const PluginDescriptor` needs to live
in a `static` variable (which requires `Sync`). The pointed-to descriptor
must have `'static` lifetime.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `0` | `* const PluginDescriptor` |  |



## Enums

### `fidius-core::descriptor::BufferStrategyKind` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Buffer management strategy for an interface.

Selected per-trait via `#[plugin_interface(buffer = ...)]`.
Determines the FFI function pointer signatures in the vtable.

#### Variants

- **`CallerAllocated`** - Host allocates output buffer; plugin writes into it.
Returns `-1` with needed size if buffer is too small.
- **`PluginAllocated`** - Plugin allocates output; host frees via `PluginDescriptor::free_buffer`.
- **`Arena`** - Host provides a pre-allocated arena; plugin writes into it.
Data is valid only until the next call.



### `fidius-core::descriptor::WireFormat` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Wire serialization format.

Determined at compile time via `cfg(debug_assertions)`.
Host rejects plugins compiled with a mismatched format.

#### Variants

- **`Json`** - JSON via `serde_json` — human-readable, used in debug builds.
- **`Bincode`** - bincode — compact and fast, used in release builds.



