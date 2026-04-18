---
id: apply-adr-0002-version-derived-abi
level: initiative
title: "Apply ADR-0002 — version-derived ABI_VERSION + descriptor_size field"
short_code: "FIDIUS-I-0019"
created_at: 2026-04-18T00:50:07.138367+00:00
updated_at: 2026-04-18T00:54:35.887235+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: S
initiative_id: apply-adr-0002-version-derived-abi
---

# Apply ADR-0002 — version-derived ABI_VERSION + descriptor_size field Initiative

## Context

ADR-0002 settled two decisions for the upcoming 0.1.0 ABI release:
1. `ABI_VERSION` derives from `CARGO_PKG_VERSION_MAJOR/MINOR` via a const formula
2. `PluginDescriptor` grows a `descriptor_size: u32` first field to support additive post-1.0 minor releases

This initiative is the mechanical application of those decisions. It's the prep gate for the three ABI-bumping initiatives (I-0014 Arena, I-0016 drop JSON, I-0018 method metadata) — they all need this infrastructure in place so their additions slot into the new descriptor shape naturally.

## Goals & Non-Goals

**Goals:**
- Bump workspace version to `0.1.0` (unreleased — 0.1.0 ships when the full ABI v3 batch is done)
- Replace `pub const ABI_VERSION: u32 = 2;` with the derived formula from ADR-0002
- Add `descriptor_size: u32` as the FIRST field of `PluginDescriptor`
- Generated `descriptor_size` in the macro builder uses `std::mem::size_of::<PluginDescriptor>()` at plugin build time
- All existing tests continue to pass after the ABI value changes from 2 → 100

**Non-Goals:**
- Change any descriptor semantics beyond adding `descriptor_size` (wire_format / metadata removal is I-0016, I-0014, I-0018)
- Add host-side gated field readers for new post-1.0 fields (no new fields yet — just the mechanism)
- Publish 0.1.0 to crates.io (release happens after the ABI v3 batch finishes)

## Detailed Design

### Workspace version bump

Every `Cargo.toml` in the workspace updates from `0.0.5` to `0.1.0`:
- `fidius-core`, `fidius-macro`, `fidius-host`, `fidius-cli`, `fidius`, `fidius-test`
- Cross-crate path deps also update: `fidius-core = { path = "../fidius-core", version = "0.1.0" }` etc.

### ABI_VERSION derivation (`fidius-core/src/descriptor.rs`)

Replace `pub const ABI_VERSION: u32 = 2;` with:

```rust
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
    MAJOR * 10000 + MINOR * 100
} else {
    MAJOR * 10000
};
```

For fidius-core at 0.1.0: `ABI_VERSION = 100`.

### Descriptor layout change

Current:
```rust
#[repr(C)]
pub struct PluginDescriptor {
    pub abi_version: u32,           // offset 0
    pub interface_name: *const c_char,
    // ...
    pub method_count: u32,
}
```

New:
```rust
#[repr(C)]
pub struct PluginDescriptor {
    /// Size in bytes of this descriptor struct. The FIRST field so the host
    /// can read just this value before trusting any offset calculation.
    /// Enables post-1.0 minor releases to add new fields at the end without
    /// breaking older plugins.
    pub descriptor_size: u32,       // offset 0 — NEW
    pub abi_version: u32,           // offset 4
    pub interface_name: *const c_char,
    // ...
    pub method_count: u32,
}
```

### Macro codegen (`fidius-macro/src/interface.rs:generate_descriptor_builder`)

The `__fidius_build_<trait>_descriptor` function needs to populate `descriptor_size`. It's a `const unsafe fn`, so we can use `std::mem::size_of::<PluginDescriptor>() as u32` inline:

```rust
pub const unsafe fn __fidius_build_trait_descriptor(
    plugin_name: *const std::ffi::c_char,
    vtable: *const TraitVTable,
    capabilities: u64,
    free_buffer: Option<unsafe extern "C" fn(*mut u8, usize)>,
    method_count: u32,
) -> fidius::descriptor::PluginDescriptor {
    fidius::descriptor::PluginDescriptor {
        descriptor_size: std::mem::size_of::<fidius::descriptor::PluginDescriptor>() as u32,
        abi_version: fidius::descriptor::ABI_VERSION,
        // ... rest unchanged ...
    }
}
```

### Tests to update

- `fidius-core/tests/layout_and_roundtrip.rs::descriptor_field_offsets` — `abi_version` offset moves from 0 to 4; all subsequent offsets shift by 4. Needs re-computation.
- `fidius-core/tests/layout_and_roundtrip.rs::descriptor_size_and_align` — total struct size increases by 4 (adjusted for alignment padding; likely +8 on 64-bit).
- `fidius-core/tests/layout_and_roundtrip.rs::version_constants` — asserts `ABI_VERSION == 2`; update to `ABI_VERSION == 100`.
- `fidius-macro/tests/impl_basic.rs::descriptor_fields_are_correct` — asserts `desc.abi_version == 2`; update to `100`.

No host-side reader changes needed: existing fields (through `method_count`) are "core" and always present regardless of `descriptor_size`. The gated-read pattern is for fields added in future minor releases, which don't exist yet.

### Files touched

- `Cargo.toml` files across all workspace members (6 files) — version bump
- `fidius-core/src/descriptor.rs` — ABI_VERSION derivation + descriptor_size field
- `fidius-macro/src/interface.rs` — emit descriptor_size in the builder
- `fidius-core/tests/layout_and_roundtrip.rs` — offset/size assertions
- `fidius-macro/tests/impl_basic.rs` — abi_version assertion
- `docs/reference/abi-specification.md` — update the descriptor layout section

## Alternatives Considered

- **Don't bump workspace version yet; manually bump ABI_VERSION to 3.** Avoids committing to 0.1.0 but means doing the derivation change twice. Rejected — the workspace version is unpublished between releases, bumping is free.
- **Add `descriptor_size` as the LAST field instead of first.** Rejected — breaks the whole point. Host needs to read size before trusting offsets, so it must be at a known position. First is the only safe choice.
- **Use a trailing `padding: [u8; _]` field to reserve space.** Over-engineered, doesn't solve the versioning question.

## Implementation Plan

1. Bump workspace `Cargo.toml` versions from `0.0.5` to `0.1.0`
2. Replace `ABI_VERSION` constant with the derived formula; add `parse_u32` const helper
3. Add `descriptor_size: u32` as the first field of `PluginDescriptor`
4. Update `generate_descriptor_builder` codegen to populate `descriptor_size` with `size_of::<PluginDescriptor>() as u32`
5. Update `layout_and_roundtrip.rs` tests for new offsets and size
6. Update `impl_basic.rs` test for `abi_version == 100`
7. Update `docs/reference/abi-specification.md` descriptor layout section (add descriptor_size field, note ADR-0002)
8. Run full `angreal test`, `angreal lint`

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All workspace crates at version `0.1.0`
- [ ] `ABI_VERSION` derives from `CARGO_PKG_VERSION_MAJOR/MINOR`; equals `100` for 0.1.0
- [ ] `PluginDescriptor.descriptor_size` is the first field, populated via `size_of` in generated builder
- [ ] `angreal test` — all suites pass with updated layout assertions
- [ ] `angreal lint` — clean
- [ ] ABI spec doc updated