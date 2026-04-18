# fidius-host::package <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Host-side package integration: manifest loading, discovery, and building.

## Functions

### `fidius-host::package::load_package_manifest`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn load_package_manifest < M : DeserializeOwned > (dir : & Path ,) -> Result < PackageManifest < M > , PackageError >
```

Load and validate a package manifest against a host-defined schema.

This is the primary entry point for host applications working with packages.
The type parameter `M` is the host's metadata schema — if the manifest's
`[metadata]` section doesn't deserialize into `M`, this returns an error.

**Examples:**

```ignore
#[derive(Deserialize)]
struct MySchema {
    category: String,
    min_host_version: String,
}

let manifest = load_package_manifest::<MySchema>(Path::new("./packages/blur/"))?;
assert_eq!(manifest.metadata.category, "image-processing");
```

<details>
<summary>Source</summary>

```rust
pub fn load_package_manifest<M: DeserializeOwned>(
    dir: &Path,
) -> Result<PackageManifest<M>, PackageError> {
    fidius_core::package::load_manifest(dir)
}
```

</details>



### `fidius-host::package::discover_packages`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn discover_packages (dir : & Path) -> Result < Vec < PathBuf > , PackageError >
```

Discover packages in a directory.

Scans `dir` for subdirectories containing a `package.toml` file.
Returns the paths to each package directory found.

<details>
<summary>Source</summary>

```rust
pub fn discover_packages(dir: &Path) -> Result<Vec<PathBuf>, PackageError> {
    let mut packages = Vec::new();

    if !dir.is_dir() {
        return Ok(packages);
    }

    let entries = std::fs::read_dir(dir).map_err(PackageError::Io)?;

    for entry in entries {
        let entry = entry.map_err(PackageError::Io)?;
        let path = entry.path();
        if path.is_dir() && path.join("package.toml").exists() {
            packages.push(path);
        }
    }

    packages.sort();
    Ok(packages)
}
```

</details>



### `fidius-host::package::verify_package`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn verify_package (dir : & Path , trusted_keys : & [VerifyingKey]) -> Result < () , PackageError >
```

Verify a source package's signature against trusted public keys.

Recomputes the package digest from files on disk and verifies the
`package.sig` signature against the provided trusted keys.

**Raises:**

| Exception | Description |
|-----------|-------------|
| `PackageError::SignatureNotFound` | — if `package.sig` doesn't exist |
| `PackageError::SignatureInvalid` | — if no trusted key verifies the signature |


<details>
<summary>Source</summary>

```rust
pub fn verify_package(dir: &Path, trusted_keys: &[VerifyingKey]) -> Result<(), PackageError> {
    let sig_path = dir.join("package.sig");
    if !sig_path.exists() {
        return Err(PackageError::SignatureNotFound {
            path: dir.display().to_string(),
        });
    }

    let sig_bytes: [u8; 64] =
        std::fs::read(&sig_path)?
            .try_into()
            .map_err(|_| PackageError::SignatureInvalid {
                path: dir.display().to_string(),
            })?;

    let signature = Signature::from_bytes(&sig_bytes);
    let digest = fidius_core::package::package_digest(dir)?;

    for key in trusted_keys {
        if key.verify(&digest, &signature).is_ok() {
            return Ok(());
        }
    }

    Err(PackageError::SignatureInvalid {
        path: dir.display().to_string(),
    })
}
```

</details>



### `fidius-host::package::unpack_fid`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn unpack_fid (archive : & Path , dest : & Path) -> Result < PathBuf , PackageError >
```

Extract a `.fid` archive and validate its contents.

Delegates to [`fidius_core::package::unpack_package`] and emits a
`tracing::warn!` if the unpacked package has no `package.sig`.

**Examples:**

```ignore
let pkg_dir = unpack_fid(Path::new("blur-filter-1.0.0.fid"), Path::new("./plugins/"))?;
let manifest = load_package_manifest::<MySchema>(&pkg_dir)?;
```

<details>
<summary>Source</summary>

```rust
pub fn unpack_fid(archive: &Path, dest: &Path) -> Result<PathBuf, PackageError> {
    let pkg_dir = fidius_core::package::unpack_package(archive, dest)?;

    if !pkg_dir.join("package.sig").exists() {
        #[cfg(feature = "tracing")]
        tracing::warn!(
            package = %pkg_dir.display(),
            "unpacked package is unsigned (no package.sig found)"
        );
    }

    Ok(pkg_dir)
}
```

</details>



### `fidius-host::package::build_package`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn build_package (dir : & Path , release : bool) -> Result < PathBuf , PackageError >
```

Build a package by running `cargo build` inside the package directory.

Returns the path to the compiled cdylib on success.

<details>
<summary>Source</summary>

```rust
pub fn build_package(dir: &Path, release: bool) -> Result<PathBuf, PackageError> {
    let cargo_toml = dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(PackageError::BuildFailed(format!(
            "Cargo.toml not found in {}",
            dir.display()
        )));
    }

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build").arg("--manifest-path").arg(&cargo_toml);
    if release {
        cmd.arg("--release");
    }

    let output = cmd.output().map_err(PackageError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PackageError::BuildFailed(stderr.to_string()));
    }

    let profile = if release { "release" } else { "debug" };
    let target_dir = dir.join("target").join(profile);

    // Find the cdylib in the target directory
    let dylib_ext = if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(target_os = "windows") {
        "dll"
    } else {
        "so"
    };

    // Look for any file with the right extension
    if let Ok(entries) = std::fs::read_dir(&target_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some(dylib_ext) {
                return Ok(path);
            }
        }
    }

    Err(PackageError::BuildFailed(format!(
        "build succeeded but no .{} file found in {}",
        dylib_ext,
        target_dir.display()
    )))
}
```

</details>



