# fidius-core::host_registry <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Host-import registry assembly (plugin → host callback channel).

Each `#[fidius::host_interface]` trait submits a [`HostImportDescriptor`]
pointer via `inventory::submit!` (from the interface crate, gated to
non-wasm builds). `fidius_plugin_registry!()` emits the optional
`fidius_get_host_imports` export that collects them, so a plugin dylib
that links a host interface automatically advertises it. Plugins with no
host interfaces export an **empty** registry — and plugins built before
this channel existed export no such symbol at all; the host treats both
identically (nothing to bind).

## Structs

### `fidius-core::host_registry::HostImportEntry`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


A submitted host-import descriptor pointer, collected via `inventory`.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `descriptor` | `& 'static HostImportDescriptor` |  |



## Functions

### `fidius-core::host_registry::build_host_import_registry`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn build_host_import_registry () -> HostImportRegistry
```

Build the host-import registry from all submitted descriptors.

<details>
<summary>Source</summary>

```rust
fn build_host_import_registry() -> HostImportRegistry {
    let entries: Vec<*const HostImportDescriptor> = inventory::iter::<HostImportEntry>()
        .map(|e| e.descriptor as *const HostImportDescriptor)
        .collect();

    let count = entries.len() as u32;
    let ptr = entries.as_ptr();
    std::mem::forget(entries);

    HostImportRegistry {
        magic: FIDIUS_MAGIC,
        registry_version: HOST_IMPORTS_VERSION,
        import_count: count,
        imports: ptr,
    }
}
```

</details>



### `fidius-core::host_registry::get_host_import_registry`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn get_host_import_registry () -> & 'static HostImportRegistry
```

Get or build the host-import registry (cached after first call).

<details>
<summary>Source</summary>

```rust
pub fn get_host_import_registry() -> &'static HostImportRegistry {
    static REGISTRY: std::sync::OnceLock<HostImportRegistry> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(build_host_import_registry)
}
```

</details>



