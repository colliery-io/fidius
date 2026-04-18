<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# Interface Evolution and ABI Safety

How fidius detects incompatible changes at load time, and how to evolve
a plugin interface without breaking existing plugins.

## The Problem

A host application loads plugin dylibs compiled at different times, possibly
by different authors. If the interface has changed between when a plugin was
compiled and when the host tries to load it, calling a vtable function pointer
with the wrong signature is undefined behavior. Fidius needs to reject
incompatible plugins before any code runs.

## Why Interface Hashing Works

Fidius computes a 64-bit FNV-1a hash from the required method signatures of
the interface trait. This hash is embedded in the `PluginDescriptor` at compile
time and checked by the host at load time.

### How the Hash Is Computed

The proc macro (`crates/fidius-macro/src/interface.rs`) calls
`fidius_core::hash::interface_hash()` during expansion:

1. Collect the signature strings of all **required** methods (those without
   `#[optional]`).
2. Each signature has the format: `name:arg_type_1,arg_type_2->return_type`
   (built by `build_signature_string()` in `crates/fidius-macro/src/ir.rs`).
3. Sort the signatures lexicographically.
4. Join them with `\n`.
5. Hash the combined bytes with FNV-1a 64-bit.

```
trait ImageFilter {
    fn name(&self) -> String;
    fn process(&self, input: &[u8], params: Value) -> Result<Vec<u8>, PluginError>;
}

Signatures (sorted):
  "name:->String"
  "process:& [u8],Value->Result < Vec < u8 >, PluginError >"

Combined (joined by \n):
  "name:->String\nprocess:& [u8],Value->Result < Vec < u8 >, PluginError >"

Hash = fnv1a(combined_bytes) → u64
```

### Why FNV-1a

FNV-1a was chosen for three properties:

- **Deterministic and portable.** The algorithm is trivial (XOR + multiply per
  byte) with well-known constants. No platform-specific behavior.
- **`const fn` compatible.** The `fnv1a()` function is `const fn`, allowing the
  hash to be computed at compile time and embedded as a constant.
- **Good distribution.** While not cryptographic, FNV-1a provides excellent
  avalanche behavior for short strings, meaning even small changes (e.g.,
  `String` vs `string`) produce completely different hashes.

### Why Sorting Matters

Method declaration order in the trait does not affect the hash. This is
deliberate: reordering methods in the trait definition is not a breaking change
because the vtable uses declaration order (which is fixed once published).
Sorting ensures the hash reflects *what* methods exist, not *where* they appear
in source code.

### Why Only Required Methods

Optional methods (marked `#[optional(since = N)]`) are excluded from the hash.
Their presence or absence is tracked by the capability bitfield instead. This
separation is essential: if optional methods affected the hash, adding a new
optional method would break all existing plugins even though they remain
perfectly usable.

## The Capability Bitfield

The `capabilities` field in `PluginDescriptor` is a `u64` bitfield where bit N
indicates that optional method N is implemented by this plugin.

### How Bits Are Assigned

Bits are assigned in **declaration order** within the trait, counting only
optional methods:

```rust
#[fidius::plugin_interface(version = 2, buffer = PluginAllocated)]
pub trait ImageFilter: Send + Sync {
    fn name(&self) -> String;                          // required, no bit
    fn process(&self, input: Vec<u8>) -> Vec<u8>;      // required, no bit

    #[optional(since = 2)]
    fn process_v2(&self, input: Vec<u8>) -> Vec<u8>;   // bit 0

    #[optional(since = 2)]
    fn metadata(&self) -> String;                      // bit 1
}
```

The macro generates constants like `ImageFilter_CAP_PROCESS_V2 = 1` and
`ImageFilter_CAP_METADATA = 2`.

### Why 64 Bits

A `u64` provides 64 optional methods per interface. This is a pragmatic
limit: interfaces with more than 64 optional methods are likely over-broad
and should be split. The bitfield is a single atomic read with no
allocation, making capability checks essentially free. If the limit is ever
hit, the macro emits a compile-time error rather than silently truncating.

### How the Host Uses Capabilities

Before calling an optional method, the host checks:

```rust
if handle.has_capability(ImageFilter_CAP_PROCESS_V2.trailing_zeros()) {
    handle.call_method::<In, Out>(vtable_index, &input)?;
} else {
    // Fall back to the required method or skip
}
```

