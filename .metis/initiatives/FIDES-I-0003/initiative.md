---
id: fides-host-loading-and-calling
level: initiative
title: "fides-host — Loading and Calling"
short_code: "FIDES-I-0003"
created_at: 2026-03-29T00:26:18.756472+00:00
updated_at: 2026-03-29T11:25:34.695677+00:00
parent: FIDES-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
initiative_id: fides-host-loading-and-calling
---

# fides-host — Loading and Calling

## Context

The host-side library is what application developers use to load, validate, and call plugins at runtime. It wraps `libloading`, implements the full validation sequence from the spec (magic → abi_version → interface_hash → wire_format → buffer_strategy → signature → capabilities), and provides `PluginHandle` — a type-safe proxy that makes calling a plugin look like calling a normal Rust method.

Depends on FIDES-I-0001 (core types) and FIDES-I-0002 (macro generates the types this crate validates against).

## Goals & Non-Goals

**Goals:**
- `PluginHost<dyn Trait>` builder: search paths, load policy (strict/lenient), signature requirements, trusted keys
- `discover()` — scan a directory for valid plugins, return `Vec<PluginInfo>` with name, version, capabilities, signature status
- `load(name)` → `PluginHandle<dyn Trait>` — full validation sequence, then wrap vtable in callable proxy
- `PluginHandle` proxy that deserializes/serializes transparently — calling `handle.process(input, params)` looks like a normal method call
- `supports::<Method>()` capability checks for optional methods
- Signature verification via `ed25519-dalek`
- Proper library lifecycle: keep `Library` alive while `PluginHandle` exists, copy all metadata into owned types before any use
- Architecture detection (ELF/Mach-O/PE header parsing) to reject wrong-platform dylibs early
- Clear error types: `LoadError` enum with variants for each validation failure

**Non-Goals:**
- Plugin compilation (that's `cargo build`)
- CLI commands (FIDES-I-0005)
- Hot-reloading (future work)

## Detailed Design

### Load Sequence Implementation

```
PluginHost::load(name)
  → find dylib in search paths
  → check architecture (ELF/Mach-O/PE headers)
  → libloading::Library::new(path)
  → dlsym("FIDES_PLUGIN_REGISTRY")
  → validate PluginRegistry (magic, registry_version)
  → iterate descriptors:
      → copy all *const c_char to owned Strings
      → validate abi_version, interface_hash, wire_format, buffer_strategy
      → verify signature if required
      → read capabilities
      → find matching interface_name + plugin_name
  → construct PluginHandle with owned metadata + vtable pointer + Library Arc
```

### PluginHandle

The macro (FIDES-I-0002) generates a trait-specific `PluginHandle` impl. fides-host provides the generic machinery:

- `PluginHandleInner` holds `Arc<Library>` (prevents unload while handle exists), owned metadata, vtable pointer
- Method calls: serialize args → call vtable fn pointer → check status code → deserialize result or error → call `free_buffer` if PluginAllocated
- `supports::<Method>()` checks the capability bitfield

### Signature Verification

- Load `.sig` file from same directory as dylib
- Hash dylib bytes with SHA-512 (ed25519-dalek's default)
- Verify signature against trusted public keys
- `LoadPolicy::Strict` rejects unsigned; `LoadPolicy::Lenient` warns but allows

### Dependencies

- `libloading` — cross-platform dlopen/dlsym
- `ed25519-dalek` — signature verification
- `fides-core` — shared types

## Testing Strategy

- Unit tests for architecture detection (craft minimal ELF/Mach-O headers)
- Unit tests for descriptor validation (mock descriptors with various mismatches)
- Unit tests for signature verification (sign/verify round-trip)
- Integration tests require a compiled cdylib plugin — deferred to FIDES-I-0004

## Implementation Plan

1. Scaffold crate with `libloading`, `ed25519-dalek` dependencies
2. Implement architecture detection
3. Implement registry/descriptor loading and validation
4. Implement `PluginHost` builder
5. Implement `discover()` directory scanning
6. Implement `PluginHandle` generic machinery + `free_buffer` lifecycle
7. Implement signature verification
8. Error type design (`LoadError` enum)