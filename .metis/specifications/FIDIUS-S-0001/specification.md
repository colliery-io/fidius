---
id: fidius-abi-and-wire-protocol
level: specification
title: "Fidius ABI and Wire Protocol Specification"
short_code: "FIDIUS-S-0001"
created_at: 2026-03-28T22:40:35.229889+00:00
updated_at: 2026-03-29T00:26:08.595434+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#specification"
  - "#phase/drafting"


exit_criteria_met: false
initiative_id: NULL
---

# Fidius ABI and Wire Protocol Specification

## Overview

This document specifies the binary interface contract between Fidius host applications and dynamically loaded plugins. It covers the plugin descriptor layout, vtable structure, buffer management strategies, wire format negotiation, interface hashing, capability bitfields, and the multi-plugin-per-dylib mechanism.

All types crossing the FFI boundary use `#[repr(C)]` layout. All complex data is serialized (JSON or bincode). No Rust-specific types cross the boundary.

## System Context

### Actors
- **Interface author**: Defines the plugin trait with `#[plugin_interface]` in a dedicated interface crate. Publishes the crate so plugin authors can depend on it.
- **Plugin author**: Implements the trait with `#[plugin_impl]`. Depends only on the interface crate (which re-exports fidius macros and types).
- **Host application**: Loads dylibs, validates descriptors, calls plugin methods through generated proxies. Depends on the interface crate + `fidius-host`.

### Boundaries
- **Inside scope**: C ABI vtable generation, descriptor format, wire serialization, buffer management, interface evolution, signing format
- **Outside scope**: Plugin distribution/packaging, WASM sandboxing, cross-language plugins, statefulness

## Crate Structure

```
fidius/
├── fidius-core/       # Shared types: descriptor layout, buffer strategy traits, wire format,
│                     # capability bits, version hashes, error types, signing types
├── fidius-macro/      # proc-macro: #[plugin_interface], #[plugin_impl]
├── fidius-host/       # Host-side: loading, validation, signature verification, calling
├── fidius-cli/        # CLI: init-interface, init-plugin, sign, verify, inspect
└── fidius/            # Facade crate re-exporting core + macro
```

`fidius-core` is the only shared library dependency. The `fidius` facade re-exports `fidius-core` types and `fidius-macro` macros so that interface crates have a single dependency.

## Developer Workflow

### Interface Crate (The Schema)

The interface author creates a dedicated crate that defines the plugin contract. This crate is the single dependency for plugin authors — it re-exports all necessary fidius macros and types.

**Scaffolding via CLI:**

```
$ cargo install fidius-cli
$ fidius init-interface my-app-plugin-api --trait ImageFilter
```

Generates:

```
my-app-plugin-api/
├── Cargo.toml          # depends on `fidius`
└── src/
    └── lib.rs
```

```rust
// my-app-plugin-api/src/lib.rs

// Re-export so plugin authors need only depend on this crate
pub use fidius::plugin_impl;
pub use fidius::PluginError;

#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait ImageFilter: Send + Sync {
    fn name(&self) -> String;
    fn process(&self, input: &[u8], params: serde_json::Value) -> Result<Vec<u8>, PluginError>;
}
```

### Plugin Crate

```
$ fidius init-plugin my-blur-plugin --interface my-app-plugin-api
```

Generates:

```
my-blur-plugin/
├── Cargo.toml          # depends on `my-app-plugin-api`, crate-type = ["cdylib"]
└── src/
    └── lib.rs
```

```rust
// my-blur-plugin/src/lib.rs
use my_app_plugin_api::{plugin_impl, ImageFilter, PluginError};

pub struct BlurFilter;

#[plugin_impl(ImageFilter)]
impl ImageFilter for BlurFilter {
    fn name(&self) -> String { "blur".into() }
    fn process(&self, input: &[u8], params: serde_json::Value) -> Result<Vec<u8>, PluginError> {
        // ...
        Ok(vec![])
    }
}

fidius_core::fidius_plugin_registry!();
```

> **Note on `PluginError`**: The `details` field is `Option<String>` (JSON-encoded) rather than `Option<serde_json::Value>` to ensure compatibility with the bincode wire format. Use `PluginError::with_details()` to set it and `details_value()` to parse it back.
```

### Dependency Graph

```
my-app (host)
├── my-app-plugin-api (interface crate)
│   └── fidius (facade → fidius-core + fidius-macro)
└── fidius-host (loading, validation, calling)
    └── fidius-core

my-blur-plugin (cdylib)
└── my-app-plugin-api (interface crate)
    └── fidius (facade → fidius-core + fidius-macro)
