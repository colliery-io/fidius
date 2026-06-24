---
id: bd-3-wasm-import-input-resource
level: task
title: "BD.3 — WASM import(input)+resource(output) co-occurrence + codegen + E2E"
short_code: "FIDIUS-T-0168"
created_at: 2026-06-20T22:21:12.310037+00:00
updated_at: 2026-06-20T22:50:23.339571+00:00
parent: FIDIUS-I-0032
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0032
---

# BD.3 — WASM import(input)+resource(output) co-occurrence + codegen + E2E

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0032]]

## Objective **[REQUIRED]**

WASM bidirectional (ADR-0010): the component **imports** `fidius:stream-pull/pull.next`
(input, CS2.3) AND **exports** the streaming **resource** (output, WS) for the same method.
Macro wasm codegen: emit the method WIT with no stream param and a resource return; the
guest body builds the input `Stream` from `WasmHostStream` and returns the lazy output
resource. Host: set the producer + pull the output resource's `next()` (which re-enters
the `pull.next` import). Fixture + E2E: host produces `In`, pulls `Out`, asserts the
transform; re-entrancy holds across the wasm boundary. Depends on BD.1 (reuses BD.2's host
shape where shared).

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

**DONE (commit 8a5a520).** WASM bidirectional works end to end. The component BOTH imports
`fidius:stream-pull` (input, CS2.3) AND exports a streaming resource (output, WS).
- macro: a bidi wasm guest branch (before the server-only branch) builds the input
  `Stream<In>` from `WasmHostStream`, calls the method → `Stream<Out>`, returns it as the
  output resource. **The WIT machinery already composed** — the params loop skips the
  input stream via the client-streaming rule, and `(ret, stream_item)` makes the output a
  resource via the server-streaming rule — so no WIT changes were needed. Removed the BD.2
  wasm guard.
- host: extracted the server-streaming pump into `stream_with_producer(producer:
  Option<Vec<Vec<u8>>>)` — `Some` seeds `HostState.client_stream` in the **pump-owned**
  store before the export call (so it lives for the stream's lifetime, and the output
  resource's `next()` re-enters the import); `call_streaming` delegates with `None`. Added
  `call_bidi_streaming` + the `PluginHandle` wasm arm.
- New `bidi-stream` fixture + `wasm_bidi_stream_e2e`: host produces [1..=5] → guest doubles
  → [2,4,6,8,10]. Default 70 + wasm server/client-streaming regression (refactor) + lint
  green. **Bidirectional now works on cdylib + WASM.** Next: BD.4 (Python).