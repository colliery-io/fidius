# fidius-host::signing <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Ed25519 signature verification for plugin dylibs.

## Functions

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
| `LoadError::SignatureRequired` | if the `.sig` file doesn't exist |
| `LoadError::SignatureInvalid` | if no trusted key verifies the signature |


<details>
<summary>Source</summary>

```rust
pub fn verify_signature(dylib_path: &Path, trusted_keys: &[VerifyingKey]) -> Result<(), LoadError> {
    let path_str = dylib_path.display().to_string();

    // Build the .sig path
    let sig_path = dylib_path.with_extension(format!(
        "{}.sig",
        dylib_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
    ));

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



