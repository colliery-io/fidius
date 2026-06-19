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
| `python` | `Option < PythonPackageMeta >` | Python-runtime fields. Required when `package.runtime == "python"`,
rejected otherwise. Validated by [`PackageManifest::validate_runtime`]
after deserialization, since serde alone can't enforce cross-section
invariants. |
| `wasm` | `Option < WasmPackageMeta >` | WASM-component fields. Required when `package.runtime == "wasm"`,
rejected otherwise. Validated by [`PackageManifest::validate_runtime`]. |



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
| `runtime` | `Option < String >` | Plugin runtime. `"rust"` (default) → cdylib; `"python"` → Python package
loaded by `fidius-python`. Unknown values are rejected at validation
time (see [`PackageManifest::validate_runtime`]). |

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



##### `runtime` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn runtime (& self) -> PackageRuntime
```

Returns the runtime kind, defaulting to `Rust` when absent. Returns `PackageRuntime::Rust` for unknown values; callers that need to reject unknown runtimes should use [`Self::runtime_strict`].

<details>
<summary>Source</summary>

```rust
    pub fn runtime(&self) -> PackageRuntime {
        match self.runtime.as_deref() {
            None | Some("rust") => PackageRuntime::Rust,
            Some("python") => PackageRuntime::Python,
            Some("wasm") => PackageRuntime::Wasm,
            // Unknown values fall back to Rust for `runtime()`, but the
            // strict validator rejects them. Keep the lenient form so display
            // code never panics on an unfamiliar manifest.
            _ => PackageRuntime::Rust,
        }
    }
```

</details>



##### `runtime_strict` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn runtime_strict (& self) -> Result < PackageRuntime , PackageError >
```

Returns the runtime kind, erroring on unknown values.

<details>
<summary>Source</summary>

```rust
    pub fn runtime_strict(&self) -> Result<PackageRuntime, PackageError> {
        match self.runtime.as_deref() {
            None | Some("rust") => Ok(PackageRuntime::Rust),
            Some("python") => Ok(PackageRuntime::Python),
            Some("wasm") => Ok(PackageRuntime::Wasm),
            Some(other) => Err(PackageError::InvalidManifest(format!(
                "unknown runtime '{other}': allowed values are \"rust\", \"python\", \"wasm\""
            ))),
        }
    }
```

</details>





### `fidius-core::package::PythonPackageMeta`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`, `Clone`, `Serialize`, `Deserialize`

Fields under the `[python]` section of `package.toml`. Required when `package.runtime == "python"`, rejected otherwise.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `entry_module` | `String` | Python module the loader imports first. Dotted-path form (e.g.
`"my_plugin.entry"`) corresponding to a file inside the package
directory or its `vendor/` tree. |
| `requirements` | `Option < String >` | Path to the requirements file consumed by `fidius pack` to vendor
dependencies into `vendor/`. Defaults to `"requirements.txt"`. |

#### Methods

##### `requirements_path` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn requirements_path (& self) -> & str
```

Returns the requirements file path, defaulting to `"requirements.txt"`.

<details>
<summary>Source</summary>

```rust
    pub fn requirements_path(&self) -> &str {
        self.requirements.as_deref().unwrap_or("requirements.txt")
    }
```

</details>





### `fidius-core::package::WasmPackageMeta`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`, `Clone`, `Serialize`, `Deserialize`

Fields under the `[wasm]` section of `package.toml`. Required when `package.runtime == "wasm"`, rejected otherwise.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `component` | `String` | Component filename inside the package directory (e.g. `"plugin.wasm"`).
A WIT component, not a core module. |
| `precompiled` | `Option < String >` | Optional precompiled `.cwasm` (produced at pack time by the wasmtime
engine; engine/version-specific). When present and valid, the loader
uses the AOT fast path instead of JIT-compiling `component`. |
| `capabilities` | `Vec < String >` | WASI capability allow-list (e.g. `["clocks", "random", "sockets"]`).
Empty = deny-all sandbox. Consumed by the capability policy (T-0104);
filesystem is never granted in v1. |



