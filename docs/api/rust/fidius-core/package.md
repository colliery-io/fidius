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
| `extension` | `Option < String >` | Custom file extension for `.fid` archives (e.g., `"cloacina"`).
Defaults to `"fid"` when absent. |

#### Methods

##### `extension` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn extension (& self) -> & str
```

Returns the package extension, defaulting to `"fid"`.

<details>
<summary>Source</summary>

```rust
    pub fn extension(&self) -> &str {
        self.extension.as_deref().unwrap_or("fid")
    }
```

</details>





### `fidius-core::package::PackResult`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`

Result of packing a package, including any warnings.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `path` | `PathBuf` | Path to the created `.fid` archive. |
| `unsigned` | `bool` | Whether the package was unsigned (no `package.sig` found). |



## Enums

### `fidius-core::package::PackageError` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Errors that can occur when loading a package manifest.

#### Variants

- **`ManifestNotFound`** - The `package.toml` file was not found in the given directory.
- **`ParseError`** - The manifest file could not be parsed as valid TOML or failed
schema validation (the `[metadata]` section didn't match `M`).
- **`Io`** - An I/O error occurred reading the manifest file.
- **`BuildFailed`** - Build failed.
- **`SignatureNotFound`** - Package signature file not found.
- **`SignatureInvalid`** - Package signature is invalid (no trusted key verified it).
- **`ArchiveError`** - An error occurred creating or reading an archive.
- **`InvalidArchive`** - The archive does not contain a valid package.



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
fn load_manifest_untyped (dir : & Path) -> Result < PackageManifest < toml :: Value > , PackageError >
```

Load a manifest validating only the fixed header (accepting any metadata).

Uses `toml::Value` as the metadata type so any `[metadata]` section is accepted.
Useful for CLI tools that validate structure without knowing the host's schema.

<details>
<summary>Source</summary>

```rust
pub fn load_manifest_untyped(dir: &Path) -> Result<PackageManifest<toml::Value>, PackageError> {
    load_manifest::<toml::Value>(dir)
}
```

</details>



### `fidius-core::package::package_digest`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn package_digest (dir : & Path) -> Result < [u8 ; 32] , PackageError >
```

Compute a deterministic SHA-256 digest over all package source files.

Walks the package directory, collects all files (excluding `target/`,
`.git/`, and `*.sig` files), sorts by relative path, and feeds each
file's relative path and contents into a SHA-256 hasher.
The resulting 32-byte digest covers the entire package contents.
Sign this digest to protect against tampering.

<details>
<summary>Source</summary>

```rust
pub fn package_digest(dir: &Path) -> Result<[u8; 32], PackageError> {
    use sha2::{Digest, Sha256};

    let mut files = Vec::new();
    collect_files(dir, dir, &mut files)?;
    files.sort();

    let mut hasher = Sha256::new();
    for rel_path in &files {
        let abs_path = dir.join(rel_path);
        let contents = std::fs::read(&abs_path)?;
        // Hash the relative path (as UTF-8 bytes) then the file contents.
        // Length-prefix both to prevent ambiguity.
        let path_bytes = rel_path.as_bytes();
        hasher.update((path_bytes.len() as u64).to_le_bytes());
        hasher.update(path_bytes);
        hasher.update((contents.len() as u64).to_le_bytes());
        hasher.update(&contents);
    }

    Ok(hasher.finalize().into())
}
```

</details>



### `fidius-core::package::collect_files`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn collect_files (root : & Path , dir : & Path , out : & mut Vec < String >) -> Result < () , PackageError >
```

Recursively collect file paths relative to `root`, skipping excluded dirs/files.

<details>
<summary>Source</summary>

```rust
fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), PackageError> {
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip excluded directories
        if path.is_dir() {
            if name_str == "target" || name_str == ".git" {
                continue;
            }
            collect_files(root, &path, out)?;
            continue;
        }

        // Skip signature files
        if name_str.ends_with(".sig") {
            continue;
        }

        // Store relative path using forward slashes for cross-platform determinism
        let rel = path
            .strip_prefix(root)
            .expect("path is under root")
            .to_string_lossy()
            .replace('\\', "/");
        out.push(rel);
    }
    Ok(())
}
```

</details>



### `fidius-core::package::collect_archive_files`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn collect_archive_files (root : & Path , dir : & Path , out : & mut Vec < String > ,) -> Result < () , PackageError >
```

