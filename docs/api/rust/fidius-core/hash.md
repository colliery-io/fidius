# fidius-core::hash <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


FNV-1a interface hashing for compile-time ABI drift detection.

The proc macro computes an `interface_hash` from the sorted required method
signatures of a trait. The host checks this hash at load time to reject
plugins compiled against a different interface.

## Functions

### `fidius-core::hash::fnv1a`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
const fn fnv1a (bytes : & [u8]) -> u64
```

Compute the FNV-1a 64-bit hash of a byte slice.

<details>
<summary>Source</summary>

```rust
pub const fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}
```

</details>



### `fidius-core::hash::interface_hash`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn interface_hash (signatures : & [& str]) -> u64
```

Compute the interface hash from a set of method signatures.

Signatures are sorted lexicographically before hashing to ensure
order-independence. Each signature is joined with `\n` as a separator.
This function is **not** `const` because it allocates for sorting.
The proc macro calls this at compile time via a build-script-like pattern,
or uses `fnv1a` directly on pre-sorted, concatenated signatures.

<details>
<summary>Source</summary>

```rust
pub fn interface_hash(signatures: &[&str]) -> u64 {
    let mut sorted: Vec<&str> = signatures.to_vec();
    sorted.sort();
    let combined = sorted.join("\n");
    fnv1a(combined.as_bytes())
}
```

</details>



