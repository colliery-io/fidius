---
id: bd-2-cdylib-bidistreamfn-shape
level: task
title: "BD.2 — cdylib BidiStreamFn shape + shim + host call path + re-entrancy + E2E"
short_code: "FIDIUS-T-0167"
created_at: 2026-06-20T22:21:10.899033+00:00
updated_at: 2026-06-20T22:43:13.337063+00:00
parent: FIDIUS-I-0032
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0032
---

# BD.2 — cdylib BidiStreamFn shape + shim + host call path + re-entrancy + E2E

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0032]]

## Objective **[REQUIRED]**

cdylib bidirectional (ADR-0010): a combined vtable entry `BidiStreamFn(instance,
input_producer_handle, args, *out_stream_handle) -> status` — the client-streaming
producer handle (CS2.2) is passed in, the server-streaming output stream handle (CS.1/ST)
is returned. Macro shim: build the user method's `Stream<In>` from the input handle
(`HostStream`), call the method to get `Stream<Out>`, hand back an output stream handle.
Host `call_bidi_streaming` (or extend the typed API): set the producer, get the output
handle, pull it (each `out.next()` re-enters `input_producer.next()`). E2E: a transform
plugin (e.g. running-sum / filter-double) fed host-produced `In`, host pulls `Out`,
asserts the sequence; **plus an explicit re-entrancy test** (producer re-entered during an
output pull). Depends on BD.1.

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

**DONE (commit e8c4909).** cdylib bidirectional works end to end. Key realization: the
bidi vtable slot is the **existing `ClientStreamFn` shape** — `(instance, input_handle,
args, *out, *out_len)` — only the out-bytes *meaning* differs (a stream-handle pointer,
not a value). So no new vtable type was needed.
- macro: the bidi shim (checked BEFORE the server-only branch, since bidi sets
  `stream_item` too) composes client-streaming's input (`HostStream::from_handle` →
  `Stream<In>`) with server-streaming's output (`StreamState<Out>` + `next`/`drop` →
  `FidiusStreamHandle` returned via `out_ptr`). Removed the BD.1 blanket reject; kept a
  wasm-only guard (BD.3); the native compile-fail was deleted (cdylib bidi compiles now).
- host: extracted the server-streaming pump into a shared `pump_stream_handle`; added
  `call_bidi_streaming_raw` (ClientStreamFn init → output handle → pump) + the typed async
  `PluginHandle::call_bidi_streaming<I,A,O>` (cdylib wired; WASM/Python gated).
- E2E `cdylib_bidi_stream_e2e`: a `Doubler` (lazy `from_fn` pulling input per output)
  doubles [1..=5] → [2,4,6,8,10] — the re-entrancy proof (output.next pulls input.next on
  the pump thread) — plus a drop-cancel test (pull 2 of 100, drop, no hang/leak).
- Default 69 + server/client-streaming regression (pump refactor) + lint green.

**Note (inherited from client-streaming):** the host producer eager-collects input items
(`bincode_items`) — host-side production isn't lazy yet. Plugin-side pull IS lazy
(`from_fn`). A lazy host producer is a shared follow-on. Next: BD.3 (WASM bidi).