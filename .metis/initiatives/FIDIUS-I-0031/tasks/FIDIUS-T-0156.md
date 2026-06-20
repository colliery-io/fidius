---
id: pc-5-docs-production-connector
level: task
title: "PC.5 — Docs + production-connector example"
short_code: "FIDIUS-T-0156"
created_at: 2026-06-20T15:39:23.504864+00:00
updated_at: 2026-06-20T16:28:56.585291+00:00
parent: FIDIUS-I-0031
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0031
---

# PC.5 — Docs + production-connector example

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0031]]

## Objective **[REQUIRED]**

Tie the arc together with docs + a runnable **production-connector example**: extend `examples/` (or a documented wasm fixture + test) with a connector that **streams typed records** (PC.2) using **rich types** (PC.1) and makes a **time-boxed HTTP** call (PC.3). Update the connector/streaming docs to reflect records-streaming + rich types + timeouts.

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

- [ ] A runnable example (or a documented fixture + test) demonstrates typed-record streaming + rich types + an http timeout together.
- [ ] Docs updated: the "primitives only" streaming caveat is gone; rich types + http timeouts are documented; mkdocs nav updated if a page was added.
- [ ] `examples/README.md` (and the host-application how-to) reference the new example.
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

**DONE (commit b3621ee).** Added `examples/05_record_stream` — a runnable host example
streaming rich-typed records (an `Event` with a `HashMap` field) via `call_streaming`
(covers PC.1 rich types + PC.2 record streaming at the host-composition level; runs
green). New `docs/how-to/production-connector.md` ties the arc together: a WASM
connector using rich types + typed-record streaming + time-boxed HTTP, each piece
pointing at its worked fixture/test (records-greeter `tally`, records-stream, macro-fetcher
`fetch_timeout`). Added to mkdocs nav; `examples/README` + the host-application how-to
reference the new example + how-to. The "primitives only" streaming caveat was already
removed in PC.2. Default suite (65) + lint + example run green. (Note: a single fully-
runnable example can't combine the WASM-only HTTP piece with the in-process examples/
style, so HTTP is shown via the how-to + the macro-fetcher test — documented honestly.)