---
id: pc-2-typed-record-stream-items
level: task
title: "PC.2 — Typed-record stream items over WASM"
short_code: "FIDIUS-T-0153"
created_at: 2026-06-20T15:39:20.111336+00:00
updated_at: 2026-06-20T15:39:20.111336+00:00
parent: FIDIUS-I-0031
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: FIDIUS-I-0031
---

# PC.2 — Typed-record stream items over WASM

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0031]]

## Objective **[REQUIRED]**

Lift the restriction that WASM **streaming items must be primitives/`String`**. Today `generate_wasm_adapter` rejects `Stream<UserType>` for `#[derive(WitType)]` records (`crates/fidius-macro/src/impl_macro.rs` ~435, "server-streaming alongside #[derive(WitType)] user types is not yet supported"). Make the streaming WIT resource's `next()` carry a **user record type** via the `emit_wit`/`build.rs` authoring path (the resource yields `option<record>` instead of `option<primitive>`), and wire the host `StreamExecutor` in `crates/fidius-host/src/executor/wasm.rs` to decode the record `Val` → `Value` per item. Builds on PC.1 ([[FIDIUS-T-0152]]) for the record field types.

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

- [ ] `fn read(...) -> fidius::Stream<MyRecord>` with `#[derive(WitType)] MyRecord` compiles via the `emit_wit` path (the rejection at `impl_macro.rs` ~435 is gone or scoped to genuinely-unsupported cases).
- [ ] A new WASM fixture streams a user record; a host E2E collects typed records (`Stream<Record>` over WASM) covering all items + drop-cancel.
- [ ] Server-streaming of primitives/`String` still works (no regression in `macro_wasm_streaming` / `wasm_streaming_e2e`).
- [ ] The "streaming items must be primitives/String" caveat is removed from `docs/explanation/streaming.md` and `configured-instances.md`.
- [ ] `angreal test` and `angreal lint` are green.

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