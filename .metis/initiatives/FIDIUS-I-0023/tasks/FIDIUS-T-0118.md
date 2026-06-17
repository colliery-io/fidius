---
id: w-6-lift-wittype-v1-limits-struct
level: task
title: "W.6 — Lift WitType v1 limits: struct/multi-field variant cases (synthetic records) + multi-file modules"
short_code: "FIDIUS-T-0118"
created_at: 2026-06-17T13:55:26.334763+00:00
updated_at: 2026-06-17T14:08:56.788655+00:00
parent: FIDIUS-I-0023
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0023
---

# W.6 — Lift WitType v1 limits: struct/multi-field variant cases (synthetic records) + multi-file modules

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0023]]

## Objective **[REQUIRED]**

Remove the two WitType v1 limits: support struct-style (named-field) enum variant cases (via synthesized payload records) and `#[derive(WitType)]` types in submodules (inline + external files).

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] Struct variant cases → synthetic `record <enum>-<case>` + `case(<enum>-<case>)`, with field-by-field conversions both ways. Multi-field tuple cases rejected with a clear message (serde→seq can't round-trip as a WIT record).
- [x] `generate_from_path` walks inline `mod m {}` + external `mod m;` (`m.rs`/`m/mod.rs`), tracking module paths so conversions target `crate::<mod::path>::<T>`. `fidius-build` + `fidius wit` use it.
- [x] E2E: `records-greeter` extended — `Point` in a `geom` submodule + a `Triangle { base, height }` struct variant — builds to a component (record + synthetic record + variant) and round-trips through `load_wasm`.
- [x] Docs updated (struct cases, submodules, multi-field-tuple rejection, flat-namespace caveat). fidius-wit 13 / fidius-build 3 / wasm E2E 2 / native 46 / wasm 11; lint green.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.** Commit `3cd3325`. Both limits lifted + verified E2E (module + struct variant through the real wasm build + host round-trip). Remaining deliberate constraints: multi-field tuple variant cases (use a struct case); record/variant names share one flat WIT namespace (unique names per interface).

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

## Acceptance Criteria **[REQUIRED]**

- [ ] {Specific, testable requirement 1}
- [ ] {Specific, testable requirement 2}
- [ ] {Specific, testable requirement 3}

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

## Status Updates **[REQUIRED]**

*To be added during implementation*