### `fidius-core::package::UnpackOptions`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`, `Clone`

Options controlling archive extraction safety limits.

Construct with `UnpackOptions::default()` for strict defaults suitable for
untrusted input. Override individual fields for known-trusted archives that
legitimately exceed the default caps (e.g. packages that vendor large
native dependencies).

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `max_decompressed` | `u64` | Maximum total declared uncompressed size of all entries, in bytes.
Archives exceeding this are rejected as potential decompression bombs. |
| `max_ratio` | `u64` | Maximum ratio of total declared uncompressed size to compressed
archive size. Archives exceeding this are rejected. |
| `max_entries` | `u32` | Maximum number of entries in the archive. Guards against archives
that exhaust inodes or directory-entry limits via tiny-file spam. |



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

### `fidius-core::package::PackageRuntime` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Plugin runtime kind. Determines which loader the host's `PluginHost` dispatches to.

#### Variants

- **`Rust`** - Default. Plugin is a cdylib + `PluginRegistry`. Loaded by the existing
dylib loader in `fidius-host`.
- **`Python`** - Plugin is a directory of `.py` files (+ optional `vendor/`) loaded by
`fidius-python` via an embedded interpreter. Requires the host crate
to enable the `python` feature.
- **`Wasm`** - Plugin is a signed `.wasm` **component** (Component Model + WIT),
loaded by the `WasmComponentExecutor`. Reserved by FIDIUS-I-0021 Phase 1;
the loader lands in Phase 2 (until then, loading a wasm package errors
clearly rather than silently falling back to rust).



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
- **`InvalidManifest`** - Manifest passed serde parsing but failed cross-section validation
(e.g. `runtime = "python"` without a `[python]` section, or unknown
runtime value).
- **`PathTraversal`** - Archive entry contains a `..` component that would escape `dest`.
- **`AbsolutePath`** - Archive entry has an absolute path (root or drive prefix).
- **`SymlinkRejected`** - Archive contains a symlink entry, which could be used to overwrite
arbitrary files outside `dest` on a follow-up write.
- **`HardlinkRejected`** - Archive contains a hardlink entry, same threat model as symlinks.
- **`SizeLimitExceeded`** - Cumulative decompressed size exceeded the configured cap.
- **`TooManyEntries`** - Archive contains more entries than the configured cap allows.



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
    // Reject unknown runtime values + cross-section invariants. We do this
    // here (not in serde) because the python-section presence depends on
    // the runtime field, which serde can't express in a single derive.
    manifest.package.runtime_strict()?;
    manifest.validate_runtime()?;
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



