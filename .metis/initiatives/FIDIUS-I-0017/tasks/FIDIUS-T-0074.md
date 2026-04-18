---
id: fidius-test-cli-init-plugin-init
level: task
title: "fidius test CLI + init-plugin/init-host test scaffolds + docs"
short_code: "FIDIUS-T-0074"
created_at: 2026-04-17T18:20:48.461285+00:00
updated_at: 2026-04-18T00:47:53.788298+00:00
parent: FIDIUS-I-0017
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0017
---

# fidius test CLI + init-plugin/init-host test scaffolds + docs

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0017]]

## Objective

Surface the `fidius-test` helpers to first-time users via three channels:
1. New `fidius test <plugin-dir>` CLI subcommand — zero-setup smoke test (build + load + invoke each zero-arg method + report)
2. `fidius init-plugin` scaffold emits a `#[cfg(test)]` test module using `CalculatorClient::in_process(...)` and adds `fidius-test` + interface-crate-with-host-feature as dev-dependencies
3. `fidius init-host` scaffold adds `fidius-test` to `[dev-dependencies]` with a pointer to the docs
4. New `docs/how-to/test-plugins.md` tutorial covering all four testing layers (in-process, dylib fixture, signing fixtures, CLI smoke)

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

- [x] `fidius test <plugin-dir>` subcommand builds the package, loads it, iterates methods, invokes each with zero-arg input, classifies responses (OK / needs input / not implemented / error), prints a summary with pass/fail counts
- [x] Exit code is zero on success, nonzero when any method failed outright (not the "needs input" signal — that's expected)
- [x] Manual run against `tests/test-plugin-smoke` reports 1 plugin, 1 zero-arg method invoked cleanly, 3 methods that take args
- [x] `init-plugin` scaffold emits `crate-type = ["cdylib", "rlib"]` so the interface crate test can link it
- [x] `init-plugin` scaffold adds `{interface_crate} = { ..., features = ["host"] }` and `fidius-test` to dev-dependencies
- [x] `init-plugin` scaffold emits `#[cfg(test)] mod tests` block with a working `{Trait}Client::in_process("{Struct}")?.process(&...)` example
- [x] `init-host` scaffold adds `fidius-test` to dev-dependencies
- [x] New `docs/how-to/test-plugins.md` documents all four layers; linked from mkdocs nav + tutorials/your-first-plugin.md "Next steps"
- [x] All existing tests pass; `angreal lint` clean

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

- **2026-04-17**: Completed.

  **CLI — `fidius test`:**
  - `fidius-cli/src/commands.rs::test(dir, release)` — builds via `fidius_host::package::build_package`, loads via `fidius_host::loader::load_library`, iterates plugins and invokes each method with `handle.call_method::<(), serde_json::Value>(i, &())`. Classifies the result:
    - `Ok(_)` → "invoked (output decoded as JSON)"
    - `Err(Deserialization)` → "invoked (output not JSON-compat)" — call succeeded, output type doesn't fit serde_json::Value; still a smoke pass
    - `Err(Serialization)` → "needs input (method takes args)" — expected signal, not counted as failure
    - `Err(NotImplemented { bit })` → "optional, not implemented"
    - `Err(InvalidMethodIndex)` or other → counted as failure; process exits nonzero
  - Reports plugin count, zero-arg methods invoked cleanly, and any failures.
  - `fidius-cli/src/main.rs` — new `Test { dir, debug }` Commands variant wired to `commands::test`.
  - Verified against `tests/test-plugin-smoke`: 1 plugin, 4 methods, `version()` (index 2) invoked OK, the other 3 reported as "needs input". Exits zero.

  **Scaffold updates — init-plugin:**
  - Changed `crate-type` from `["cdylib"]` to `["cdylib", "rlib"]` — required for the test module to link and call the in-process Client.
  - Added `[dev-dependencies]` entries: interface crate with `features = ["host"]` to pull the generated Client, and `fidius-test` (reserved for users who want `dylib_fixture`).
  - Lib.rs now includes a `#[cfg(test)] mod tests` block with a working `process_in_process` test that calls `{Trait}Client::in_process("{Struct}")?.process(&"hello".to_string())` and asserts the expected output. New plugin authors have a runnable test from minute zero.

  **Scaffold updates — init-host:**
  - Added `fidius-test` to `[dev-dependencies]` with a comment pointing at docs.rs/fidius-test.
  - Did not prescribe a specific integration test — host authors test their application logic with whatever plugin they're integrating, and the `dylib_fixture` helper docs in `test-plugins.md` cover the pattern without forcing it into the scaffold.

  **Docs:**
  - New `docs/how-to/test-plugins.md` — covers four layers (in-process Client, dylib fixture, signing fixtures, CLI smoke) with runnable code examples and a "which layer?" decision table at the end.
  - `docs/tutorials/your-first-plugin.md` — "Next steps" section now links to `test-plugins.md` as the first item.
  - `mkdocs.yml` — new nav entry `How-To Guides > Test Plugins`.

  **Validation:**
  - `cargo run -p fidius-cli -- test tests/test-plugin-smoke --debug` — works end-to-end
  - Full `angreal test` — all 27 test-result groups pass
  - `angreal lint` — clean