```

Plugin authors never depend on `fidius` directly — the interface crate is their entire contract surface.

### CLI Commands

| Command | Purpose |
|---------|---------|
| `fidius init-interface <name> --trait <TraitName>` | Scaffold an interface crate |
| `fidius init-plugin <name> --interface <crate>` | Scaffold a plugin crate (cdylib) |
| `fidius sign --key <secret> <dylib>` | Sign a compiled plugin |
| `fidius verify --key <public> <dylib>` | Verify a plugin signature |
| `fidius inspect <dylib>` | Dump registry: plugin names, interface hashes, capabilities, wire format |
| `fidius keygen --out <name>` | Generate Ed25519 keypair |

## Plugin Descriptor

Every dylib exports one or more static descriptors via well-known symbols.

### Registry (Multi-Plugin Support)

Each dylib exports a single registry symbol:

```rust
#[repr(C)]
pub struct PluginRegistry {
    pub magic: [u8; 8],                    // b"FIDIUS\0\0"
    pub registry_version: u32,             // Layout version of this struct
    pub plugin_count: u32,                 // Number of descriptors
    pub descriptors: *const *const PluginDescriptor,  // Array of pointers
}

// Plugin crates call fidius_plugin_registry!() to emit this export:
#[no_mangle]
pub extern "C" fn fidius_get_registry() -> *const PluginRegistry { ... }
```

Each `#[plugin_impl]` registers its descriptor via `inventory::submit!`. The `fidius_plugin_registry!()` macro emits the `fidius_get_registry()` export function, which lazily builds the registry from all submitted descriptors via `OnceLock`.

The host calls `dlsym("fidius_get_registry")`, invokes it, and gets back a `*const PluginRegistry`.

### Descriptor Layout

```rust
#[repr(C)]
pub struct PluginDescriptor {
    pub abi_version: u32,             // Wire format / descriptor layout version
    pub interface_name: *const c_char,// Null-terminated interface trait name
    pub interface_hash: u64,          // FNV-1a of required method signatures
    pub interface_version: u32,       // The `version = N` from #[plugin_interface]
    pub capabilities: u64,            // Bitfield: one bit per optional method
    pub wire_format: u8,              // 0 = JSON, 1 = bincode
    pub buffer_strategy: u8,          // 0 = CallerAllocated, 1 = PluginAllocated, 2 = Arena
    pub plugin_name: *const c_char,   // Null-terminated human-readable name
    pub vtable: *const c_void,        // Pointer to the interface-specific VTable
    pub free_buffer: Option<unsafe extern "C" fn(*mut u8, usize)>,  // Only for PluginAllocated
}
```

**Field semantics:**

| Field | Purpose |
|-------|---------|
| `abi_version` | Descriptor struct layout version. Bump when adding/reordering fields. |
| `interface_name` | Trait name as string — used for discovery ("give me all ImageFilter plugins") |
| `interface_hash` | FNV-1a hash of required method signatures (name + arg types + return type). Detects ABI drift. |
| `interface_version` | User-specified version from `#[plugin_interface(version = N)]` |
| `capabilities` | Bit N = 1 means optional method N is implemented. Up to 64 optional methods per interface. |
| `wire_format` | Encodes serialization format. Host rejects mismatches. |
| `buffer_strategy` | Encodes which allocation pattern the vtable expects. Host must match. |
| `plugin_name` | Human-readable identifier for this specific plugin impl |
| `vtable` | Opaque pointer to the `#[repr(C)]` vtable struct for this interface |
| `free_buffer` | Deallocation function for plugin-allocated buffers. Must be `Some` when `buffer_strategy == 1`. |

## VTable Generation

The `#[plugin_interface]` macro generates a `#[repr(C)]` vtable struct with one function pointer per trait method.

### FFI Signature Patterns by Buffer Strategy

All methods follow a common pattern: input data as `(*const u8, u32)` pairs, output via strategy-specific mechanism, return `i32` status code.

**CallerAllocated (strategy = 0):**
```rust
// Host allocates output buffer, plugin writes into it.
// Returns 0 on success, -1 if buffer too small (writes needed size to out_len).
unsafe extern "C" fn(
    in_ptr: *const u8, in_len: u32,       // serialized input
    out_ptr: *mut u8, out_cap: u32,       // caller-provided buffer
    out_len: *mut u32,                    // actual bytes written (or needed size)
) -> i32
```

**PluginAllocated (strategy = 1):**
```rust
// Plugin allocates output, host frees via descriptor's free_buffer.
// Returns 0 on success. Host must call free_buffer(out_ptr, out_len) after reading.
unsafe extern "C" fn(
    in_ptr: *const u8, in_len: u32,       // serialized input
    out_ptr: *mut *mut u8,                // plugin writes allocated pointer here
    out_len: *mut u32,                    // plugin writes length here
) -> i32
```

