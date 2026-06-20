---
id: ci-1-cdylib-construct-destroy-abi
level: task
title: "CI.1 — cdylib construct/destroy ABI (singleton = construct-with-unit, ABI 400→500)"
short_code: "FIDIUS-T-0147"
created_at: 2026-06-20T01:44:06.033709+00:00
updated_at: 2026-06-20T01:44:39.373591+00:00
parent: FIDIUS-I-0029
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: FIDIUS-I-0029
---

# CI.1 — cdylib construct/destroy ABI (singleton = construct-with-unit, ABI 400→500)

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

**ABI fully mapped — this is an atomic change (no green intermediate). Execution map:**

Current ABI: cdylib method = uniform `unsafe extern "C" fn(in_ptr,in_len, out_ptr_ptr,out_len) -> i32`
(PluginAllocated) / `(...,arena_ptr,cap,out_off,out_len)` (Arena). Host casts
`vtable as *const FfiFn` and indexes by method number. Shims dispatch on a `static`
singleton. Pivot: **prepend `instance: *mut c_void`** to that fn type everywhere +
add construct/destroy.

Touch points (all move in lockstep):
1. `crates/fidius-guest/src/descriptor.rs` (struct `PluginDescriptor`, ~L177): append
   `construct: Option<unsafe extern "C" fn(*const u8, u32) -> *mut c_void>` +
   `destroy: Option<unsafe extern "C" fn(*mut c_void)>`. Size 104→120, 8-aligned.
2. `crates/fidius-macro/src/interface.rs::generate_vtable` (L202-216): both `fn_type`
   variants gain a leading `*mut ::core::ffi::c_void`. (Also the Client/host caller if
   it invokes directly.)
3. `crates/fidius-macro/src/impl_macro.rs`: shims (L800/891/944 + streaming init L800)
   gain `instance: *mut c_void` first param → `let __p = &*(instance as *const #impl_type);`
   → dispatch on `__p`. Generate `construct` (`Box::into_raw(Box::new(<unit ctor>)) as *mut c_void`;
   typed `config = C` deserialize is CI.2) + `destroy` (`drop(Box::from_raw(p as *mut #impl_type))`);
   wire both into the descriptor literal (~L1072). Keep the `static #instance_name`
   for the **wasm** adapter path (cfg-split); cdylib shims now use the param.
4. `crates/fidius-host/src/executor/cdylib.rs`: `FfiFn`/`ArenaFn` aliases + every call
   site (244/328/419/497/611 + streaming) gain the instance arg. Executor calls
   `construct` at load → store `*mut c_void` → pass to each call → `destroy` on Drop.
5. `crates/fidius-host/src/loader.rs` (L39-43): carry `construct`/`destroy` from desc.
6. ABI_VERSION 400→500 = bump workspace to **0.5.0** (version-derived); fix
   `layout_and_roundtrip.rs` (descriptor_size 104→120, +2 offsets, ABI 400→500) and the
   macro abi asserts (impl_basic/multi_plugin/smoke_cdylib 400→500).

CI.1 scope = singleton works via `construct(())` end to end; typed `config=C` ctor +
`ConfiguredHandle` is CI.2. Verify: existing cdylib suites green on the new ABI.