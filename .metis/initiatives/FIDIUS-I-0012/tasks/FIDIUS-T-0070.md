---
id: un-defer-typed-client-codegen-in
level: task
title: "Un-defer typed Client codegen in plugin_interface"
short_code: "FIDIUS-T-0070"
created_at: 2026-04-17T17:51:50.214444+00:00
updated_at: 2026-04-17T18:01:19.087615+00:00
parent: FIDIUS-I-0012
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0012
---

# Un-defer typed Client codegen in plugin_interface

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0012]]

## Objective

Un-defer the dormant `_generate_client_deferred` in `fidius-macro/src/interface.rs`, fix its bugs (hardcoded `fidius_host::` paths, wrong wire encoding for single-arg methods, unnecessary `.clone()` in multi-arg), gate its emission with `#[cfg(feature = "host")]`, and wire it into `generate_interface`'s output so every `#[plugin_interface]` declaration now yields a typed `{Trait}Client` struct when downstream crates enable the `host` feature.

Depends on FIDIUS-T-0069 (which added the `host` feature and re-exports `PluginHandle`/`CallError` via `fidius`). End-to-end validation deferred to FIDIUS-T-0071.

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

- [x] `_generate_client_deferred` renamed to `generate_client`, `#[allow(dead_code)]` removed
- [x] All `fidius_host::` hardcoded paths replaced with `#crate_path::` (respects `crate = "..."` override for white-label)
- [x] Uniform tuple encoding in serialized args: `&(#(#arg_names,)*)` matches plugin-side shim for 0/1/N arg counts (fixes single-arg wire mismatch bug)
- [x] `.clone()` removed from multi-arg path — args passed by reference through
- [x] Client struct + impl gated with `#[cfg(feature = "host")]`
- [x] `generate_client` wired into `generate_interface`'s output (outside the companion module)
- [x] Existing tests pass — Client is invisible when `host` feature not enabled
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

- **2026-04-17**: Completed. Single-file change: `fidius-macro/src/interface.rs`.
  - Replaced `_generate_client_deferred` (lines ~280-370) with new `generate_client` function. Documented the uniform tuple encoding invariant (matches plugin-side shim deserialization in `impl_macro.rs::generate_shims`).
  - Bugfixes over the deferred version:
    1. Wire encoding: old code serialized single-arg as `&arg` (bare `T`), but plugin-side deserializes as `(T,)` (1-tuple). New code always emits `&(#(#arg_names,)*)` which collapses to `&()`, `&(a,)`, or `&(a, b,)` naturally.
    2. Crate path: old code hardcoded `fidius_host::PluginHandle` / `fidius_host::CallError`. New code uses `#crate_path::` — respects the `crate = "..."` override on `#[plugin_interface]`, supports white-label via the fidius facade.
    3. `.clone()` removal: old code cloned each arg for the tuple builder. New code passes refs directly; serde serializes `&T` identically to `T` (bincode doesn't carry type metadata, so wire bytes match).
  - Feature gating: wrapped Client struct + impl in `#[cfg(feature = "host")]`. Cdylibs without the feature see nothing; host apps with the feature see a typed Client.
  - Wired into `generate_interface`: `let client = generate_client(ir);` then `#client` appears at the end of the emitted tokens (outside the companion module — Client is user-visible).
  - Removed outdated "typed client for host-side calling" language from companion module doc-comment.

  `angreal test` — 23 test-result groups pass (Client gated out via cfg, so existing tests exercise the original code paths unchanged). `angreal lint` clean.

  End-to-end Client validation (compile + runtime) is FIDIUS-T-0071's job, which migrates `fidius-host/tests/integration.rs` to use the generated Client.