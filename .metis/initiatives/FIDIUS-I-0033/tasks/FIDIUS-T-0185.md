---
id: p3-cargo-mutants-scoped-pass-on
level: task
title: "P3 — cargo-mutants scoped pass on fidius-macro (IR/codegen): baseline + targeted kills"
short_code: "FIDIUS-T-0185"
created_at: 2026-06-23T17:32:40.269014+00:00
updated_at: 2026-06-23T23:19:51.974644+00:00
parent: FIDIUS-I-0033
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0033
---

# P3 — cargo-mutants scoped pass on fidius-macro (IR/codegen): baseline + targeted kills

## Parent Initiative

[[FIDIUS-I-0033]] — Phase 3 (mutation testing the core).

## Objective

Run a **scoped** `cargo-mutants` pass on `fidius-macro` (IR / codegen logic),
capture the baseline, and kill the high-value survivors with targeted tests.

## Acceptance Criteria

## Acceptance Criteria

- [x] A scoped `cargo-mutants` baseline is captured for `fidius-macro`.
- [x] High-value survivors in the IR/codegen logic are killed.
- [x] The scope boundary (what was included vs. deferred) is documented.

## Implementation Notes

Codegen/IR mutation is higher-effort than core — **bound the scope** deliberately
rather than running the whole crate.

### Dependencies
[[FIDIUS-T-0184]] (establishes the mutation workflow first).

## Status Updates

**2026-06-23 — scoped baseline + kills done.** Scope (deliberately bounded, per the
task): `cargo mutants --package fidius-macro -f '**/ir.rs'` — the IR-extraction logic
(the most unit-testable macro layer; codegen token-stream output is largely
"unviable" under mutation and deferred). Baseline: **50 mutants → 10 caught, 7 missed,
33 unviable** (2 min; the high unviable count is expected for proc-macro code).

Added 4 targeted unit tests (`ir.rs` test module) and re-ran: **7 → 3 missed**
(caught 10 → 14). Killed:
- `is_required -> true` → `is_required_reflects_optional_attr` (an `#[optional]`
  method is not required).
- `is_vec_u8 -> true` → `is_vec_u8_distinguishes_types`.
- `extract_arg_names -> vec![]` → `extract_arg_names_returns_each_non_self_arg`.
- `validate_raw_method_signature -> Ok(())` → `validate_raw_method_signature_rejects_bad_shapes`.

**Accepted (3), documented:** all three are the `parse_interface` optional-method
**counter + 64-limit bound** (`ir.rs:521` `+=`→`*=`, `522` `>`→`==`/`>=`). Killing
them needs an interface with **64–65 `#[optional]` methods** to exercise the exact
boundary — impractical/low-value for a guard on a rarely-hit limit. Left as-is.

**Out of scope (deferred):** codegen modules (`impl_macro.rs`, `interface.rs`) — the
token-stream output mutates mostly to unviable/equivalent and is far higher-effort;
the macro's codegen is covered structurally by the `trybuild` + e2e suites. fmt clean;
the only clippy noise is the pre-existing `cfg(host)` macro artifact (not my tests).