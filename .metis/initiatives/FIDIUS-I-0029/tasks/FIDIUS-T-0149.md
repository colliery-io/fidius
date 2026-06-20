---
id: ci-3-wasm-resource-construction
level: task
title: "CI.3 — WASM resource construction (config constructor, composes w/ streaming+egress)"
short_code: "FIDIUS-T-0149"
created_at: 2026-06-20T01:44:12.601074+00:00
updated_at: 2026-06-20T12:36:49.866822+00:00
parent: FIDIUS-I-0029
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0029
---

# CI.3 — WASM resource construction (config constructor, composes w/ streaming+egress)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0029]]

## Objective **[REQUIRED]**

{Clear statement of what this task accomplishes}

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

**SCOPED (not started) — fresh-session-sized, architectural. Execution map:**

Current wasm shape: `generate_wasm_adapter` (impl_macro.rs ~L329) emits a WIT world
that exports the interface as **free functions** dispatching on the cfg(wasm)
static singleton `super::#instance_name` (two code paths: primitive ~L440-514, and
`#[derive(WitType)]` user-type ~L516+). The executor (`executor/wasm.rs`)
**instantiates a fresh `Store` per call** (`fn instantiate` L515, used by call_value
L615 / call_streaming L699); streaming is the only thing that holds a store alive
across calls (the stream resource), which is the precedent to reuse.

Two viable shapes:
- **(R) WIT `resource`**: world exports a resource w/ `configure(cfg) -> handle` +
  methods on it. Cleanest Component-Model fit, reuses streaming's persistent-store
  machinery — but restructures BOTH generator paths + the executor dispatch.
- **(C) `fidius-configure(cfg: list<u8>)` export + guest config storage**: change
  the guest singleton `static INSTANCE: Type` → `OnceLock<Type>` set by configure;
  methods read it. Smaller generator change, but still needs the persistent store.

Either way the load-bearing piece is the **host executor**: a *configured* mode that
instantiates ONE persistent `Store`, calls configure once, holds `(Store, Instance)`
and dispatches method calls on it — vs today's per-call fresh store. A held `Store`
is **not `Sync`** → wrap in `Mutex` (or make the configured handle !Sync), unlike
the current Send+Sync per-call executor. Then `PluginHost`/`PluginHandle` configure
path for wasm + a wasm E2E (config bound once; composes w/ streaming + the egress
two-key gate).

RECOMMENDATION: shape **(R)**, reuse the streaming resource/store lifetime. Build:
macro adapter (both paths) → executor configured mode (Mutex<Store>) → host
configure wiring → E2E. Sibling tasks CI.4 (Python — small: construct the class
with config) + CI.5 (docs). cdylib is DONE (CI.1 3543e19 + CI.2 a122b86); 0.5.0
cuts when CI.3-5 land.