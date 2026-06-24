---
id: scoped-env-capability-for-wasm
level: task
title: "Scoped `env` capability for WASM guests (replace inherit-all-secrets)"
short_code: "FIDIUS-T-0142"
created_at: 2026-06-19T20:04:46.054380+00:00
updated_at: 2026-06-19T20:33:30.174536+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Scoped `env` capability for WASM guests (replace inherit-all-secrets)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

> Surfaced during the WASM security audit in [[FIDIUS-I-0027]]; referenced from `docs/explanation/wasm-capabilities.md`.

## Objective **[REQUIRED]**

Replace the coarse `env` capability (currently `WasiCtxBuilder::inherit_env()` — grants the guest **every** host environment variable) with a **scoped allow-list of specific variable names**, so granting `env` to an untrusted connector no longer leaks every host secret.

**Why it matters:** host env vars are where secrets live (`AWS_SECRET_ACCESS_KEY`, `DATABASE_URL`, API tokens). `inherit_env()` hands all of them to the guest with zero network calls needed — arguably a bigger hole than SSRF. It also **defeats the [[FIDIUS-I-0027]] credential-injection design**: brokering a secret in the egress hook is pointless if the same connector can read it from `env`.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

### Type
- [x] Tech Debt — security

### Priority
- [x] P2 — real secret-exposure path for the untrusted-connector tier; the grant is opt-in today, but the blast radius is all-or-nothing.

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

### 2026-06-19 — fixed ✅
- `build_wasi_ctx`: bare `env` arm removed; `env:VAR_NAME` (guard arm) → `WasiCtxBuilder::env(name, std::env::var(name))` (skipped if unset). Per-variable, never inherit-all.
- `validate_capabilities`: bare `env` rejected at load with a helpful message ("…grant specific variables with 'env:VAR_NAME'"); empty `env:` rejected; `env` removed from `KNOWN_CAPABILITIES`.
- Tests (`wasm_executor.rs`): `env_capability_granted_via_allowlist` (now scoped), `bare_env_capability_rejected`, `scoped_env_does_not_leak_other_vars`. Polyglot test manifest → `env:FIDIUS_TEST_CAP`.
- Docs: `wasm-capabilities.md` (`env:VAR_NAME` row + "`env` is per-variable" section + credential-injection note updated).
- **Verified**: wasm_executor 23/23; lint clean. Committed `947d391`.