# fidius-core::package <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Source package manifest types and parsing.

A package is a directory containing plugin source code and a `package.toml`
manifest. The manifest has a fixed header (name, version, interface) and
an extensible `[metadata]` section validated via serde against a
host-defined schema type.

## Structs

### `fidius-core::package::PackageManifest`<M>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`, `Clone`, `Serialize`, `Deserialize`

A parsed package manifest, generic over the host-defined metadata schema.

The `M` type parameter is the host's metadata schema. If the `[metadata]`
section of `package.toml` doesn't deserialize into `M`, parsing fails —
this is how schema validation works.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `package` | `PackageHeader` | Fixed header fields required by fidius. |
| `dependencies` | `BTreeMap < String , String >` | Dependencies on other packages (name → version requirement). |
| `metadata` | `M` | Host-defined metadata. Must deserialize from the `[metadata]` section. |



### `fidius-core::package::PackageHeader`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`, `Clone`, `Serialize`, `Deserialize`

Fixed header fields that every package manifest must have.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `name` | `String` | Package name (e.g., `"blur-filter"`). |
| `version` | `String` | Package version (e.g., `"1.2.0"`). |
| `interface` | `String` | Name of the interface crate this package implements. |
| `interface_version` | `u32` | Expected interface version. |
| `source_hash` | `Option < String >` | Optional SHA-256 hash of the source directory contents. |



## Enums

### `fidius-core::package::PackageError` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Errors that can occur when loading a package manifest.

#### Variants

- **`ManifestNotFound`** - The `package.toml` file was not found in the given directory.
- **`ParseError`** - The manifest file could not be parsed as valid TOML or failed
schema validation (the `[metadata]` section didn't match `M`).
- **`Io`** - An I/O error occurred reading the manifest file.
- **`BuildFailed`** - Build failed.



## Functions

### `fidius-core::package::load_manifest`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn load_manifest < M : DeserializeOwned > (dir : & Path) -> Result < PackageManifest < M > , PackageError >
```

Load and parse a `package.toml` manifest from a package directory.

The type parameter `M` is the host's metadata schema. If the `[metadata]`
section doesn't deserialize into `M`, this returns `PackageError::ParseError`.

**Examples:**

```ignore
#[derive(Deserialize)]
struct MySchema {
    category: String,
    min_host_version: String,
}

let manifest = load_manifest::<MySchema>(Path::new("./my-package/"))?;
println!("Package: {} v{}", manifest.package.name, manifest.package.version);
println!("Category: {}", manifest.metadata.category);
```

<details>
<summary>Source</summary>

```rust
pub fn load_manifest<M: DeserializeOwned>(dir: &Path) -> Result<PackageManifest<M>, PackageError> {
    let manifest_path = dir.join("package.toml");

    if !manifest_path.exists() {
        return Err(PackageError::ManifestNotFound {
            path: dir.display().to_string(),
        });
    }

    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest: PackageManifest<M> = toml::from_str(&content)?;
    Ok(manifest)
}
```

</details>



### `fidius-core::package::load_manifest_untyped`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn load_manifest_untyped (dir : & Path ,) -> Result < PackageManifest < toml :: Value > , PackageError >
```

Load a manifest validating only the fixed header (accepting any metadata).

Uses `toml::Value` as the metadata type so any `[metadata]` section is accepted.
Useful for CLI tools that validate structure without knowing the host's schema.

<details>
<summary>Source</summary>

```rust
pub fn load_manifest_untyped(
    dir: &Path,
) -> Result<PackageManifest<toml::Value>, PackageError> {
    load_manifest::<toml::Value>(dir)
}
```

</details>



