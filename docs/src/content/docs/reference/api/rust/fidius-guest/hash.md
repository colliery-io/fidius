---
title: "fidius-guest::hash"
description: "FNV-1a interface hashing for compile-time ABI drift detection."
sidebar:
  order: 15
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


FNV-1a interface hashing for compile-time ABI drift detection.

The proc macro computes an `interface_hash` from the sorted required method
signatures of a trait. The host checks this hash at load time to reject
plugins compiled against a different interface.

## Functions

### <span id="fn.fnv1a"></span>`fn fnv1a`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


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



### <span id="fn.interface_hash"></span>`fn interface_hash`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


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



### <span id="fn.signature_string"></span>`fn signature_string`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


```rust
fn signature_string (name : & str , arg_types : & [String] , ret : & str , wire_raw : bool , streaming : bool , client_streaming : bool ,) -> String
```

Build the canonical signature string for one method.

Format: `"{name}:{arg_type_1},{arg_type_2}->{return_type}{!raw?}"`.
- `arg_types` are pre-stringified (typically by `syn::Type` →
`to_token_stream().to_string()` — the proc macro and any other
tooling that wants to compute the same hash must use the same
formatter).
- `return_type` is the stringified return type, or empty string for
methods returning `()`.
- `wire_raw = true` appends a trailing `!raw` marker so methods opted
into raw wire mode hash differently from bincode-typed methods of
the same Rust signature. This is the protection that makes a
wire-mode mismatch surface as a load-time hash mismatch instead of
silent data corruption.
- `streaming = true` appends a trailing `!stream` marker (after any
`!raw`) so a server-streaming method (`-> fidius::Stream<T>`, where
`ret` is the per-item type `T`) hashes differently from a unary
`-> T` method of the same name/args. Same protection as `!raw`, for
the streaming/unary split (FIDIUS-I-0026, D4).
This function lives in `fidius-core` (not `fidius-macro`) so the proc
macro and downstream tooling like `fidius python-stub` share a single
source of truth for the format. Drift between them = silent hash
mismatch, which is exactly what the load-time check is meant to catch
— but better to never have the drift in the first place.

<details>
<summary>Source</summary>

```rust
pub fn signature_string(
    name: &str,
    arg_types: &[String],
    ret: &str,
    wire_raw: bool,
    streaming: bool,
    client_streaming: bool,
) -> String {
    let raw_marker = if wire_raw { "!raw" } else { "" };
    let stream_marker = if streaming { "!stream" } else { "" };
    // Client-streaming (`Stream<T>` in argument position) hashes distinctly from a
    // unary or server-streaming method of the same name/args (FIDIUS-I-0030 / ADR-0007).
    let client_stream_marker = if client_streaming { "<stream" } else { "" };
    format!(
        "{}:{}->{}{}{}{}",
        name,
        arg_types.join(","),
        ret,
        raw_marker,
        stream_marker,
        client_stream_marker
    )
}
```

</details>



