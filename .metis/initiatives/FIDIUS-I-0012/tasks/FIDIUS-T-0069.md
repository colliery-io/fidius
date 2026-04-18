---
id: fidius-host-feature-plumbing-dep
level: task
title: "fidius host feature plumbing + dep-graph verification"
short_code: "FIDIUS-T-0069"
created_at: 2026-04-17T17:51:48.692648+00:00
updated_at: 2026-04-17T17:57:09.316718+00:00
parent: FIDIUS-I-0012
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0012
---

# fidius host feature plumbing + dep-graph verification

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0012]]

## Objective

Add a `host` feature to the `fidius` facade crate that opt-in activates `fidius-host` as a dependency and re-exports its key types (`PluginHandle`, `CallError`, etc.). Plugin cdylibs do NOT enable this feature, so they do not pull `libloading` or other host-only deps. Back the guarantee with a regression test that runs `cargo tree` on `test-plugin-smoke` and asserts `libloading` is absent. Unblocks FIDIUS-T-0070 (Client codegen) and FIDIUS-T-0071 (scaffolds).

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

- [x] `fidius/Cargo.toml` declares `host = ["dep:fidius-host"]` feature and optional `fidius-host` dep
- [x] `fidius/src/lib.rs` re-exports `PluginHandle`, `CallError`, `LoadError`, `LoadPolicy`, `PluginHost`, `PluginInfo` under `#[cfg(feature = "host")]`
- [x] New regression test `fidius-host/tests/plugin_dep_graph.rs` asserts `test-plugin-smoke` does not pull `libloading` in its dep tree
- [x] `angreal test` — all suites pass including the new dep-graph test
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

- **2026-04-17**: Completed. Files changed:
  - `fidius/Cargo.toml` — added `host = ["dep:fidius-host"]` feature; added optional `fidius-host` dep
  - `fidius/src/lib.rs` — added `#[cfg(feature = "host")]` re-export for `CallError`, `LoadError`, `LoadPolicy`, `PluginHandle`, `PluginHost`, `PluginInfo`
  - `fidius-host/tests/plugin_dep_graph.rs` (new) — runs `cargo tree -p test-plugin-smoke --edges normal` and asserts `libloading` is not in the output

  The dep-graph test passes today; it serves as a regression guard so future changes that accidentally make `fidius-host` a hard dep of `fidius` (or that enable `host` by default) fail loudly.

  `angreal test` — all suites pass. `angreal lint` clean.

  Note: the re-export set is minimal for unblocking T-0070 (typed Client needs `PluginHandle` + `CallError`). Additional re-exports from `fidius-host` can be added under the same `host` cfg as callers need them, without a feature proliferation.