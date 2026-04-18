# fidius-test::signing <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Deterministic signing fixtures for tests that exercise Fidius signature verification flows.

These helpers are **not secure** — the signing keys are derived from a
single byte seed. They exist so tests can sign and verify plugin dylibs
without generating fresh random keys each run.

## Functions

### `fidius-test::signing::fixture_keypair_with_seed`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn fixture_keypair_with_seed (seed : u8) -> (SigningKey , VerifyingKey)
```

Deterministic Ed25519 keypair derived from `seed` repeated 32 times.

Use different seeds across tests that need distinct keys (e.g., to verify
that a wrong-key signature is rejected).

<details>
<summary>Source</summary>

```rust
pub fn fixture_keypair_with_seed(seed: u8) -> (SigningKey, VerifyingKey) {
    let signing = SigningKey::from_bytes(&[seed; 32]);
    let verifying = signing.verifying_key();
    (signing, verifying)
}
```

</details>



### `fidius-test::signing::fixture_keypair`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn fixture_keypair () -> (SigningKey , VerifyingKey)
```

Convenience: [`fixture_keypair_with_seed(1)`](fixture_keypair_with_seed).

<details>
<summary>Source</summary>

```rust
pub fn fixture_keypair() -> (SigningKey, VerifyingKey) {
    fixture_keypair_with_seed(1)
}
```

</details>



### `fidius-test::signing::sign_dylib`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn sign_dylib (dylib : & Path , key : & SigningKey) -> std :: io :: Result < () >
```

Sign a plugin dylib in place by writing a detached `.sig` file alongside it.

The signature file uses the same naming convention as `fidius sign` —
appends `.sig` to the full filename (e.g., `foo.dylib` → `foo.dylib.sig`).

<details>
<summary>Source</summary>

```rust
pub fn sign_dylib(dylib: &Path, key: &SigningKey) -> std::io::Result<()> {
    let bytes = std::fs::read(dylib)?;
    let signature = key.sign(&bytes);
    let sig_path = dylib.with_extension(format!(
        "{}.sig",
        dylib.extension().and_then(|e| e.to_str()).unwrap_or("")
    ));
    std::fs::write(sig_path, signature.to_bytes())?;
    Ok(())
}
```

</details>



