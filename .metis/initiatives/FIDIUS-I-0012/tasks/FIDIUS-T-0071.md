---
id: dogfood-client-in-integration
level: task
title: "Dogfood Client in integration tests + init-host scaffold + docs"
short_code: "FIDIUS-T-0071"
created_at: 2026-04-17T17:51:51.468723+00:00
updated_at: 2026-04-17T18:08:15.305922+00:00
parent: FIDIUS-I-0012
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0012
---

# Dogfood Client in integration tests + init-host scaffold + docs

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0012]]

## Objective

End-to-end validation of the typed Client (T-0070): migrate `fidius-host/tests/integration.rs` to use `CalculatorClient` for method-call tests, add a new `fidius init-host` CLI subcommand that scaffolds a host binary using the Client, update `init-interface` scaffold to declare the `host` feature, and update `docs/tutorials/your-first-plugin.md` to teach the Client pattern (imports types from interface crate, no magic indices, no duplicate struct definitions).

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

- [x] `test-plugin-smoke` is `crate-type = ["cdylib", "rlib"]` with a `host = ["fidius/host"]` feature
- [x] `fidius-host` dev-depends on `test-plugin-smoke` with `features = ["host"]` so integration tests can use `CalculatorClient`
- [x] `fidius-host/tests/integration.rs` — 4 method-call tests migrated from `handle.call_method(index, &input)` to `CalculatorClient::from_handle(handle).method(&input)`: `call_add_method_via_client`, `call_multiply_method_via_client` (optional), `call_multi_arg_add_direct_via_client` (multi-arg), `call_zero_arg_version_via_client` (zero-arg)
- [x] Input/output type duplicates (`AddInput`/`AddOutput`/`MulInput`/`MulOutput`) removed from integration.rs — imported from test-plugin-smoke instead
- [x] `fidius init-interface` scaffold includes `[features] host = ["fidius/host"]` in generated Cargo.toml
- [x] New `fidius init-host <name> --interface <path> --trait <Trait>` CLI subcommand scaffolds a host binary with Cargo.toml (interface with host feature + fidius-host) and src/main.rs (PluginHost::builder + CalculatorClient::from_handle)
- [x] `docs/tutorials/your-first-plugin.md` Step 2 and Step 4 updated: interface Cargo.toml declares `host` feature; host Cargo.toml depends on interface with `features = ["host"]`; host main.rs uses `CalculatorClient` and imports types from interface crate
- [x] `angreal test` — 10/10 integration tests pass including all 4 Client-migrated tests; all other suites unchanged
- [x] `angreal lint` clean

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates

- **2026-04-17**: Completed. Summary of changes:

  **Test fixture (dual crate-type approach, avoided splitting into separate interface crate):**
  - `tests/test-plugin-smoke/Cargo.toml` — added `rlib` to `crate-type` alongside `cdylib`; added `[features] host = ["fidius/host"]`. The cdylib build (subprocess `cargo build --manifest-path ...`) still uses default features (no host) so the dylib stays lean. The rlib (linked into fidius-host test binary as dev-dep) enables host, giving integration.rs access to `CalculatorClient`.
  - `tests/test-plugin-smoke/src/lib.rs` — added `Debug, PartialEq` to AddInput/AddOutput/MulInput/MulOutput derives for use in `assert_eq!`.
  - `fidius-host/Cargo.toml` — dev-dep `test-plugin-smoke = { path = "...", features = ["host"] }` added.

  **Integration test migration:**
  - `fidius-host/tests/integration.rs` — removed duplicate AddInput/AddOutput/MulInput/MulOutput structs; added `use test_plugin_smoke::{AddInput, AddOutput, CalculatorClient, MulInput, MulOutput};`; added a `client()` helper; renamed 4 tests to `*_via_client` and updated bodies to use `CalculatorClient::from_handle(handle).method(&input)`. Handle-specific tests (out_of_bounds, has_capability, plugin_info) retained raw `PluginHandle` usage since they test those APIs directly.

  **CLI scaffolds:**
  - `fidius-cli/src/commands.rs::init_interface` — generated Cargo.toml now includes `[features] host = ["fidius/host"]`.
  - `fidius-cli/src/commands.rs::init_host` (new) — scaffolds a host binary crate with Cargo.toml (interface crate dep with host feature + fidius-host) and src/main.rs (PluginHost::builder + CalculatorClient::from_handle + TODO call).
  - `fidius-cli/src/main.rs` — new `InitHost` Commands variant with matching args wired to `commands::init_host`.

  **Docs:**
  - `docs/tutorials/your-first-plugin.md` Step 2 — interface Cargo.toml now shows the `[features] host = ["fidius/host"]` declaration with inline explanation.
  - `docs/tutorials/your-first-plugin.md` Step 4 — host Cargo.toml depends on interface with `features = ["host"]` (dropped direct serde dep); host main.rs uses `CalculatorClient::from_handle(handle).add(&AddInput { ... })` and imports types from the interface crate. Key-points section updated: "Client is generated by #[plugin_interface]", "no magic indices", "single source of truth for input/output types", "optional methods surface as regular methods that check capability internally."

  **Validation:**
  - `angreal test` — all 23 test-result groups pass; integration tests show 10/10 including all 4 Client-migrated tests.
  - `angreal lint` clean (one rustfmt nit on `init_host` auto-fixed).

  **Notes on approach choices:**
  - Chose `crate-type = ["cdylib", "rlib"]` + feature-gated host dep over splitting test-plugin-smoke into separate interface and plugin crates. Simpler fixture, zero risk of breaking the existing subprocess cargo-build flow, and the test-plugin-smoke rlib linked into the fidius-host test binary is harmless (inventory descriptor there is ignored — tests dlopen a separately-built dylib).
  - `docs/explanation/architecture.md` unchanged — it's a high-level overview that didn't mention call_method or magic indices; the Client pattern is taught in the tutorial.
  - `docs/api/rust/fidius-host/handle.md` still mentions `call_method` — it's auto-generated (plissken) and will regenerate with the rustdoc-level changes.