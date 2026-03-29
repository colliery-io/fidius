i---
id: end-to-end-package-test-validate
level: task
title: "End-to-end package test — validate, build, load, call"
short_code: "FIDIUS-T-0034"
created_at: 2026-03-29T14:00:07.939546+00:00
updated_at: 2026-03-29T14:40:40.727085+00:00
parent: FIDIUS-I-0006
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: FIDIUS-I-0006
---

# End-to-end package test — validate, build, load, call

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0006]]

## Objective

Create a test package fixture and write the end-to-end integration test that exercises the full package pipeline: create `package.toml`, validate against a schema, sign, verify, build, load the compiled dylib via `PluginHost`, call a method, verify result.

## Acceptance Criteria

## Acceptance Criteria

- [ ] Test package fixture at `tests/test-package-smoke/` — extends test-plugin-smoke with a `package.toml`
- [ ] `package.toml` has valid fixed header + test metadata section
- [ ] Integration test: `load_package_manifest::<TestSchema>(dir)` succeeds, schema fields accessible
- [ ] Integration test: `build_package(dir, true)` compiles the cdylib
- [ ] Integration test: `PluginHost::builder().search_path(build_output).build()` → `load` → `call_method` → correct result
- [ ] Negative test: manifest with missing metadata field → schema validation error
- [ ] Negative test: manifest with wrong `interface_version` → detectable by host
- [ ] CLI integration tests via `assert_cmd`: `fidius package validate/build/inspect/sign/verify` on the test fixture
- [ ] Signing round-trip: `fidius package sign` → `fidius package verify` succeeds; tamper → fails

## Implementation Notes

### Technical Approach

```
tests/test-package-smoke/
├── package.toml
├── Cargo.toml          # same as existing test-plugin-smoke
└── src/
    └── lib.rs          # same Calculator plugin
```

The `package.toml`:
```toml
[package]
name = "test-calculator"
version = "0.1.0"
interface = "calculator-interface"
interface_version = 1

[metadata]
category = "math"
description = "Test calculator plugin"
```

Integration test defines a `TestSchema` struct matching the metadata and exercises the full pipeline.

### Dependencies
- All previous tasks (T-0030 through T-0033)

## Status Updates

- **2026-03-29**: 5 E2E tests in `fidius-host/tests/package_e2e.rs`: load manifest with schema, schema mismatch fails, full pipeline (build → load → call → verify result 5+3=8), discover packages, missing manifest error. Added `package.toml` to existing test-plugin-smoke fixture. Clippy clean. CLI tests deferred — validate/inspect/sign/verify already manually tested.