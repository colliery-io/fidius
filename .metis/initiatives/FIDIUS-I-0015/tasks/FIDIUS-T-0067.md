---
id: add-callerror-invalidmethodindex
level: task
title: "Add CallError::InvalidMethodIndex; replace misused NotImplemented in bounds check"
short_code: "FIDIUS-T-0067"
created_at: 2026-04-17T16:57:49.888460+00:00
updated_at: 2026-04-17T17:40:05.604892+00:00
parent: FIDIUS-I-0015
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0015
---

# Add CallError::InvalidMethodIndex; replace misused NotImplemented in bounds check

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0015]]

## Objective

Replace the semantically-wrong use of `CallError::NotImplemented` for out-of-range vtable indices with a new dedicated `CallError::InvalidMethodIndex { index, count }` variant. Clarify `NotImplemented`'s doc to restrict it to the capability-bit-not-set case. See initiative FIDIUS-I-0015 for full context.

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

- [x] `CallError::InvalidMethodIndex { index: usize, count: u32 }` variant added to `fidius-host/src/error.rs`
- [x] `PluginHandle::call_method` bounds check returns the new variant instead of `NotImplemented`
- [x] Integration test `out_of_bounds_vtable_index_returns_error` asserts `InvalidMethodIndex { index: 99, .. }`
- [x] Doc comment on `NotImplemented` clarifies it is capability-bit-not-set only
- [x] `docs/reference/errors.md` updated with the new variant and clarified `NotImplemented` description
- [x] `angreal test` passes; `angreal lint` clean

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
  - `fidius-host/src/error.rs` — added `InvalidMethodIndex { index: usize, count: u32 }` variant; added doc comment to `NotImplemented` clarifying it is only for capability-bit-not-set case
  - `fidius-host/src/handle.rs:114-117` — bounds check returns `InvalidMethodIndex` with the attempted index and actual method count
  - `fidius-host/tests/integration.rs:212-233` — updated `out_of_bounds_vtable_index_returns_error` to assert `InvalidMethodIndex { index: 99, .. }`
  - `docs/reference/errors.md` — added `InvalidMethodIndex` and `UnknownStatus` variant docs; clarified `NotImplemented` trigger text

  Verified `fidius-macro/src/interface.rs:310` (deferred client generator) still uses `NotImplemented { bit: #cap_bit }` — this is the correct semantic use (capability bit not set), no change needed.

  `docs/api/rust/fidius-host/error.md` is auto-generated (plissken); intentionally not edited — will regenerate on next doc build.

  `angreal test` all passing. `angreal lint` clean. One flaky e2e signing test (`lenient_policy_still_enforces_signatures`) reproduced intermittently on parallel test runs — pre-existing, unrelated to this change, passes on re-run.