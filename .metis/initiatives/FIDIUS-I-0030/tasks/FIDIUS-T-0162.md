---
id: cs2-2-cdylib-pull-callback-abi
level: task
title: "CS2.2 — cdylib pull-callback ABI + Iterator wrapper + host producer + E2E"
short_code: "FIDIUS-T-0162"
created_at: 2026-06-20T16:44:13.823701+00:00
updated_at: 2026-06-20T18:51:05.774875+00:00
parent: FIDIUS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0030
---

# CS2.2 — cdylib pull-callback ABI + Iterator wrapper + host producer + E2E

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0030]]

## Objective **[REQUIRED]**

cdylib client-streaming: a host-provided **pull callback** the plugin invokes in a loop — `next(state) -> (status, item_bytes)` (+ a drop/cancel) — wrapped by the macro into an `Iterator<Item = T>` the user's method consumes. The host owns the producer + the callback. The inverse of the FIDIUS-I-0026 server-streaming iterator-handle ABI ([[FIDIUS-A-0007]]). E2E: a writer/sink plugin consumes a host-produced stream of records. Depends on CS2.1 ([[FIDIUS-T-0161]]).

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

**Design — reuse `FidiusStreamHandle`.** Shipped server-streaming
(`fidius-guest/src/stream_ffi.rs`): `FidiusStreamHandle { next(handle, buf, cap,
&out_len) -> status, drop_fn, state }` + `StreamState<T>` (guest PRODUCER).
Client-streaming = the **inverse with the same handle struct** (host produces, guest
consumes):
1. Guest consumer `HostStream<T>` (stream_ffi): wraps `*mut FidiusStreamHandle`,
   `impl Iterator` via `handle.next` + bincode-deserialize; `Drop` runs `drop_fn`.
2. Host producer (fidius-host): builds a handle from a Rust iterator; its `next`
   serializes each item (mirror of StreamState).
3. Vtable shape `ClientStreamFn = (instance, *mut FidiusStreamHandle, *const u8 args,
   u32 len, *mut *mut u8 out, *mut u32 out_len) -> i32`; macro emits it; descriptor
   marks the method; host calls with the producer handle.
4. Macro: for `fn m(&self, s: Stream<T>, ...) -> O`, build `HostStream<T>` → `Stream<T>`
   from the handle, decode other args, call the user method, bincode O out.
5. Host raw path here; typed `call_client_streaming` is CS2.5.
6. E2E: `fn load(&self, rows: Stream<u64>) -> u64` (sum), host produces [1,2,3] → 6.

Step order: (a) HostStream consumer + host producer + round-trip unit test;
(b) ClientStreamFn shape + descriptor + macro shim; (c) host call path; (d) E2E.

**DONE (commits c1ea7fa, 6778155, 0b8df43, 0ae8e88).** All steps landed:
(a) `HostStream<T>` consumer + `host_producer_handle` + round-trip unit test (host
produces → guest consumes). (b) `generate_vtable` emits a per-method `ClientStreamFn`
slot; the macro cdylib shim builds `Stream::from_iter(HostStream::from_handle(handle))`,
decodes non-stream args, calls the user method, bincodes the result; Client skips
client-streaming methods. (c) `call_client_streaming_raw` on CdylibExecutor + PluginHandle
(unsafe — takes the producer handle); WASM/Python return clear "not yet wired"; WASM
adapter + Arena reject it. (d) E2E `cdylib_client_stream_e2e`: `fn load(&self, rows:
Stream<u64>) -> u64` sums host-produced [1..=5] → 15. Removed the obsolete
`stream_in_arg_position` compile-fail (now compiles). Default 66 + server-streaming +
macro_wasm + clippy + lint green. **The cdylib client-streaming pull ABI is proven end
to end.** Remaining in the arc: CS2.3 (WASM), CS2.4 (Python), CS2.5 (typed host API + docs).