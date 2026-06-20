---
id: bd-1-macro-ir-accept-stream-in
level: task
title: "BD.1 — macro/IR: accept Stream in both arg and return + dual hash marker"
short_code: "FIDIUS-T-0166"
created_at: 2026-06-20T22:21:09.024837+00:00
updated_at: 2026-06-20T22:32:08.761537+00:00
parent: FIDIUS-I-0032
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0032
---

# BD.1 — macro/IR: accept Stream in both arg and return + dual hash marker

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0032]]

## Objective **[REQUIRED]**

Macro/IR foundation (ADR-0010): accept a trait method with **both** a `Stream<In>`
argument and a `Stream<Out>` return — today each alone is recognized but co-occurrence
isn't routed. The IR carries both `client_stream_item` (arg) and `stream_item` (return);
`build_signature_string`/`signature_string` emits **both** markers (`<stream` + `!stream`)
so the interface hash is distinct from unary/server/client; the macro dispatch recognizes
"bidirectional" and routes to per-backend codegen (scaffold only — BD.2/3/4 fill it).
Unit test: a bidi method's hash differs from the same method as server-only and as
client-only. Depends on CS2.1 (`<stream` marker) + ST (`!stream`).

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

**DONE (commit 144f0cd).** The foundation was largely already in place: the IR carries
both `client_stream_item` (arg) and `stream_item` (return) independently, and
`build_signature_string` already passes both `stream_item.is_some()` and `client_streaming`
to the hash — so a bidi method already hashes as `...!stream<stream`, distinct from
unary/server/client (CS2.1's `<stream` and server-streaming's `!stream` never interfered).
Added: (1) `streaming_markers_are_distinct` now covers the bidi case (asserts the dual
marker + pairwise-distinct hashes across all four shapes); (2) a clean early rejection in
`generate_plugin_impl` — `Err` at the method span before any codegen, so the descriptor
never references a stubbed shim — covering cdylib/WASM/Python until BD.2/3/4; (3) the
`bidi_stream_not_wired` compile-fail proves the guard fires cleanly. The interface side
needs no guard: the vtable's `ClientStreamFn` shape is valid for bidi and `generate_client`
already skips streaming methods, so a bidi *interface* compiles (hash/descriptor valid) —
only the *impl* shim is gated. Default 68 + lint green. Next: BD.2 lifts the cdylib gate +
wires `BidiStreamFn` shim + host path.