The `has_capability()` method asserts that the bit index is less than 64
and checks `capabilities & (1u64 << bit) != 0`.

## Breaking vs Non-Breaking Changes

| Change | Breaking? | Detection mechanism | Action required |
|--------|-----------|---------------------|-----------------|
| Add optional method | No | Capability bit unset in old plugins | Host checks bit before calling |
| Remove optional method | **Yes** | Undetected — vtable layout changes silently | Never remove; deprecate instead |
| Add required method | **Yes** | `interface_hash` changes | Recompile all plugins |
| Remove required method | **Yes** | `interface_hash` changes | Recompile all plugins |
| Change required method signature | **Yes** | `interface_hash` changes | Recompile all plugins |
| Reorder methods in source | **Yes** | `interface_hash` unchanged (hash is order-independent), but vtable indices follow declaration order -- so reordering silently changes which function pointer each index maps to | Avoid; treat as breaking |
| Change buffer strategy | **Yes** | `buffer_strategy` mismatch at load | Recompile all plugins |
| Bump `version` attribute | No | Informational; host can check if desired | Update interface crate |
| Add/change/remove `#[method_meta]` or `#[trait_meta]` | No | Metadata is a host-readable side channel; does **not** participate in `interface_hash` | None — metadata changes are always backward-compatible at the ABI level |

### "Additive and Free"

Optional methods are *additive* (they only add capability, never remove
existing behavior) and *free* (they do not affect the interface hash, so
existing plugins continue to load). This is the primary mechanism for evolving
an interface without a breaking change.

## interface_version vs interface_hash

These two fields serve different purposes and are not redundant:

| Field | Set by | Purpose |
|-------|--------|---------|
| `interface_version` | Interface author via `version = N` | Human-meaningful version number. Lets the host apply business logic ("require at least version 3"). |
| `interface_hash` | Proc macro, computed automatically | Machine-checkable ABI fingerprint. Catches accidental drift that the author might not realize is breaking. |

### When to Bump Version

Bump `interface_version` when you make a deliberate, planned API change.
This is a semantic signal: "I know I changed the contract." The hash will
also change (for required method changes), but the version gives the host
a stable number to compare against.

### When the Hash Catches It

The hash catches changes the author did not intend to be breaking, or did not
realize were breaking. For example:

- Changing `fn process(&self, data: Vec<u8>)` to
  `fn process(&self, data: &[u8])` changes the signature string, which changes
  the hash, which causes load-time rejection. The author might not have
  thought of this as an ABI break, but it is.
- Renaming a parameter (`data` to `input`) does **not** change the hash because
  parameter names are not included in the signature string -- only types are.

### Typical Workflow

1. Add an optional method with `#[optional(since = N)]` where N is the current
   version. This is non-breaking; no version bump needed.
2. When you must add a required method or change an existing one, bump the
   version number and communicate the break to plugin authors.
3. The hash will automatically reject plugins compiled against the old
   interface. No manual hash management is needed.

## Load-Time Validation Sequence

The host validates plugins through `validate_against_interface()` in
`crates/fidius-host/src/loader.rs`:

```
load_library(path)
  ├─ check magic bytes == FIDIUS_MAGIC
  ├─ check registry_version == REGISTRY_VERSION
  ├─ check abi_version == ABI_VERSION (descriptor layout compatibility)
  └─ copy descriptor fields to owned PluginInfo

validate_against_interface(plugin, expected_hash, expected_strategy)
  ├─ if expected_hash is set:
  │     reject if plugin.interface_hash != expected_hash
  └─ if expected_strategy is set:
        reject if plugin.buffer_strategy != expected_strategy
```

Both interface-level checks are optional — the host configures which ones
to enforce via the `PluginHostBuilder`. `abi_version` and magic-bytes
checks are always enforced. `ABI_VERSION` itself is derived from the
`fidius-core` crate version per ADR-0002 (pre-1.0: `MAJOR*10000 + MINOR*100`;
post-1.0: `MAJOR*10000`), so ABI compatibility tracks the release process.

Each mismatch produces a specific `LoadError` variant with both the
expected and actual values, making diagnosis straightforward.

---

*Related documentation:*

- [Architecture Overview](architecture.md) -- the full pipeline from trait to FFI call
- [Wire Format](wire-format.md) -- debug vs release serialization and mismatch detection
- [Buffer Strategies](buffer-strategies.md) -- memory ownership across the FFI boundary
