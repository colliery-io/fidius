---
id: fidius-macro-code-generation
level: initiative
title: "fidius-macro — Code Generation"
short_code: "FIDIUS-I-0002"
created_at: 2026-03-29T00:26:17.838269+00:00
updated_at: 2026-03-29T01:26:35.624088+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: XL
initiative_id: fidius-macro-code-generation
---

# fidius-macro — Code Generation

## Context

The proc macro crate is the core of fidius — it turns idiomatic Rust traits into stable C ABI vtables and FFI shims. This is the hardest initiative because the macro must:
- Parse trait definitions and extract method signatures
- Generate `#[repr(C)]` vtable structs with correct FFI function pointer signatures per buffer strategy
- Generate `extern "C"` shim functions with serialization, `catch_unwind`, and async runtime management
- Compute interface hashes at compile time
- Produce capability bitfields for optional methods
- Emit the populated `FIDIUS_PLUGIN_REGISTRY` static

Depends on FIDIUS-I-0001 (fidius-core types).

## Goals & Non-Goals

**Goals:**
- `#[plugin_interface]` macro: trait → vtable struct + descriptor type + interface hash constant + capability bit assignments
- `#[plugin_impl]` macro: impl block → extern "C" shims + registry static + optional async runtime
- Support for `#[optional(since = N)]` methods with capability bitfield
- Buffer strategy selection via trait-level attribute (`buffer = PluginAllocated`)
- Interface hash computation from sorted required method signatures (FNV-1a)
- Feature-gated async support: detect `async fn`, generate lazy tokio runtime + `block_on` in shims
- Multi-plugin support: multiple `#[plugin_impl]` in one cdylib → single registry with N descriptors
- Clear compile errors for unsupported patterns (non-Send types, too many optional methods, etc.)

**Non-Goals:**
- CallerAllocated and Arena buffer strategy codegen (post-MVP, but the match arms should exist as `unimplemented!()`)
- Cross-language FFI generation

## Detailed Design

### `#[plugin_interface(version = N, buffer = Strategy)]`

Applied to a trait. Generates:

1. **VTable struct** — `#[repr(C)]` struct with one function pointer per method. Optional methods use `Option<fn_ptr>`. Function pointer signatures vary by buffer strategy.

2. **Interface hash constant** — `const {TRAIT}_INTERFACE_HASH: u64` computed from sorted required method signatures.

3. **Capability bit mapping** — `const {TRAIT}_CAP_{METHOD}: u64` for each optional method.

4. **Descriptor builder** — A function/macro that `#[plugin_impl]` calls to populate the `PluginDescriptor` for this interface.

### `#[plugin_impl(TraitName)]`

Applied to an impl block. Generates:

1. **Extern "C" shim functions** — One per trait method. Each shim:
   - Deserializes input bytes via `fidius_core::wire::deserialize`
   - Calls the real method on a static instance
   - Serializes the return value via `fidius_core::wire::serialize`
   - Wraps everything in `std::panic::catch_unwind`
   - Returns status code

2. **Static instance** — The impl struct constructed as a static (stateless, so ZST or unit struct).

3. **Async runtime** (if `async` feature + any async methods) — `LazyLock<tokio::Runtime>` initialized on first call.

4. **Registry contribution** — A `#[used]` static `PluginDescriptor` and a linkage mechanism (e.g., `#[link_section]` or a `ctor`-based registration) so multiple impls in one cdylib produce a single `FIDIUS_PLUGIN_REGISTRY` with all descriptors.

### Multi-Plugin Registry Assembly

The challenge: multiple `#[plugin_impl]` invocations in one cdylib each produce a `PluginDescriptor`, but we need a single `FIDIUS_PLUGIN_REGISTRY` that points to all of them.

Approach: Each `#[plugin_impl]` emits a descriptor in a well-known link section (e.g., `__DATA,__fides_desc` on macOS, `.fides.desc` on Linux). A `#[used]` `FIDIUS_PLUGIN_REGISTRY` static uses linker tricks or a build script to collect them. Alternative: a `ctor`-based registration pattern where each descriptor registers itself into a global `Vec` at load time.

The `ctor` approach is simpler and more portable. Trade-off: it runs code at dlopen time (violating "no code until first call" for metadata — but registration is not plugin business logic, it's framework bookkeeping). This is an acceptable exception.

## Testing Strategy

- **Compile tests** via `trybuild`: verify that correct trait annotations compile, and incorrect ones produce clear errors
- **Expansion tests**: snapshot the macro output for known inputs (golden file tests)
- **Unit tests**: interface hash computation produces known values for known signatures
- **Integration tests**: compile a test plugin as cdylib, load via fidius-host, call methods — but this overlaps with FIDIUS-I-0004

## Implementation Plan

1. Scaffold proc-macro crate with `syn`, `quote`, `proc-macro2` dependencies
2. Implement trait parsing: extract method names, arg types, return types, detect `async`, detect `#[optional]`
3. Implement `#[plugin_interface]` — vtable struct generation for PluginAllocated strategy
4. Implement interface hash computation
5. Implement capability bit assignment for optional methods
6. Implement `#[plugin_impl]` — extern "C" shim generation with serde + catch_unwind
7. Implement async support (feature-gated): lazy runtime + block_on shims
8. Implement multi-plugin registry assembly (ctor approach)
9. Compile error tests (trybuild) + expansion snapshot tests