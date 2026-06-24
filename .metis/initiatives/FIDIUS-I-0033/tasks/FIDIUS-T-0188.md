---
id: p4-proptest-for-wit-type-mapping
level: task
title: "P4 — proptest for WIT type mapping + multi-arg tuple packing invariants"
short_code: "FIDIUS-T-0188"
created_at: 2026-06-23T17:32:44.008740+00:00
updated_at: 2026-06-23T23:04:02.788906+00:00
parent: FIDIUS-I-0033
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0033
---

# P4 — proptest for WIT type mapping + multi-arg tuple packing invariants

## Parent Initiative

[[FIDIUS-I-0033]] — Phase 4 (property tests).

## Objective

Add `proptest` invariants for WIT type-mapping round-trips and multi-arg
tuple-pack/unpack over arbitrary arities.

## Acceptance Criteria

## Acceptance Criteria

- [x] Property tests assert WIT type-mapping round-trips (model-driven, since the
      mapping has no inverse — see status).
- [x] Property tests assert multi-arg tuple pack/unpack over arbitrary arities
      (0, 1, 2, 3, 4 — incl. the `()` and `(a,)` edge cases).

## Implementation Notes

Complements the wire round-trip (T-0187); together they cover the two main
encode/decode boundaries the macro generates against.

### Dependencies
[[FIDIUS-T-0187]] (shares the `proptest` setup).

## Status Updates

**2026-06-23 — implemented + verified.** Two pieces:

1. **WIT type mapping** — `crates/fidius-wit/tests/proptest_wit.rs` (+ `proptest`
   dev-dep). `rust_type_to_wit` has no inverse, so the round-trip is *model-driven*:
   an arbitrary `TyModel` renders BOTH its Rust source and its expected WIT from the
   same node; the test parses the Rust with `syn` and asserts `rust_type_to_wit` ==
   the expected WIT. Covers arbitrary nestings of bool / i8..i64 / u8..u64 / f32 /
   f64 / char / String / `Vec` / `Option` / `HashMap` / tuples (incl. the `s8`-style
   signed renaming and `list<tuple<…>>` map projection).

2. **Multi-arg tuple packing** — appended to `crates/fidius-guest/tests/proptest_wire.rs`:
   `pack_arity0`…`pack_arity4` assert the wire round-trips the macro's arg packing
   (`()` → 0 args, `(a,)` → 1, `(a, b, …)` → N) over arbitrary contents.

All pass (256 cases each); fmt + clippy clean. **Phase 4 (property tests) complete.**