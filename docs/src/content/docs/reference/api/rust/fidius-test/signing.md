---
title: "fidius-test::signing"
description: "Deterministic signing fixtures for tests that exercise Fidius signature verification flows."
editUrl: false
sidebar:
  order: 53
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Deterministic signing fixtures for tests that exercise Fidius signature verification flows.

These helpers are **not secure** — the signing keys are derived from a
single byte seed. They exist so tests can sign and verify plugin dylibs
without generating fresh random keys each run.

## Functions

### <span id="fn.fixture_keypair_with_seed"></span><code style="font-size: 0.85em">fn fixture_​keypair_​with_​seed</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn fixture_keypair_with_seed(seed: u8) -> (SigningKey, VerifyingKey)
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

### <span id="fn.fixture_keypair"></span><code style="font-size: 0.85em">fn fixture_​keypair</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn fixture_keypair() -> (SigningKey, VerifyingKey)
```

Convenience: [`fixture_keypair_with_seed(1)`](../../../rust/fidius-test/signing/#fn.fixture_keypair_with_seed).

<details>
<summary>Source</summary>

```rust
pub fn fixture_keypair() -> (SigningKey, VerifyingKey) {
    fixture_keypair_with_seed(1)
}
```

</details>

### <span id="fn.sign_dylib"></span><code style="font-size: 0.85em">fn sign_​dylib</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn sign_dylib(dylib: &Path, key: &SigningKey) -> std::io::Result<()>
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
