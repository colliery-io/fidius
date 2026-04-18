# fidius-core::descriptor <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


FFI descriptor and registry types for the Fidius plugin framework.

These types form the stable C ABI contract between host and plugin.
All types use `#[repr(C)]` layout and are read directly from dylib memory.

## Structs

### `fidius-core::descriptor::MetaKv`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Static key/value pair for method-level or trait-level metadata.

Both `key` and `value` point to null-terminated UTF-8 C strings with
`'static` lifetime (typically string literals embedded in the plugin's
`.rodata`). Fidius treats values as opaque — hosts define conventions
via their own metadata schemas. See ADR/spec for the `fidius.*`
reserved namespace.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `key` | `* const c_char` | Null-terminated UTF-8 key. Never null. |
| `value` | `* const c_char` | Null-terminated UTF-8 value. Never null (may be empty string). |



### `fidius-core::descriptor::MethodMetaEntry`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Per-method metadata entry. One entry per method in declaration order, stored in the array referenced by `PluginDescriptor::method_metadata`.

Methods with no `#[method_meta(...)]` annotations have `kvs: null` and
`kv_count: 0` — the entry exists but is empty, so hosts can index
uniformly by method index.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `kvs` | `* const MetaKv` | Pointer to an array of `kv_count` `MetaKv` entries, or null if this
method has no metadata. |
| `kv_count` | `u32` | Number of key/value pairs for this method. Zero when `kvs` is null. |



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
| `descriptor_size` | `u32` | Size in bytes of this descriptor struct at plugin build time.

The host reads this field FIRST (it's at offset 0) before trusting any
other offset calculation. Any field whose offset is >= `descriptor_size`
is not present in this plugin's build — the plugin was compiled against
an older fidius version that didn't have that field yet.

Enables post-1.0 minor releases to add new fields at the end of this
struct without breaking older plugins. See ADR-0002. |
| `abi_version` | `u32` | Descriptor struct layout version. Must equal `ABI_VERSION`. |
| `interface_name` | `* const c_char` | Null-terminated name of the trait this plugin implements (e.g., `"ImageFilter"`). |
| `interface_hash` | `u64` | FNV-1a hash of the required method signatures. Detects ABI drift. |
| `interface_version` | `u32` | User-specified interface version from `#[plugin_interface(version = N)]`. |
| `capabilities` | `u64` | Bitfield where bit N indicates optional method N is implemented.
Supports up to 64 optional methods per interface. |
| `buffer_strategy` | `u8` | Buffer management strategy this plugin's vtable expects. |
| `plugin_name` | `* const c_char` | Null-terminated human-readable name for this plugin implementation. |
| `vtable` | `* const c_void` | Opaque pointer to the interface-specific `#[repr(C)]` vtable struct. |
| `free_buffer` | `Option < unsafe extern "C" fn (* mut u8 , usize) >` | Deallocation function for plugin-allocated buffers.
Must be `Some` when `buffer_strategy == PluginAllocated`.
The host calls this after reading output data to free the plugin's allocation. |
| `method_count` | `u32` | Total number of methods in the vtable (required + optional).
Used for bounds checking in `call_method`. |
| `method_metadata` | `* const MethodMetaEntry` | Pointer to an array of `method_count` `MethodMetaEntry` structs,
one per method in declaration order. Each entry may be empty
(kvs=null, kv_count=0) if the method declared no metadata.

Null if the interface used no `#[method_meta(...)]` annotations
at all (optimization for the common case). |
| `trait_metadata` | `* const MetaKv` | Pointer to an array of `trait_metadata_count` `MetaKv` entries for
trait-level metadata (declared via `#[trait_meta(...)]`).

Null if no trait-level metadata was declared. |
| `trait_metadata_count` | `u32` | Number of entries in `trait_metadata`. Zero when `trait_metadata`
is null. |

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
fn buffer_strategy_kind (& self) -> Result < BufferStrategyKind , u8 >
```

Returns the `buffer_strategy` field as a `BufferStrategyKind`.

Returns `Err(value)` if the discriminant is unknown. This can happen
with malformed plugins — callers should reject rather than panic.

<details>
<summary>Source</summary>

```rust
    pub fn buffer_strategy_kind(&self) -> Result<BufferStrategyKind, u8> {
        match self.buffer_strategy {
            1 => Ok(BufferStrategyKind::PluginAllocated),
            2 => Ok(BufferStrategyKind::Arena),
            v => Err(v),
        }
    }
```

</details>



##### `has_capability` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn has_capability (& self , bit : u32) -> bool
```

Check if the given optional method capability bit is set.

Returns `false` for bit indices >= 64 rather than panicking.

<details>
<summary>Source</summary>

```rust
    pub fn has_capability(&self, bit: u32) -> bool {
        if bit >= 64 {
            return false;
        }
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
Discriminant value `0` is reserved (previously `CallerAllocated`, removed
in 0.1.0 — its value proposition was subsumed by `PluginAllocated`).

#### Variants

- **`PluginAllocated`** - Plugin allocates output; host frees via `PluginDescriptor::free_buffer`.
VTable fns: `(in_ptr, in_len, out_ptr, out_len) -> i32`.
- **`Arena`** - Host provides a pre-allocated arena buffer; plugin writes its serialized
output into the buffer. Returns `STATUS_BUFFER_TOO_SMALL` (with needed
size written to `out_len`) if the arena is too small; host grows and
retries. Data is valid only until the next call.

VTable fns: `(in_ptr, in_len, arena_ptr, arena_cap, out_offset, out_len) -> i32`.



## Functions

### `fidius-core::descriptor::parse_u32_const`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
const fn parse_u32_const (s : & str) -> u32
```

<details>
<summary>Source</summary>

```rust
const fn parse_u32_const(s: &str) -> u32 {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut n = 0u32;
    while i < bytes.len() {
        n = n * 10 + (bytes[i] - b'0') as u32;
        i += 1;
    }
    n
}
```

</details>



