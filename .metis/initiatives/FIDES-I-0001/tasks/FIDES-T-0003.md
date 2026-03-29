---
id: implement-wire-format-module-with
level: task
title: "Implement wire format module with JSON/bincode cfg switch"
short_code: "FIDES-T-0003"
created_at: 2026-03-29T00:33:51.284150+00:00
updated_at: 2026-03-29T00:43:06.402003+00:00
parent: FIDES-I-0001
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDES-I-0001
---

# Implement wire format module with JSON/bincode cfg switch

## Parent Initiative

[[FIDES-I-0001]]

## Objective

Implement the `fides_core::wire` module that provides `serialize()` and `deserialize()` functions. In debug builds (`cfg(debug_assertions)`), these use `serde_json` for human-readable output. In release builds, they use `bincode` for compact/fast serialization. Also expose a `WIRE_FORMAT` constant so the descriptor can encode which format is in use.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `wire::serialize<T: Serialize>(val: &T) -> Result<Vec<u8>, WireError>` works with JSON in debug, bincode in release
- [ ] `wire::deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WireError>` works with JSON in debug, bincode in release
- [ ] `wire::WIRE_FORMAT: WireFormat` constant returns `WireFormat::Json` in debug, `WireFormat::Bincode` in release
- [ ] `WireError` type wraps both `serde_json::Error` and `bincode::Error` with `From` impls
- [ ] `WireError` implements `std::error::Error` and `Display`
- [ ] Both paths compile and work (testable via `cargo test` and `cargo test --release`)

## Implementation Notes

### Technical Approach

File: `fides-core/src/wire.rs`

```rust
#[cfg(debug_assertions)]
pub const WIRE_FORMAT: WireFormat = WireFormat::Json;

#[cfg(not(debug_assertions))]
pub const WIRE_FORMAT: WireFormat = WireFormat::Bincode;
```

Two `mod` blocks gated by `cfg(debug_assertions)`, each implementing the same public API. The `WireError` enum has variants for both serde_json and bincode errors regardless of build profile (so the type signature is stable).

### Dependencies
- FIDES-T-0001 (workspace with serde, serde_json, bincode deps)

## Status Updates

- **2026-03-29**: Implemented in `fides-core/src/wire.rs`. `WireError` uses thiserror with `From` impls for both serde_json and bincode errors. `WIRE_FORMAT` constant and serialize/deserialize functions cfg-gated. Compiles clean in dev profile.