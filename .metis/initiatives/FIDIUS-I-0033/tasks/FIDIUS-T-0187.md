---
id: p4-proptest-wire-format-round-trip
level: task
title: "P4 — proptest + wire-format round-trip invariants over arbitrary Value trees"
short_code: "FIDIUS-T-0187"
created_at: 2026-06-23T17:32:42.615638+00:00
updated_at: 2026-06-23T23:00:34.622643+00:00
parent: FIDIUS-I-0033
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0033
---

# P4 — proptest + wire-format round-trip invariants over arbitrary Value trees

## Parent Initiative

[[FIDIUS-I-0033]] — Phase 4 (property tests).

## Objective

Add `proptest` as a dev-dependency and assert the wire-format round-trip invariant
`decode(encode(v)) == v` over arbitrary `Value` trees.

## Acceptance Criteria

## Acceptance Criteria

- [x] `proptest` is a dev-dependency.
- [x] A property test asserts `decode(encode(x)) == x` over arbitrary nested trees
      (concrete payloads over the wire + the `Value` bridge — see status for why not
      `Value`-over-bincode).
- [x] Generation covers the full payload shape (nesting + all the variants `Value`
      models).

## Implementation Notes

Generative tests subsume many hand-written cases and tend to surface the exact
inputs fuzzing/mutation also probe.

### Dependencies
None hard — parallelizable after Phase 1.

## Status Updates

**2026-06-23 — implemented + verified.** Added `proptest` dev-dep to `fidius-guest`
and `crates/fidius-guest/tests/proptest_wire.rs` with two properties over an arbitrary
recursive `Concrete` tree (bool/int/uint/float/text/bytes/unit/option/list/map/pair —
mirrors the shapes `Value` models):
- `wire_bincode_roundtrip`: `deserialize(serialize(x)) == x` — the core wire contract.
- `value_bridge_roundtrip`: `from_value(to_value(&x)) == x` — `Value` as a lossless
  bridge for concrete payloads.

**Key finding (corrected the task's premise):** the literal "`decode(encode(v)) == v`
over arbitrary **`Value`** trees" is impossible — the bincode wire is not
self-describing and `Value::Deserialize` uses `deserialize_any`, so `Value` never
crosses the bincode wire (only concrete user types do). And `from_value(to_value(&Value))`
is *lossy* (ints normalize, `Variant` doesn't survive), so even the in-memory
self-round-trip isn't an identity. So the invariants are asserted over an arbitrary
**concrete** tree instead — which is what actually crosses the wire. (Same finding
drove the T-0181 `wire_value` fuzz-target correction.)

Floats: NaN filtered from generation (`NaN != NaN` would break structural equality;
bincode itself preserves the bits). Both properties pass (default 256 cases each).
fmt/clippy clean.