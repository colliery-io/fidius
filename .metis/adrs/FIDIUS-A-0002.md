---
id: 001-abi-version-derivation-and
level: adr
title: "ABI Version Derivation and Additive Descriptor Discipline"
number: 1
short_code: "FIDIUS-A-0002"
created_at: 2026-04-17T16:10:20.086844+00:00
updated_at: 2026-04-17T16:29:09.355602+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-1: ABI Version Derivation and Additive Descriptor Discipline

## Context

`PluginDescriptor` (`fidius-core/src/descriptor.rs:126`) is the binary record every plugin exposes so the host can read its metadata without calling it. It has a fixed `#[repr(C)]` layout and a manually-maintained `ABI_VERSION: u32` constant that's bumped whenever the layout changes. Today `ABI_VERSION == 2` (bumped for the `method_count` field added in FIDIUS-T-0043).

Two problems with the current scheme surface as we plan the 0.1.0 release:

1. **Manual bumping drifts.** Three in-flight initiatives (FIDIUS-I-0014 Arena, FIDIUS-I-0016 drop JSON, FIDIUS-I-0018 method metadata) each touch the descriptor and each separately says "bump `ABI_VERSION` to 3." Without coordination, a PR race could land two "v3" changes with different meanings. The version should derive mechanically from a single authoritative source.

2. **No story for additive evolution.** Post-1.0, semver says minor releases are backward-compatible. But any descriptor field addition today forces an ABI break, because the host reads fields by fixed byte offset — if the plugin was built with N fields and the host with N+1, the host reads garbage past the plugin's bytes. Without a mechanism to tolerate "shorter" descriptors, every minor release with a descriptor change becomes a major-breaking event.

This ADR settles both: derive `ABI_VERSION` from the crate version, and establish additive-descriptor discipline so post-1.0 minor releases can add fields without breaking compatibility.

## Decision

### 1. `ABI_VERSION` derived from `CARGO_PKG_VERSION_{MAJOR,MINOR}`

```rust
// fidius-core/src/descriptor.rs
const fn parse_u32(s: &str) -> u32 {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut n = 0u32;
    while i < bytes.len() {
        n = n * 10 + (bytes[i] - b'0') as u32;
        i += 1;
    }
    n
}

const MAJOR: u32 = parse_u32(env!("CARGO_PKG_VERSION_MAJOR"));
const MINOR: u32 = parse_u32(env!("CARGO_PKG_VERSION_MINOR"));

pub const ABI_VERSION: u32 = if MAJOR == 0 {
    MAJOR * 10000 + MINOR * 100   // pre-1.0: every minor is breaking
} else {
    MAJOR * 10000                  // post-1.0: only major is breaking
};
```

Resulting values:

| Fidius version | `ABI_VERSION` |
|---|---|
| 0.0.5 (today, manual value 2) | 2 (grandfathered) |
| 0.1.0 (next release) | 100 |
| 0.1.7 (hypothetical patch) | 100 (unchanged — patch is ABI-compat) |
| 0.2.0 | 200 |
| 1.0.0 | 10000 |
| 1.1.0 | 10000 (ABI-compat with 1.0.0 via additive discipline) |
| 2.0.0 | 20000 |

The 2 → 100 jump at 0.0.5 → 0.1.0 is monotonic — old hosts expecting ABI=2 reject new plugins with ABI=100 via the existing `IncompatibleAbiVersion` error. Clean transition, no special case.

### 2. `descriptor_size` as the first field

```rust
#[repr(C)]
pub struct PluginDescriptor {
    /// Size in bytes of this descriptor. Host uses this to detect fields
    /// added in later minor versions: any field at offset >= size is not
    /// present in this plugin's build.
    pub descriptor_size: u32,
    pub abi_version: u32,
    // ... all existing fields ...
}
```

Must be the very first field so the host can safely read just those 4 bytes before trusting any offset calculation. `abi_version` stays second.

Host reads future-added fields through a gate:

```rust
// Existing fields (through method_count) are core — always present, read directly.
// New fields added in a minor version use this pattern:

fn method_metadata(desc: &PluginDescriptor) -> Option<*const MethodMetaEntry> {
    let offset = memoffset::offset_of!(PluginDescriptor, method_metadata);
    let needed = offset + std::mem::size_of::<*const MethodMetaEntry>();
    if (desc.descriptor_size as usize) >= needed {
        Some(desc.method_metadata)
    } else {
        None   // plugin was built before this field existed
    }
}
```

Plugin codegen always emits `descriptor_size: std::mem::size_of::<PluginDescriptor>() as u32` at build time — the value is automatic, reflects the plugin's view of the struct at its compile time.

### 3. Layout evolution rules

**Post-1.0 minor releases MAY:**
- Add fields at the end of `PluginDescriptor`
- Add variants to repr enums (`BufferStrategyKind`, etc.) — only at the end, never renumber
- Add optional methods to traits (already supported via capability bits)
- Add new methods to a typed Client or host API (normal Rust additive)

**Post-1.0 major releases ONLY may:**
- Remove or reorder descriptor fields
- Remove or renumber existing enum variants
- Change vtable function signatures (including for a given buffer strategy)
- Change required method signatures (already caught at load time by `interface_hash`)
- Change the meaning of a `MetaKv` key reserved in the `fidius.*` namespace

**Pre-1.0 (0.x) releases may do any of the above per minor.** The user-facing compatibility guarantee kicks in at 1.0.0. Until then, every minor is a clean break.

### 4. Coordination with the 0.1.0 batch

