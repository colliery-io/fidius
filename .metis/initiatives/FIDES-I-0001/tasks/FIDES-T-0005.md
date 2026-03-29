---
id: layout-assertion-tests-and-round
level: task
title: "Layout assertion tests and round-trip tests"
short_code: "FIDES-T-0005"
created_at: 2026-03-29T00:33:53.612991+00:00
updated_at: 2026-03-29T00:52:09.720386+00:00
parent: FIDES-I-0001
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDES-I-0001
---

# Layout assertion tests and round-trip tests

## Parent Initiative

[[FIDES-I-0001]]

## Objective

Write the test suite for fides-core that guards ABI stability and serialization correctness. Layout assertion tests catch accidental field reordering in `#[repr(C)]` structs. Round-trip tests verify that wire format serialization produces correct output in both JSON and bincode modes. Interface hash tests verify determinism and collision resistance.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `offset_of!` assertions for every field in `PluginRegistry` and `PluginDescriptor` — tests fail if field order changes
- [ ] `size_of` and `align_of` assertions for `PluginRegistry`, `PluginDescriptor`, `BufferStrategyKind`, `WireFormat`
- [ ] Wire format round-trip: serialize a struct → deserialize → assert equality (for both JSON and bincode paths)
- [ ] Wire format cross-mode test: JSON-serialized bytes are valid JSON (parseable by serde_json::from_slice); bincode bytes are not valid JSON
- [ ] `PluginError` round-trip: serialize → deserialize with and without `details` field
- [ ] Interface hash determinism: same inputs in different order → same hash
- [ ] Interface hash sensitivity: changing one signature character → different hash
- [ ] Interface hash known vectors: at least 3 hardcoded input→output pairs for regression
- [ ] All tests pass under `cargo test` and `cargo test --release`

## Implementation Notes

### Technical Approach

File: `fides-core/tests/layout.rs` (integration test) or `fides-core/src/descriptor.rs` `#[cfg(test)]` module.

For layout assertions, use `std::mem::offset_of!` (stabilized in Rust 1.77). Example:
```rust
assert_eq!(offset_of!(PluginDescriptor, abi_version), 0);
assert_eq!(offset_of!(PluginDescriptor, interface_name), 4); // after u32
// etc.
```

For wire format testing across debug/release: the `cargo test --release` invocation tests the bincode path. Consider also testing both paths explicitly by calling the underlying serde_json/bincode directly, not just through the cfg-switched API.

### Dependencies
- FIDES-T-0002 (descriptor types must exist)
- FIDES-T-0003 (wire format module must exist)
- FIDES-T-0004 (PluginError and hash must exist)

## Status Updates

- **2026-03-29**: 17 integration tests in `fides-core/tests/layout_and_roundtrip.rs` + 5 unit tests in hash module = 22 total. All pass in both debug (JSON) and release (bincode). Fixed `PluginError.details` — changed from `Option<serde_json::Value>` to `Option<String>` because bincode v1 doesn't support `deserialize_any`. Added `details_value()` accessor to parse back to Value.