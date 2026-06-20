---
id: bd-4-python-host-fed-iterator
level: task
title: "BD.4 — Python host-fed iterator(input)+generator(output) + E2E"
short_code: "FIDIUS-T-0169"
created_at: 2026-06-20T22:21:13.724451+00:00
updated_at: 2026-06-20T22:55:35.605423+00:00
parent: FIDIUS-I-0032
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0032
---

# BD.4 — Python host-fed iterator(input)+generator(output) + E2E

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0032]]

## Objective **[REQUIRED]**

Python bidirectional (ADR-0010): the method **receives** the `HostFedStream` iterator
(input, CS2.4) AND **returns** a generator (output, ST). Host call path: feed the producer
items + pull the returned generator (which pulls the host-fed iterator) → encode the output
sequence. Fixture (`def transform(rows): for r in rows: yield ...`) + E2E: host produces
`In`, pulls `Out`, asserts the transform. Single-threaded Python — the generator pulling the
iterator is the re-entrancy. Depends on BD.1.

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

**DONE (commit 8812577).** Python bidirectional works end to end — a literal composition of
CS2.4's input and ST's output.
- fidius-python: `call_bidi_streaming_start` passes the `HostFedStream` iterator (CS2.4) as
  the method's first positional arg, then wraps the returned generator as the output
  `PythonStream` (ST's `try_iter`). The generator pulling the iterator is the re-entrancy
  (single-threaded Python).
- host: extracted the server-streaming Python pump into `pump_python_stream` (shared);
  added `Pyo3Executor::call_bidi_streaming` (Value items → JSON → start → pump) + the
  `PluginHandle` python arm.
- New `py-bidi-stream` fixture (`def transform(rows): for r in rows: yield r*2`) +
  `python_bidi_stream_e2e`: host produces [1..=5] → [2,4,6,8,10].
- Default 71 + python server/client-streaming regression (pump refactor) + lint green.

**Bidirectional streaming now works on cdylib, WASM, and Python — all E2E-proven.** Each
backend was a composition of its already-shipped client- (input) and server- (output)
streaming halves, exactly as ADR-0010 predicted. Only BD.5 (docs) remains.