Recursively collect file paths for archiving (includes `.sig` files).

<details>
<summary>Source</summary>

```rust
fn collect_archive_files(
    root: &Path,
    dir: &Path,
    out: &mut Vec<String>,
) -> Result<(), PackageError> {
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if path.is_dir() {
            if name_str == "target" || name_str == ".git" {
                continue;
            }
            collect_archive_files(root, &path, out)?;
            continue;
        }

        let rel = path
            .strip_prefix(root)
            .expect("path is under root")
            .to_string_lossy()
            .replace('\\', "/");
        out.push(rel);
    }
    Ok(())
}
```

</details>



### `fidius-core::package::pack_package`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn pack_package (dir : & Path , output : Option < & Path >) -> Result < PackResult , PackageError >
```

Create a `.fid` archive (tar + bzip2) from a package directory.

The archive contains a single top-level directory `{name}-{version}/`
with all source files. Excludes `target/` and `.git/` directories.
Includes `package.sig` if present.
If `output` is `None`, the archive is written to the current directory
as `{name}-{version}.fid`.

<details>
<summary>Source</summary>

```rust
pub fn pack_package(dir: &Path, output: Option<&Path>) -> Result<PackResult, PackageError> {
    use bzip2::write::BzEncoder;
    use bzip2::Compression;

    let manifest = load_manifest_untyped(dir)?;
    let pkg = &manifest.package;
    let prefix = format!("{}-{}", pkg.name, pkg.version);
    let ext = pkg.extension();

    let unsigned = !dir.join("package.sig").exists();

    let out_path = match output {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from(format!("{prefix}.{ext}")),
    };

    let file = std::fs::File::create(&out_path).map_err(|e| {
        PackageError::ArchiveError(format!("failed to create {}: {e}", out_path.display()))
    })?;

    let encoder = BzEncoder::new(file, Compression::best());
    let mut tar = tar::Builder::new(encoder);

    let mut files = Vec::new();
    collect_archive_files(dir, dir, &mut files)?;
    files.sort();

    for rel_path in &files {
        let abs_path = dir.join(rel_path);
        let archive_path = format!("{prefix}/{rel_path}");
        tar.append_path_with_name(&abs_path, &archive_path)
            .map_err(|e| PackageError::ArchiveError(format!("failed to add {rel_path}: {e}")))?;
    }

    tar.into_inner()
        .map_err(|e| PackageError::ArchiveError(format!("failed to finish bz2 stream: {e}")))?
        .finish()
        .map_err(|e| PackageError::ArchiveError(format!("failed to finish bz2 stream: {e}")))?;

    Ok(PackResult {
        path: out_path,
        unsigned,
    })
}
```

</details>



### `fidius-core::package::unpack_package`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn unpack_package (archive : & Path , dest : & Path) -> Result < PathBuf , PackageError >
```

Extract a `.fid` archive (tar + bzip2) to a destination directory.

Returns the path to the extracted top-level package directory.
Validates that a `package.toml` exists in the extracted contents.

<details>
<summary>Source</summary>

```rust
pub fn unpack_package(archive: &Path, dest: &Path) -> Result<PathBuf, PackageError> {
    use bzip2::read::BzDecoder;

    let file = std::fs::File::open(archive).map_err(|e| {
        PackageError::ArchiveError(format!("failed to open {}: {e}", archive.display()))
    })?;

    let decoder = BzDecoder::new(file);
    let mut tar = tar::Archive::new(decoder);

    tar.unpack(dest).map_err(|e| {
        PackageError::ArchiveError(format!("failed to extract {}: {e}", archive.display()))
    })?;

    // Find the top-level directory that was extracted
    let entries = std::fs::read_dir(dest).map_err(PackageError::Io)?;
    let mut pkg_dir: Option<PathBuf> = None;
    for entry in entries {
        let entry = entry.map_err(PackageError::Io)?;
        let path = entry.path();
        if path.is_dir() && path.join("package.toml").exists() {
            pkg_dir = Some(path);
            break;
        }
    }

    let pkg_dir = pkg_dir.ok_or_else(|| {
        PackageError::InvalidArchive("archive does not contain a package.toml".to_string())
    })?;

    Ok(pkg_dir)
}
```

</details>



