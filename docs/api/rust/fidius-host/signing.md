# fidius-host::signing <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Ed25519 signature verification for plugins: dylibs (sign the file bytes) and packages (sign the runtime-agnostic `package_digest`, used by Python/WASM).

## Functions

### `fidius-host::signing::sig_path_for`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn sig_path_for (path : & Path) -> std :: path :: PathBuf
```

Compute the detached signature file path for a given file.

Appends `.sig` to the full filename (e.g., `foo.dylib` → `foo.dylib.sig`).

<details>
<summary>Source</summary>

```rust
pub fn sig_path_for(path: &Path) -> std::path::PathBuf {
    path.with_extension(format!(
        "{}.sig",
        path.extension().and_then(|e| e.to_str()).unwrap_or("")
    ))
}
```

</details>



### `fidius-host::signing::verify_signature`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn verify_signature (dylib_path : & Path , trusted_keys : & [VerifyingKey]) -> Result < () , LoadError >
```

Verify a plugin dylib's signature against trusted public keys.

Reads the dylib bytes and the detached `.sig` file, then verifies
the Ed25519 signature against each trusted key until one matches.

**Raises:**

| Exception | Description |
|-----------|-------------|
| `LoadError::SignatureRequired` | — if the `.sig` file doesn't exist |
| `LoadError::SignatureInvalid` | — if no trusted key verifies the signature |


<details>
<summary>Source</summary>

```rust
pub fn verify_signature(dylib_path: &Path, trusted_keys: &[VerifyingKey]) -> Result<(), LoadError> {
    let path_str = dylib_path.display().to_string();
    let sig_path = sig_path_for(dylib_path);

    // Read the sig file
    let sig_bytes = std::fs::read(&sig_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            LoadError::SignatureRequired {
                path: path_str.clone(),
            }
        } else {
            LoadError::Io(e)
        }
    })?;

    // Parse the signature (64 bytes)
    let signature = Signature::from_slice(&sig_bytes).map_err(|_| LoadError::SignatureInvalid {
        path: path_str.clone(),
    })?;

    // Read the dylib bytes
    let dylib_bytes = std::fs::read(dylib_path)?;

    // Try each trusted key
    for key in trusted_keys {
        if key.verify(&dylib_bytes, &signature).is_ok() {
            return Ok(());
        }
    }

    Err(LoadError::SignatureInvalid { path: path_str })
}
```

</details>



### `fidius-host::signing::verify_package_signature`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn verify_package_signature (dir : & Path , trusted_keys : & [VerifyingKey] ,) -> Result < () , LoadError >
```

Verify a **package** signature: `package.sig` in `dir`, an Ed25519 signature over `package_digest(dir)` (the runtime-agnostic content hash), against any trusted key. Used for package-based runtimes (Python, WASM) where the signed artifact is the whole package directory, not a single dylib file.

`package_digest` excludes `*.sig`, so the digest is identical at sign and
verify time, and tampering with any package file (e.g. the `.wasm`
component) changes the digest and fails verification.

<details>
<summary>Source</summary>

```rust
pub fn verify_package_signature(
    dir: &Path,
    trusted_keys: &[VerifyingKey],
) -> Result<(), LoadError> {
    let path_str = dir.display().to_string();
    let sig_path = dir.join("package.sig");

    let sig_bytes = std::fs::read(&sig_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            LoadError::SignatureRequired {
                path: path_str.clone(),
            }
        } else {
            LoadError::Io(e)
        }
    })?;

    let signature = Signature::from_slice(&sig_bytes).map_err(|_| LoadError::SignatureInvalid {
        path: path_str.clone(),
    })?;

    let digest =
        fidius_core::package::package_digest(dir).map_err(|_| LoadError::SignatureInvalid {
            path: path_str.clone(),
        })?;

    for key in trusted_keys {
        if key.verify(&digest, &signature).is_ok() {
            return Ok(());
        }
    }

    Err(LoadError::SignatureInvalid { path: path_str })
}
```

</details>



