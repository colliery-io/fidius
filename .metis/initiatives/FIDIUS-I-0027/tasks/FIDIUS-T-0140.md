---
id: e2-wasi-http-egress-fetcher-wasm
level: task
title: "E2 — wasi:http egress: fetcher wasm fixture + mock-server integration tests"
short_code: "FIDIUS-T-0140"
created_at: 2026-06-19T19:26:09.537722+00:00
updated_at: 2026-06-19T19:26:09.537722+00:00
parent: FIDIUS-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: FIDIUS-I-0027
---

# E2 — wasi:http egress: fetcher wasm fixture + mock-server integration tests

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0027]]

## Objective **[REQUIRED]**

End-to-end proof of the [[FIDIUS-T-0139]] mechanism: a real WASM guest that imports `wasi:http/outgoing-handler` and makes an outbound GET, driven against a **local mock HTTP server**, with the *test* supplying the `EgressPolicy`. Depends on [[FIDIUS-T-0139]].

**Approach:**
- A `fetcher` wasm fixture (`tests/wasm-fixtures/fetcher`, Rust component via cargo-component) exporting e.g. `fetch(url: string) -> result<string, ...>` implemented over `wasi:http` outgoing-handler.
- A local mock server in-test (e.g. a `tokio` TcpListener / tiny hyper service on `127.0.0.1:<ephemeral>`), no external network.
- Tests parameterize the `EgressPolicy` the host supplies.

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

- [ ] A `fetcher` wasm component fixture that does a real outbound GET via `wasi:http`; committed `.wasm` (gitignored like the others) + `build.sh`; test skips if absent.
- [ ] **Allowed**: host supplies a permissive `EgressPolicy` (allows `127.0.0.1`) → guest GET to the mock server returns the body/status. The worked reference policy (allow-list + IP denylist, with a loopback opt-in) lives here / in docs, not as a fidius module.
- [ ] **Denied**: host supplies a deny `EgressPolicy` → the guest's request is rejected (guest sees an HTTP error), mock server receives nothing.
- [ ] **Fail-closed**: package declares `http` but host supplies **no** policy → instantiation/call fails closed (no egress).
- [ ] **No-capability**: package doesn't declare `http` → `wasi:http` import unresolved → fails closed.
- [ ] Runs offline in CI (mock server on loopback, ephemeral port); no external network.

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