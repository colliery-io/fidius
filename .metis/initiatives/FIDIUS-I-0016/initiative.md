---
id: decouple-wire-format-from-cfg
level: initiative
title: "Decouple Wire Format from cfg(debug_assertions)"
short_code: "FIDIUS-I-0016"
created_at: 2026-04-17T13:23:37.557424+00:00
updated_at: 2026-04-18T01:01:23.380026+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: decouple-wire-format-from-cfg
---

# Decouple Wire Format from cfg(debug_assertions) Initiative

## Context

Today, wire format is tied to the build profile via `cfg(debug_assertions)` in `fidius-core/src/wire.rs:29,33`:

```rust
#[cfg(debug_assertions)]
pub const WIRE_FORMAT: WireFormat = WireFormat::Json;

#[cfg(not(debug_assertions))]
pub const WIRE_FORMAT: WireFormat = WireFormat::Bincode;
```

Every descriptor carries this constant as `wire_format: u8`, and the host checks it against its own `WIRE_FORMAT` constant at load time via `WireFormatMismatch` error (`fidius-host/src/loader.rs` + `fidius-host/src/host.rs:expected_wire`).

**Consequence:** a release host cannot load a debug plugin, and a debug host cannot load a release plugin. Switching from "iterating on the plugin" to "benchmarking the full pipeline" requires rebuilding both sides in the same profile.

**The trade-off as stated:** JSON in debug is "for inspection" — you can read raw JSON bytes in a hex dump. In practice:
- Nobody reads raw wire bytes at a call boundary during normal debugging. They use `fidius inspect`, attach a debugger, or print the deserialized value.
- JSON is 2-10x slower than bincode for typical payloads — this slows down every dev-loop iteration.
- Bincode is opaque, but `bincode::deserialize::<serde_json::Value>(...)` is not — any debug tool can decode bincode to JSON for display.

**The underlying issue:** wire format is being decided globally at compile time by an implicit build flag, rather than deliberately per-interface by the author.

## Goals & Non-Goals

**Goals:**
- Bincode is the only wire format
- `cfg(debug_assertions)` has no effect on wire format
- Host-plugin profile mixing just works (release host loads debug plugin, vice versa)
- Remove JSON serialization code path, `serde_json` from hot-path deps where unused
- `WireFormat` enum collapses to a single variant (or is removed and the descriptor field becomes reserved)

**Non-Goals:**
- Keep JSON as an opt-in alternative (user has explicitly rejected this — it's been a repeated source of pain)
- Per-interface wire format selection
- Runtime format negotiation

## Decision (Settled)

User decision: **drop JSON entirely**. The debug/release dual path has caused repeated footguns, the edit-and-inspect value was never realized in practice, and dropping it is a net simplification. Bincode everywhere.

## Detailed Design

### `fidius-core/src/wire.rs` (simplified)

```rust
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum WireError {
    #[error("bincode wire error: {0}")]
    Bincode(#[from] bincode::Error),
}

pub fn serialize<T: Serialize>(val: &T) -> Result<Vec<u8>, WireError> {
    bincode::serialize(val).map_err(WireError::Bincode)
}

pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WireError> {
    bincode::deserialize(bytes).map_err(WireError::Bincode)
}
```

No `cfg(debug_assertions)`. No `WIRE_FORMAT` const. No `serde_json`.

### `WireFormat` enum: remove or collapse

Two options:

**A. Remove `WireFormat` entirely.** Drop the `wire_format: u8` field from `PluginDescriptor`. Bump `ABI_VERSION` to 3. Cleanest.

**B. Keep `wire_format: u8` field as `reserved` for future format negotiation.** No `WireFormat` enum needed in public API. Descriptor stays the same shape (no ABI bump). Set to `0` always; any non-zero value from an old plugin is ignored/accepted.

Recommendation: **Option A** — we're already willing to take ABI churn (we're 0.0.5, pre-1.0). Clean break, descriptor layout shrinks, one less footgun.

### Impact on macro codegen

- `fidius-macro/src/interface.rs:generate_descriptor_builder` — remove `wire_format` field from the emitted `PluginDescriptor` literal
- `fidius-macro/src/impl_macro.rs` — shim codegen keeps `wire::serialize`/`wire::deserialize` but they're now the single-format flavor from core

### Impact on host

- `fidius-host/src/error.rs` — remove `WireFormatMismatch` variant
- `fidius-host/src/loader.rs` — remove `validate_against_interface`'s wire-format check
- `fidius-host/src/host.rs` — remove `expected_wire`, `wire_format(...)` builder method
- `fidius-host/src/types.rs` — remove `wire_format: WireFormat` from `PluginInfo`
- `fidius-core/src/descriptor.rs` — remove `wire_format: u8` field, remove `WireFormat` enum, remove `wire_format_kind()` method. Bump `ABI_VERSION` to 3.

### Impact on CLI

- `fidius-cli/src/commands.rs:inspect` — remove the "Wire format" line from output
- Scaffolds don't need to mention wire at all

### Dependencies

- Can we drop `serde_json` from `fidius-core`? Check usage. `PluginError::details` is `Option<String>` stringified JSON — this is not wire-format JSON but a separate convention. Need to examine whether `serde_json` is used anywhere else. If not — remove it from `fidius-core/Cargo.toml`.
- `fidius-cli` still uses `serde_json` for the crates.io lookup — keep there.

## Alternatives Considered

- **Keep both formats per-interface:** user explicitly rejected — "it's bit me in the ass too many times"
- **Keep cfg(debug_assertions) but make host profile-agnostic:** half-measure, doesn't eliminate the footgun class
- **Bincode 2.0 migration at the same time:** separate concern. Stay on bincode 1.x for this initiative; bincode 2.0 is a follow-up that also makes `PluginError::details` more natural

## Implementation Plan

1. Rewrite `fidius-core/src/wire.rs` — single bincode path, no cfg, no WireFormat usage
2. Remove `WireFormat` enum from `fidius-core/src/descriptor.rs`
3. Remove `wire_format: u8` field from `PluginDescriptor`, bump `ABI_VERSION` to 3
4. Remove `wire_format_kind()` method
5. Update `fidius-macro/src/interface.rs` descriptor builder — no wire_format field
6. Remove `WireFormatMismatch` from `fidius-host/src/error.rs`
7. Remove wire-format validation from `fidius-host/src/loader.rs`
8. Remove `expected_wire` / `wire_format` builder methods from `fidius-host/src/host.rs`
9. Remove `wire_format` field from `PluginInfo` in `fidius-host/src/types.rs`
10. Update CLI `inspect` output
11. Audit `serde_json` usage in `fidius-core` — remove from manifest if unused
12. Drop `fidius-core/tests/layout_and_roundtrip.rs::wire_debug_produces_json` and `wire_release_produces_bincode` tests
13. Update remaining tests that exercised the `WireFormat` enum
14. Update spec doc — single wire format

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- No `cfg(debug_assertions)` references in `fidius-core/src/wire.rs`
- No `WireFormat` enum exists
- `PluginDescriptor` has no `wire_format` field; `ABI_VERSION` is 3
- Release host loads debug plugin successfully; debug host loads release plugin successfully
- `fidius-host` does not validate wire format (there's only one)
- `serde_json` is removed from `fidius-core/Cargo.toml` if no remaining uses
- All existing tests pass (minus the two removed wire-format-distinction tests)