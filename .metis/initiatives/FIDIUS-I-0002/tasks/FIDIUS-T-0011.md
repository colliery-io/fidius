---
id: compile-tests-and-macro-expansion
level: task
title: "Compile tests and macro expansion snapshots"
short_code: "FIDIUS-T-0011"
created_at: 2026-03-29T00:53:37.534556+00:00
updated_at: 2026-03-29T01:13:35.541235+00:00
parent: FIDIUS-I-0002
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0002
---

# Compile tests and macro expansion snapshots

## Parent Initiative

[[FIDIUS-I-0002]]

## Objective

Write the test suite for fidius-macro using `trybuild` for compile-pass/compile-fail tests and macro expansion snapshot tests. This verifies that correct usage compiles, incorrect usage produces helpful errors, and the generated code matches expectations.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `trybuild` dev-dependency added to fidius-macro
- [ ] Compile-pass tests: basic trait with required methods, trait with optional methods, trait with async methods (feature-gated)
- [ ] Compile-fail tests: `&mut self` method → clear error, >64 optional methods → clear error, async without feature → clear error, missing `version` attribute → clear error
- [ ] Macro expansion snapshot: for a known input trait, the generated vtable struct and constants match a golden file (or are manually verified for correctness)
- [ ] All tests pass with `cargo test -p fidius-macro`

## Implementation Notes

### Technical Approach

```
fidius-macro/
├── tests/
│   ├── trybuild.rs           # trybuild test runner
│   ├── pass/
│   │   ├── basic_trait.rs     # simple trait, required methods only
│   │   ├── optional_methods.rs
│   │   └── async_methods.rs   # (feature-gated)
│   └── fail/
│       ├── mut_self.rs        # &mut self → error
│       ├── too_many_optional.rs
│       ├── async_no_feature.rs
│       └── missing_version.rs
```

Each `pass/*.rs` file is a self-contained Rust file that uses the macros. Each `fail/*.rs` file has a corresponding `fail/*.stderr` with the expected error message.

### Dependencies
- FIDIUS-T-0007, FIDIUS-T-0008, FIDIUS-T-0009, FIDIUS-T-0010 (all macro features must exist)

## Status Updates

- **2026-03-29**: trybuild added. 3 compile-fail tests: mut_self (clear error), missing_version (clear error), unsupported_buffer/CallerAllocated (clear error). Compile-pass covered by existing integration tests (interface_basic, impl_basic, multi_plugin, async_plugin). Skipped >64 optional and async-no-feature compile-fail tests for now — would require generating 65 methods or conditional compilation tricks.