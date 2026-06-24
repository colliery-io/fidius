---
id: wasm-record-output-items-record-in
level: task
title: "WASM: record output items + record-in-WIT-position for client/bidi streaming"
short_code: "FIDIUS-T-0175"
created_at: 2026-06-20T23:53:54.048903+00:00
updated_at: 2026-06-21T00:09:25.566192+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# WASM: record output items + record-in-WIT-position for client/bidi streaming

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective **[REQUIRED]**

Close the remaining WASM streaming user-type gaps left by T-0171: a bidi method with a
**record OUTPUT** item (crosses via the WIT resource), and client/bidi methods where a
record is used in a **WIT-typed non-stream arg/return** alongside a stream. Add client/bidi
codegen to the macro's user-type (`has_user`/build.rs) branch + make fidius-wit skip the
`Stream<T>` arg, so the WIT matches the guest adapter.

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

**DONE (commit 92ab8cc).** The macro's user-type (`has_user`/build.rs) branch now generates
client- and bidi-streaming methods (it previously rejected them). The bincode `Stream<T>`
input arg is excluded from the WIT params; non-stream args and the bidi **output** item use
the generated WIT types + `conv_expr` conversions, mirroring the primitives-branch codegen.
`fidius-wit` (build.rs WIT) skips the `Stream<T>` arg too, so the generated WIT matches the
guest adapter. Removed the obsolete rejection. New `record-stream-user-types` fixture +
`wasm_record_stream_user_types` E2E covering all three new shapes: a **record-output bidi**
(primitive in → `Stream<Row>`), a **client method with a user-typed WIT arg** alongside a
primitive stream, and a **record stream item** (bincode) where `Row` is also a WIT type.
Default 74 + full wasm user-type/streaming regression + lint green; streaming.md caveats
lifted. **The WASM streaming user-type matrix is now complete** (server/client/bidi × record
items, WIT args).