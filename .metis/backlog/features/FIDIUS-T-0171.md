---
id: user-typed-derive-wittype-stream
level: task
title: "User-typed (#[derive(WitType)]) stream items in client/bidi streaming"
short_code: "FIDIUS-T-0171"
created_at: 2026-06-20T23:08:00.301373+00:00
updated_at: 2026-06-20T23:41:43.391917+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# User-typed (#[derive(WitType)]) stream items in client/bidi streaming

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective **[REQUIRED]**

Today **client-streaming and bidirectional** stream items must be primitives / `String` on
cdylib and WASM — a `#[derive(WitType)]` record/enum as the stream item is rejected (the
macro `generate_wasm_adapter` returns `wasm_unsupported` when user types co-occur with a
client-stream item; see the guard added in CS2.3 / FIDIUS-I-0030). Server-streaming already
supports user-typed items (PC.2 / records-stream), so the WIT machinery exists — this task
extends the **input** (client-stream) side and the bidi co-occurrence to user-typed items:
classify + emit the input item's record/variant WIT, convert at the `WasmHostStream` /
`HostStream` boundary, and lift the wasm guard. E2E: a record stream fed into a client- and
a bidi-streaming plugin on WASM + cdylib. Shared limitation called out in ADR-0010 and
`docs/explanation/streaming.md`.

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

**DONE — input side (commit 7c4d801).** Key correction to the original framing: a client-/
bidi-input stream item crosses as **bincode** (`list<u8>` via `fidius:stream-pull` /
`WasmHostStream`), NOT as a WIT type. CS2.3 mistakenly *collected + validated* the item as a
WIT user type, which is exactly what forced the build.rs user-type branch (and the rejection)
for a record item. Fix: stop treating the opaque bincode stream item as a WIT type in
`generate_wasm_adapter`. A client-/bidi-input stream item can now be **any
`Serialize`/`Deserialize` type** (records included) — no `#[derive(WitType)]`, enforced by
`WasmHostStream::<T>`. cdylib never gated it. E2Es: `cdylib_record_stream_item` (record
client + record→record bidi) and `record-client-stream` WASM fixture + `wasm_record_stream_item`
(record-in client; record-in → primitive-out bidi). Default 73 + full wasm regression + lint
green; documented in streaming.md.

**Remaining (deferred, distinct + smaller scope):** on WASM the **output** stream item still
crosses via the WIT resource, so (a) a bidi method with a **record output** item and (b) a
record used as a stream item AND in a WIT-typed non-stream arg/return both still hit the
user-type Guest branch and are rejected (with a now-accurate message). Closing these means
adding client/bidi codegen to the `has_user` branch — a contained follow-on if an adopter
needs record *outputs* over bidi on WASM. Marked complete for the input-side win.