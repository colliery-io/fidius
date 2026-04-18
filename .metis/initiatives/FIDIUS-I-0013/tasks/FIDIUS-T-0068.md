---
id: migrate-ffi-output-from-vec-shrink
level: task
title: "Migrate FFI output from Vec+shrink_to_fit to Box&lt;[u8]&gt; in shim and free_buffer codegen"
short_code: "FIDIUS-T-0068"
created_at: 2026-04-17T17:40:53.477508+00:00
updated_at: 2026-04-17T17:44:41.178422+00:00
parent: FIDIUS-I-0013
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0013
---

# Migrate FFI output from Vec+shrink_to_fit to Box&lt;[u8]&gt; in shim and free_buffer codegen

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0013]]

## Objective

Replace the fragile `Vec<u8> + shrink_to_fit + mem::forget` + `Vec::from_raw_parts(ptr, len, len)` FFI output pattern with `Box<[u8]>` + `Box::into_raw` + `Box::from_raw(slice::from_raw_parts_mut(ptr, len))`. `Box<[u8]>` has `cap == len` by construction, eliminating the implicit invariant that one missing `shrink_to_fit` would make UB. See FIDIUS-I-0013 for full context.

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

- [x] Shim success-path uses `Box::into_raw(output_bytes.into_boxed_slice())` — no `shrink_to_fit`, no `mem::forget`
- [x] Shim panic-path uses the same pattern for the panic-message output
- [x] Generated `__fidius_free_buffer_*` uses `Box::from_raw(slice::from_raw_parts_mut(ptr, len) as *mut [u8])` — no `Vec::from_raw_parts`
- [x] `rg 'shrink_to_fit' fidius-macro/src/` returns no matches
- [x] `rg 'Vec::from_raw_parts' fidius-macro/src/ fidius-host/src/` returns no matches
- [x] `angreal test` — all suites pass (layout, smoke, integration, e2e, multi-arg, multi-plugin, async, crate_path, trybuild, CLI full pipeline)
- [x] `angreal lint` clean

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

## Status Updates

- **2026-04-17**: Completed. Files changed:
  - `fidius-macro/src/impl_macro.rs:148-159` — `free_buffer` codegen now uses `Box::from_raw(slice::from_raw_parts_mut(ptr, len) as *mut [u8])`; explanatory comment notes the cap==len guarantee
  - `fidius-macro/src/impl_macro.rs:245-258` — shim success path uses `into_boxed_slice()` + `Box::into_raw`; removed `shrink_to_fit` and `mem::forget`
  - `fidius-macro/src/impl_macro.rs:270-278` — shim panic path uses the same pattern for the panic-message output buffer

  Verified: `rg 'shrink_to_fit' fidius-macro/src/` and `rg 'Vec::from_raw_parts' fidius-macro/src/ fidius-host/src/` both return no matches.

  No ABI change: the FFI surface (`fn(*mut u8, usize)` for free_buffer, `(out_ptr, out_len)` for shim output) is identical — this is a pure codegen internal cleanup. Plugins built with the new codegen are byte-level ABI-compatible with old hosts and vice versa.

  `angreal test` — all 23 test-result groups pass (layout, wire, package, signing, arch, hash, status, e2e, integration, full_pipeline, smoke_cdylib, multi_arg, multi_plugin, async_plugin, crate_path, impl_basic, interface_basic, trybuild, doc-tests).

  `angreal lint` clean.