The three in-flight ABI-affecting initiatives (I-0014 Arena, I-0016 drop JSON, I-0018 method metadata) all land in 0.1.0, which will have `ABI_VERSION = 100`. They no longer each "bump to 3" — the version comes from the crate. The `descriptor_size` field is added as part of this release, giving us the machinery for post-1.0 additive growth.

Specifically for 0.1.0:
- I-0016 removes `wire_format`. Pre-1.0 so allowed; post-1.0 this kind of change would require a major.
- I-0014 changes vtable fn signatures for Arena strategy. Governed by `buffer_strategy` enum tag — additive variant, works under the discipline.
- I-0018 adds `method_metadata`, `trait_metadata`, `trait_metadata_count` fields at the end. Fully additive — foreshadows the post-1.0 pattern.

## Alternatives Analysis

| Option | Pros | Cons | Risk | Cost |
|---|---|---|---|---|
| **A. Chosen: version-derived + additive with `descriptor_size`** | Mechanical, no manual bumps, supports post-1.0 minor additions, one-time transition at 1.0 with no special case | Host read path for new fields needs a size check; slight indirection via `memoffset::offset_of!` | Low | S |
| B. Keep manual `ABI_VERSION` counter | Simpler code (one `const`) | Coordination burden across PRs, drift risk, no mechanism for minor additions post-1.0 | Medium | N/A (status quo) |
| C. Derive from version but strict-equal only (no `descriptor_size`) | Simpler (no size field) | Every descriptor addition post-1.0 forces a major bump; releases become artificially large | Low | S |
| D. `descriptor_size` only, keep manual `ABI_VERSION` | Allows additive growth without version coupling | Still has manual-bump coordination problem; two independent version concerns to track | Low | S |
| E. Embed full semver string in descriptor (`version: *const c_char`) | Maximum diagnostic value | Can't be compared with integer ops; breaks the `abi_version: u32` field semantics | Medium | M |

## Rationale

**Why derive from version:** eliminates manual coordination. Three initiatives each "bumping to 3" is a merge-conflict pattern. With derivation, the version in `Cargo.toml` is the single source of truth; bumping the crate version in the normal release process updates `ABI_VERSION` automatically.

**Why additive with `descriptor_size`:** post-1.0 we need to keep the semver contract. Minor releases should be backward-compatible. Without an in-band size field, every descriptor change forces a major bump — which is both a developer ergonomics tax (we'd batch features into rare major releases) and a consumer trust tax (downstream plugins need re-testing on every minor). `descriptor_size` is 4 bytes and solves both.

**Why pre-1.0 can still break per minor:** 0.x is exempt from semver's compat guarantee by convention. Using it as a runway to iterate freely is the point of pre-1.0. The conditional formula (`if MAJOR == 0`) captures this cleanly: we're not committing now to post-1.0 semantics while we still need flexibility pre-1.0.

**Why not just `ABI_VERSION == MAJOR` everywhere:** because `0 * 10000 = 0` — doesn't distinguish 0.0.5, 0.1.0, 0.2.0. Pre-1.0 every minor must be distinguishable. The `if MAJOR == 0` branch handles this.

**Why not use a build.rs to compute `ABI_VERSION`:** unnecessary — `env!("CARGO_PKG_VERSION_MAJOR")` is available at `const` context, parseable with a simple const fn.

## Consequences

### Positive
- No manual `ABI_VERSION` bumps to forget or conflict over
- Release notes and ABI compat move together by default
- Post-1.0 minor releases can add descriptor fields without breaking plugins built against an earlier minor
- `descriptor_size` is a standard pattern from stable-ABI libraries (libc, COM) — familiar, well-understood
- Patch releases are automatically ABI-compat (useful for bug fixes that don't touch descriptor)

### Negative
- Host read path for any field added post the 0.1.0 release becomes conditional (one extra comparison + `offset_of!` call — trivial perf cost, more code)
- Developers adding a new field must remember it's conditionally readable on the host — a convention to document
- The `if MAJOR == 0` branch is a one-time special case that looks odd in isolation

### Neutral
- `abi_version` field stays at offset 4 (after `descriptor_size` at offset 0) — plugins compiled against 0.0.5 won't load against 0.1.0 or later regardless of any additive discipline (different `ABI_VERSION` values)
- Enum variant additions (e.g., `BufferStrategyKind::Arena` being ABI-readable post-1.0 minor) require the discipline "only add at the end, never renumber" — already implicit but now documented
- We defer committing to how vtable additions work in minors — today new trait methods are already supported via `#[optional]` + capability bits, which is compatible with the additive discipline

## Consequences for Other Work

- **FIDIUS-I-0014, I-0016, I-0018** each lose their "bump ABI_VERSION to 3" step — replaced by "this lands in 0.1.0, which yields `ABI_VERSION = 100`."
- **`descriptor_size` addition** becomes its own small task, sequenced early in the 0.1.0 batch (before the other ABI-affecting initiatives land so their additions can assume the pattern).
- **Spec document FIDIUS-S-0001** needs an update documenting `descriptor_size` and the layout evolution rules.
- **Future descriptor additions** (post-0.1.0) follow the pattern documented here.

## Review Triggers

Revisit this ADR if:
- Compile-time `env!` access becomes unreliable (e.g., cross-compilation concerns where the plugin's `CARGO_PKG_VERSION` differs from fidius's)
- We find a case where minor releases need to break ABI for non-additive reasons (would require rethinking the post-1.0 formula)
- A downstream consumer encounters confusion from the 0.x → 1.0 formula switch

Not scheduled for recurring review — the decision is expected to be stable through 0.x and into the 1.0 era.