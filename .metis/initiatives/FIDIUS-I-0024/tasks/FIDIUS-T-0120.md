---
id: p-2-wasm-per-call-optimization
level: task
title: "P.2 — WASM per-call optimization: cache InstancePre + typed raw-bytes path"
short_code: "FIDIUS-T-0120"
created_at: 2026-06-17T16:38:44.884553+00:00
updated_at: 2026-06-17T16:39:27.239563+00:00
parent: FIDIUS-I-0024
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0024
---

# P.2 — WASM per-call optimization: cache InstancePre + typed raw-bytes path

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0024]]

## Objective **[REQUIRED]**

Adopt the WASM per-call optimizations the benchmark uncovered, before release: cache the `InstancePre` (build the WASI `Linker` once, not per call) and route `#[wire(raw)]` bytes through wasmtime's typed (bulk-copy) call.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `WasmComponentExecutor` builds the `Linker`+`InstancePre` once in a shared `build()` constructor; `instantiate()` only creates a fresh `Store` + `instance_pre.instantiate` (per-call isolation kept).
- [x] `call_raw` uses `func.typed::<(Vec<u8>,), (Vec<u8>,)>()` (bulk memcpy) instead of a `Val::List` of per-byte `Val::U8`.
- [x] wasm suite green (11 ok); native unaffected (wasm-gated); benchmark re-run + perf doc updated with before/after.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.** Both optimizations landed in `crates/fidius-host/src/executor/wasm.rs`. Re-benchmarked (dev machine, medians): `add` ~90–124 µs → **~24 µs** (~4–5×, InstancePre cache); 256 KiB echo ~6.7 ms → **~120 µs** (~55×, typed raw path). WASM now matches a local HTTP/TCP microservice on latency while keeping the sandbox + zero standing-process footprint; cdylib still ~34 ns (2–3 orders ahead). perf doc updated. Adopted pre-release.

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