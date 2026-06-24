---
id: p-1-cross-backend-latency
level: task
title: "P.1 — Cross-backend latency benchmark (cdylib/wasm vs TCP/UDS/HTTP) + perf doc"
short_code: "FIDIUS-T-0119"
created_at: 2026-06-17T15:25:50.643916+00:00
updated_at: 2026-06-17T15:26:32.484859+00:00
parent: FIDIUS-I-0024
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0024
---

# P.1 — Cross-backend latency benchmark (cdylib/wasm vs TCP/UDS/HTTP) + perf doc

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0024]]

## Objective **[REQUIRED]**

A criterion benchmark comparing the plugin backends (cdylib, wasm JIT/AOT) to microservice-style transports (localhost TCP, Unix-socket IPC, HTTP/1.1) on the same ops, plus a perf doc with an honest reading.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `crates/fidius-host/benches/backends.rs` (criterion, `required-features=["wasm"]`): `add(i64,i64)` + `echo(bytes)` at 64 B/4 KiB/256 KiB across cdylib, wasm JIT, wasm AOT, localhost TCP, Unix socket, HTTP/1.1 (persistent connections; network baselines are lower bounds).
- [x] `docs/explanation/performance.md` with the result tables + honest reading (cdylib wins 2–3 orders; WASM not faster than local microservice — fresh-instance-per-call + Value copy; plugins win footprint/ops); in the mkdocs nav.
- [x] Reproducible: `cargo bench -p fidius-host --features wasm --bench backends`.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.** Benchmark + doc done. Findings: cdylib ~42 ns/call (2–3 orders faster than any local transport); WASM ~86–124 µs/call (not faster than local HTTP ~19 µs / UDS ~9 µs — root cause fresh-Store-per-call + Value copy, fixable via instance reuse); 256 KiB echo cdylib ~15 µs vs WASM ~6.7 ms. The WASM instance-reuse + raw-bytes fast path is the open follow-on (initiative plan step 2).

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