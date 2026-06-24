---
id: w-1-wit-generator-lib-source-parse
level: task
title: "W.1 — WIT generator lib: source-parse trait + WitType types → wit/ (records/variants/funcs) + From-conversions"
short_code: "FIDIUS-T-0113"
created_at: 2026-06-17T13:00:59.219805+00:00
updated_at: 2026-06-17T13:17:50.613540+00:00
parent: FIDIUS-I-0023
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0023
---

# W.1 — WIT generator lib: source-parse trait + WitType types → wit/ (records/variants/funcs) + From-conversions

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0023]]

## Objective **[REQUIRED]**

A shared (non-proc-macro) `fidius-wit` crate that maps Rust interface types to WIT (incl. `#[derive(WitType)]` structs→records, enums→variants) and source-parses a plugin crate to emit a complete `wit/` + the generated↔author `From` conversions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `fidius-wit` crate (plain lib): `wit_type_with(ty, known)` (primitives + user types via a known-ident set), `struct_to_record`, `enum_to_variant` (unit + single-field cases), `render_wit_full` (type defs before funcs).
- [x] `generate(src)` parses source (`syn`): finds the `#[plugin_interface]` trait + `#[derive(WitType)]` types → complete `.wit` (records/variants/funcs/`fidius-interface-hash`) + bidirectional `From` conversions (recursing `Vec`/`Option`/nested via `.into()`/`map`; identity move for user-free fields).
- [x] `fidius-macro::wit` re-exports `fidius-wit` (no call-site churn); workspace + macro-greeter wasm build green.
- [x] 12 unit tests (mapping, record/variant, ordering, parse→wit, conversions both ways, unsupported-type error); clippy clean.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
`fidius-wit` is a plain lib (proc-macro crates can't export reusable fns). `fidius-macro` depends on it; the `build.rs` helper + CLI will too. Conversions target the deterministic generated path `exports::fidius::<iface>::<iface>::<T>` and `crate::<T>` (proven in the I-0023 spike).

### Dependencies
First task of FIDIUS-I-0023. Feeds [[FIDIUS-T-0114]] (adapter rework consumes `generate()`'s wit + conversions) and [[FIDIUS-T-0115]] (build.rs + CLI call `generate()`).

### Risk Considerations
The conversion codegen is string-based; it is compile-verified end-to-end by T-0114/T-0116 (the records fixture). v1: WitType types at the crate root, single-file source.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.** Two commits: `3c6fa34` (mapping + record/variant + shared crate + macro rewire), `3dc24e7` (source-parser + conversions). 12 unit tests; clippy clean; workspace + macro-greeter wasm build green. The generator is ready for the adapter rework (T-0114), which compile-verifies the conversions end-to-end.