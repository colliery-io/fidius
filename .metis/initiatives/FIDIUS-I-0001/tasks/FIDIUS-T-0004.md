---
id: implement-pluginerror-status-codes
level: task
title: "Implement PluginError, status codes, and FNV-1a interface hashing"
short_code: "FIDIUS-T-0004"
created_at: 2026-03-29T00:33:52.696321+00:00
updated_at: 2026-03-29T00:46:08.751819+00:00
parent: FIDIUS-I-0001
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0001
---

# Implement PluginError, status codes, and FNV-1a interface hashing

## Parent Initiative

[[FIDIUS-I-0001]]

## Objective

Implement three related pieces: (1) the `PluginError` type that plugins return to signal business logic errors, (2) the status code constants used as FFI return values, and (3) the FNV-1a hashing utility that computes `interface_hash` from method signatures at compile time.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `PluginError` struct with `code: String`, `message: String`, `details: Option<serde_json::Value>` — derives `Serialize`, `Deserialize`, `Debug`, `Clone`
- [ ] `PluginError` implements `std::error::Error` and `Display`
- [ ] `PluginError::new(code, message)` and `PluginError::with_details(code, message, details)` constructors
- [ ] Status code constants: `STATUS_OK = 0`, `STATUS_BUFFER_TOO_SMALL = -1`, `STATUS_SERIALIZATION_ERROR = -2`, `STATUS_PLUGIN_ERROR = -3`, `STATUS_PANIC = -4`
- [ ] `interface_hash(signatures: &[&str]) -> u64` function implementing FNV-1a
- [ ] Hash function sorts signatures before hashing (order-independent)
- [ ] Hash function is `const`-compatible or usable in `const` contexts (for compile-time computation by the macro)
- [ ] Known test vectors: same signatures in different order produce same hash; different signatures produce different hashes

## Implementation Notes

### Technical Approach

Files:
- `fidius-core/src/error.rs` — `PluginError`, `WireError` (if not already in wire.rs)
- `fidius-core/src/status.rs` — status code constants
- `fidius-core/src/hash.rs` — FNV-1a implementation

FNV-1a algorithm:
```
hash = 0xcbf29ce484222325 (FNV offset basis for 64-bit)
for each byte in input:
    hash ^= byte
    hash *= 0x100000001b3 (FNV prime for 64-bit)
```

The function should accept a slice of signature strings, sort them, concatenate with a separator, and hash the result. Must be deterministic across compilations.

### Dependencies
- FIDIUS-T-0001 (workspace)

## Status Updates

- **2026-03-29**: Implemented across 3 files: `error.rs` (PluginError with Serialize/Deserialize/Error/Display), `status.rs` (5 status code constants), `hash.rs` (const `fnv1a` + `interface_hash` with sort). 5 unit tests pass: empty input, known vector, order independence, sensitivity, collision resistance.