### `fidius-core::package::vendor_python_deps`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn vendor_python_deps (dir : & Path , py : & PythonPackageMeta) -> Result < () , PackageError >
```

Vendor Python dependencies into `<dir>/vendor/` by invoking `python3 -m pip install -r <requirements> --target ./vendor/`.

- If `vendor/` already exists, leave it alone — the plugin author may have
pre-vendored deliberately for reproducibility.
- If the declared requirements file is missing AND `vendor/` is missing,
emit a tracing warning and proceed (zero-dep python plugin).
- If pip fails, surface its stderr as `PackageError::ArchiveError` so the
user sees the resolver/build error directly.

<details>
<summary>Source</summary>

```rust
fn vendor_python_deps(dir: &Path, py: &PythonPackageMeta) -> Result<(), PackageError> {
    let vendor_dir = dir.join("vendor");
    if vendor_dir.exists() {
        tracing::debug!(
            vendor = %vendor_dir.display(),
            "pre-existing vendor/ directory — using as-is, skipping pip"
        );
        return Ok(());
    }

    let req_path = dir.join(py.requirements_path());
    if !req_path.exists() {
        tracing::warn!(
            package = %dir.display(),
            requirements = %req_path.display(),
            "python package has no requirements file and no vendor/ — packaging without deps"
        );
        return Ok(());
    }

    tracing::info!(
        requirements = %req_path.display(),
        vendor = %vendor_dir.display(),
        "vendoring python deps via pip"
    );

    // `python3 -m pip` rather than bare `pip` so we use whichever interpreter
    // happens to be on PATH and avoid relying on a separately-installed pip
    // shim. `Command` invokes the binary directly, bypassing shell aliases.
    let output = std::process::Command::new("python3")
        .arg("-m")
        .arg("pip")
        .arg("install")
        .arg("-r")
        .arg(&req_path)
        .arg("--target")
        .arg(&vendor_dir)
        .arg("--quiet")
        .output()
        .map_err(|e| {
            PackageError::ArchiveError(format!(
                "failed to invoke `python3 -m pip` (is python3 on PATH?): {e}"
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PackageError::ArchiveError(format!(
            "pip install failed (exit {}):\n{}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        )));
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
For Python packages (`runtime = "python"`), if a `requirements.txt` is
declared and a `vendor/` directory does not yet exist, `pip install -r
<requirements> --target ./vendor/` runs first and the result is included
in the archive. Pre-existing `vendor/` is respected and used as-is.
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

    // For Python packages: vendor declared deps into vendor/ before archiving.
    // Pre-existing vendor/ is respected (plugin author may pre-vendor for
    // reproducibility), missing requirements + missing vendor/ produces a
    // tracing warning but is not fatal (a Python plugin with no deps is fine).
    if matches!(pkg.runtime(), PackageRuntime::Python) {
        if let Some(py_meta) = manifest.python.as_ref() {
            vendor_python_deps(dir, py_meta)?;
        }
    }

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

Extract a `.fid` archive (tar + bzip2) to a destination directory using strict safety defaults.

Returns the path to the extracted top-level package directory, which is
guaranteed to exist inside `dest` and contain a `package.toml`.
This function validates every archive entry before extracting and rejects
archives containing: path-traversal components (`..`), absolute paths,
symlinks, hardlinks, more than 10,000 entries, or a cumulative declared
decompressed size exceeding 500 MB or 10× the compressed archive size.
Extraction is staged inside a temporary directory under `dest` and the
package directory is moved into place atomically on success. If validation
fails mid-archive, no files are left in `dest`.
For archives that legitimately exceed the default caps, use
[`unpack_package_with_options`].

<details>
<summary>Source</summary>

```rust
pub fn unpack_package(archive: &Path, dest: &Path) -> Result<PathBuf, PackageError> {
    unpack_package_with_options(archive, dest, &UnpackOptions::default())
}
```

</details>



### `fidius-core::package::unpack_package_with_options`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn unpack_package_with_options (archive : & Path , dest : & Path , options : & UnpackOptions ,) -> Result < PathBuf , PackageError >
```

Extract a `.fid` archive with caller-provided safety limits.

See [`unpack_package`] for the default-strict variant. Use this when the
archive's size or entry count legitimately exceeds the defaults.

<details>
<summary>Source</summary>

```rust
pub fn unpack_package_with_options(
    archive: &Path,
    dest: &Path,
    options: &UnpackOptions,
) -> Result<PathBuf, PackageError> {
    use bzip2::read::BzDecoder;
    use std::path::Component;

    let file = std::fs::File::open(archive).map_err(|e| {
        PackageError::ArchiveError(format!("failed to open {}: {e}", archive.display()))
    })?;
    let compressed_size = file.metadata().map(|m| m.len()).unwrap_or(0);

    let decoder = BzDecoder::new(file);
    let mut tar = tar::Archive::new(decoder);

    // Stage extraction inside `dest` so a failed or rejected archive leaves
    // nothing behind. `dest` must already exist.
    std::fs::create_dir_all(dest).map_err(PackageError::Io)?;
    let staging = tempfile::TempDir::new_in(dest).map_err(PackageError::Io)?;
    let staging_path = staging.path();

    let ratio_cap = compressed_size.saturating_mul(options.max_ratio);
    let mut total: u64 = 0;
    let mut count: u32 = 0;

    let entries = tar.entries().map_err(|e| {
        PackageError::ArchiveError(format!("failed to read {}: {e}", archive.display()))
    })?;

    for entry in entries {
        let mut entry = entry.map_err(|e| {
            PackageError::ArchiveError(format!("failed to read archive entry: {e}"))
        })?;

        count = count.saturating_add(1);
        if count > options.max_entries {
            return Err(PackageError::TooManyEntries {
                limit: options.max_entries,
            });
        }

        let path = entry
            .path()
            .map_err(|e| PackageError::ArchiveError(format!("invalid entry path: {e}")))?
            .into_owned();
        let entry_display = path.display().to_string();

        // 1. Reject link entries. A symlink or hardlink followed by a regular
        // file at the same path can overwrite files outside `dest`.
        let entry_type = entry.header().entry_type();
        if entry_type.is_symlink() {
            return Err(PackageError::SymlinkRejected {
                entry: entry_display,
            });
        }
        if entry_type.is_hard_link() {
            return Err(PackageError::HardlinkRejected {
                entry: entry_display,
            });
        }

        // 2. Reject `..` components and absolute paths. The `tar` crate has
        // best-effort guards but they are platform-dependent; check explicitly.
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    return Err(PackageError::PathTraversal {
                        entry: entry_display,
                    });
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(PackageError::AbsolutePath {
                        entry: entry_display,
                    });
                }
                _ => {}
            }
        }

        // 3. Enforce cumulative declared-size budget. Tar's own parsing
        // enforces that actual entry bytes match the declared header size,
        // so trusting the header here is safe against bomb archives.
        let declared = entry.header().size().unwrap_or(0);
        total = total.saturating_add(declared);
        if total > options.max_decompressed {
            return Err(PackageError::SizeLimitExceeded {
                limit: options.max_decompressed,
                actual: total,
            });
        }
        if compressed_size > 0 && options.max_ratio > 0 && total > ratio_cap {
            return Err(PackageError::SizeLimitExceeded {
                limit: ratio_cap,
                actual: total,
            });
        }

        // 4. Extract into the staging area. `unpack_in` itself rejects paths
        // that escape the base directory, but our explicit checks above mean
        // we never get here with a dangerous path.
        entry.unpack_in(staging_path).map_err(|e| {
            PackageError::ArchiveError(format!("failed to extract entry '{}': {e}", path.display()))
        })?;
    }

    // Find the top-level package directory inside staging.
    let mut pkg_dir_staging: Option<PathBuf> = None;
    for entry in std::fs::read_dir(staging_path).map_err(PackageError::Io)? {
        let entry = entry.map_err(PackageError::Io)?;
        let path = entry.path();
        if path.is_dir() && path.join("package.toml").exists() {
            pkg_dir_staging = Some(path);
            break;
        }
    }
    let pkg_dir_staging = pkg_dir_staging.ok_or_else(|| {
        PackageError::InvalidArchive("archive does not contain a package.toml".to_string())
    })?;

    // Atomically move the validated package directory to its final location
    // inside `dest`. If a directory with the same name already exists it is
    // removed first, matching the prior `tar::Archive::unpack` behaviour.
    let pkg_name = pkg_dir_staging
        .file_name()
        .ok_or_else(|| {
            PackageError::InvalidArchive("extracted package has no directory name".to_string())
        })?
        .to_os_string();
    let final_path = dest.join(&pkg_name);
    if final_path.exists() {
        std::fs::remove_dir_all(&final_path).map_err(PackageError::Io)?;
    }
    std::fs::rename(&pkg_dir_staging, &final_path).map_err(PackageError::Io)?;

    // `staging` TempDir drops here; any residual files are cleaned up.
    Ok(final_path)
}
```

</details>