**Arena (strategy = 2):**
```rust
// Host provides pre-allocated arena, plugin writes into it.
// Data valid only until next call. Returns 0 on success, -1 if arena too small.
unsafe extern "C" fn(
    in_ptr: *const u8, in_len: u32,       // serialized input
    arena_ptr: *mut u8, arena_cap: u32,   // host-provided arena
    out_offset: *mut u32,                 // offset into arena where output starts
    out_len: *mut u32,                    // output length
) -> i32
```

### Optional Methods

Optional methods use `Option<fn_ptr>` in the vtable. The capability bitfield provides a fast check without inspecting the vtable directly.

```rust
#[repr(C)]
pub struct ImageFilter_VTable {
    // Required
    pub name: unsafe extern "C" fn(/* ... */) -> i32,
    pub process: unsafe extern "C" fn(/* ... */) -> i32,
    // Optional — bit 0 in capabilities
    pub process_with_metadata: Option<unsafe extern "C" fn(/* ... */) -> i32>,
}
```

## Wire Format

### Serialization

| Mode | Format | Use case |
|------|--------|----------|
| `wire_format = 0` | JSON via `serde_json` | Debug builds — human-readable, inspectable |
| `wire_format = 1` | bincode via `bincode` | Release builds — compact, fast |

Selected at compile time via `#[cfg(debug_assertions)]`. The `wire_format` field in the descriptor allows the host to reject debug-plugin-in-release-host (and vice versa) at load time.

### Type Mapping

All method arguments and return values are serialized through serde. The FFI shims handle:
1. Deserialize input bytes → Rust types
2. Call the real trait method
3. Serialize return value → output bytes
4. `catch_unwind` around the entire call — panics become error status codes

## Interface Hashing

`interface_hash` is computed at compile time from required method signatures:

```
For each required method (sorted by name):
    hash(method_name + ":" + arg_type_1 + "," + arg_type_2 + ... + "->" + return_type)
Combine with FNV-1a
```

This detects:
- Added/removed required methods
- Changed argument types
- Changed return types

It does **not** detect changes to optional methods (those are tracked by capability bits).

## Interface Evolution

| Change | Effect | Detection |
|--------|--------|-----------|
| Add optional method | Old plugins load fine, bit unset in capabilities | Host checks bit before calling |
| Add required method | `interface_hash` changes | Host rejects at load |
| Change required method signature | `interface_hash` changes | Host rejects at load |
| Remove optional method | Capability bit disappears | Host already handles absence |
| Remove required method | Breaking — bump `interface_version` | New hash |
| Change wire format | Bump `abi_version` | Host rejects at load |

**Rule**: Optional methods are additive and free. Required method changes are always breaking.

## Signing

Ed25519 via `ed25519-dalek`. Detached `.sig` file alongside the dylib.

- **Sign**: Hash dylib bytes → sign with secret key → write `.sig`
- **Verify**: Hash dylib bytes → verify against `.sig` with public key
- **Host policy**: `require_signature(true)` rejects unsigned plugins; `trusted_keys(&[...])` specifies accepted public keys

Not a PKI / certificate chain — just "was this produced by a holder of this key."

## Load Sequence

```
dlopen(path)
  → dlsym("fidius_get_registry")              // get registry function
  → call fidius_get_registry()                // builds registry on first call
  → check registry magic bytes                // reject non-fidius dylibs
  → check registry_version                    // reject incompatible registry layout
  → for each descriptor in registry:
      → check abi_version                     // reject incompatible descriptor layout
      → check interface_name                  // filter to requested interface
      → check interface_hash                  // reject ABI-drifted plugins
      → check wire_format                     // reject debug/release mismatch
      → check buffer_strategy                 // reject strategy mismatch
      → verify signature (if required)        // ed25519 over dylib bytes
      → read capabilities bitfield            // know which optional methods exist
      → wrap vtable in PluginHandle           // ready to call
```

No plugin code executes until the first actual method call.

## Status Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| -1 | Buffer too small (CallerAllocated/Arena: `out_len` contains needed size) |
| -2 | Serialization error |
| -3 | Plugin logic error (details in output buffer as serialized `PluginError`) |
| -4 | Panic caught at FFI boundary |

## Constraints

- Maximum 64 optional methods per interface (u64 bitfield)
- Host and plugin must match on wire_format and buffer_strategy
- All `*const c_char` fields must point to static, null-terminated strings
- `vtable` pointer must remain valid for the lifetime of the loaded library