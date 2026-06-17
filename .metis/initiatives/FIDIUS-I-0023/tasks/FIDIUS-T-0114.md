---
id: w-2-derive-wittype-marker-adapter
level: task
title: "W.2 — #[derive(WitType)] marker + adapter rework (path-based generate! + conversions + .into() boundary)"
short_code: "FIDIUS-T-0114"
created_at: 2026-06-17T13:01:00.557934+00:00
updated_at: 2026-06-17T13:30:00.262903+00:00
parent: FIDIUS-I-0023
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0023
---

# W.2 — #[derive(WitType)] marker + adapter rework (path-based generate! + conversions + .into() boundary)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0023]]

## Objective **[REQUIRED]**

Add `#[derive(WitType)]` and rework the `#[plugin_impl]` wasm adapter so a user-typed interface emits `generate!{path:"wit"}` + the generated conversions + a converting `Guest` boundary, while primitives-only interfaces keep the inline path (no regression).

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `#[derive(WitType)]` proc-macro (marker; emits nothing — the build-time generator reads the annotation from source).
- [x] `generate_wasm_adapter` is dual-path: primitives-only → inline WIT (unchanged, no `build.rs`); user types present → `generate!{ path: "wit" }` + `include!(OUT_DIR/fidius_wit_conversions.rs)` + Guest methods using `gen_type` (signatures) + `conv_expr` (boundary, reusing `fidius-wit`).
- [x] Reference args rejected (owned-only v1); structurally-unsupported types still emit the wasm-gated `compile_error!`.
- [x] Macro builds + tests pass; workspace green; macro-greeter (inline) unchanged. (User-type path compile-verified E2E in [[FIDIUS-T-0116]].)

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

**2026-06-17 — COMPLETE.** Commit `0fa3999`. `#[derive(WitType)]` marker + dual-path adapter (`gen_type`/`conv_expr` reused from `fidius-wit`). Macro tests pass; macro-greeter (primitives, inline) unchanged; workspace green. The user-type path's generated conversions are compile-verified end-to-end by T-0116 (records fixture + build.rs).