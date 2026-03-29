---
id: fidius-core-foundation-types
level: initiative
title: "fidius-core — Foundation Types"
short_code: "FIDIUS-I-0001"
created_at: 2026-03-29T00:26:16.932707+00:00
updated_at: 2026-03-29T00:52:15.399648+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: fidius-core-foundation-types
---

# fidius-core — Foundation Types

## Context

fidius-core is the shared vocabulary crate that both plugin and host sides depend on. It defines the `#[repr(C)]` types that cross the FFI boundary, the buffer strategy abstraction, wire format serialization, error types, and status codes. Everything in the spec's "Plugin Descriptor" and "Wire Format" sections lands here as concrete Rust types.

This is the first crate to build because fidius-macro and fidius-host both depend on it. Getting these types right is critical — they become the stable ABI contract.

## Goals & Non-Goals

**Goals:**
- Define all `#[repr(C)]` FFI types: `PluginRegistry`, `PluginDescriptor`, status codes
- Implement buffer strategy enum (`CallerAllocated`, `PluginAllocated`, `Arena`) with only `PluginAllocated` functional for MVP
- Implement wire format module (JSON/bincode compile-time switch via `cfg(debug_assertions)`)
- Define `PluginError` and the error serialization contract
- Implement FNV-1a interface hashing utilities (used by the macro at compile time)
- Establish the Cargo workspace for the entire fidius project

**Non-Goals:**
- Proc macro implementation (FIDIUS-I-0002)
- Dynamic loading logic (FIDIUS-I-0003)
- CLI tooling (FIDIUS-I-0005)

## Detailed Design

### Types to Implement

```rust
// Registry — top-level dylib export
#[repr(C)]
pub struct PluginRegistry { magic, registry_version, plugin_count, descriptors }

// Descriptor — per-plugin metadata
#[repr(C)]
pub struct PluginDescriptor { abi_version, interface_name, interface_hash, 
    interface_version, capabilities, wire_format, buffer_strategy,
    plugin_name, vtable, free_buffer }

// Buffer strategy markers
pub enum BufferStrategyKind { CallerAllocated = 0, PluginAllocated = 1, Arena = 2 }

// Status codes
pub const STATUS_OK: i32 = 0;
pub const STATUS_BUFFER_TOO_SMALL: i32 = -1;
pub const STATUS_SERIALIZATION_ERROR: i32 = -2;
pub const STATUS_PLUGIN_ERROR: i32 = -3;
pub const STATUS_PANIC: i32 = -4;

// Wire format
pub mod wire { serialize(), deserialize() } // cfg-switched JSON/bincode

// Error type
pub struct PluginError { code: String, message: String, details: Option<Value> }

// Interface hashing
pub fn interface_hash(signatures: &[&str]) -> u64  // FNV-1a
```

### Workspace Setup

Initialize a Cargo workspace at the repo root with members for all five crates. Only fidius-core gets real code in this initiative; the others are empty stubs to establish the dependency graph.

### Dependencies

- `serde`, `serde_json` (always)
- `bincode` (for release wire format)
- No other external dependencies — this crate must stay minimal

## Testing Strategy

- Unit tests for wire format round-trip (JSON and bincode paths)
- Unit tests for interface hashing (deterministic, sorted, known vectors)
- `#[repr(C)]` layout assertions via `std::mem::offset_of!` to catch accidental field reordering
- Tests for `PluginError` serialization round-trip

## Implementation Plan

1. Initialize Cargo workspace + all crate stubs
2. Implement `#[repr(C)]` descriptor and registry types
3. Implement buffer strategy enum and marker types
4. Implement wire format module with cfg switch
5. Implement `PluginError` and status codes
6. Implement FNV-1a hashing utility
7. Layout assertion tests + round-trip tests