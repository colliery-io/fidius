---
id: cs-1-cdylib-server-streaming-via
level: task
title: "CS.1 — cdylib server-streaming via iterator-handle ABI (Phase 3)"
short_code: "FIDIUS-T-0136"
created_at: 2026-06-19T17:06:57.606006+00:00
updated_at: 2026-06-19T17:24:39.202992+00:00
parent: FIDIUS-I-0026
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0026
---

# CS.1 — cdylib server-streaming via iterator-handle ABI (Phase 3)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0026]]

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

## Implementation plan (design locked)

- **Shared `#[repr(C)] FidiusStreamHandle`** in `fidius-guest`: `{ next: fn(*mut Self, *mut *mut u8, *mut u32)->i32, drop_fn: fn(*mut Self), state: *mut c_void }`. Re-exported via `fidius_core`.
- **Macro** (`generate_shims`): streaming method's vtable slot = an **init** shim with the *existing* `FfiFn` signature (so **no vtable/descriptor/ABI-version change**) that boxes a handle and writes its ptr into `*out_ptr`. Plus generated `next` (pull `next_item()` → JSON(`to_value`) into a plugin `Box<[u8]>`; `None` → `out_ptr=null`=end) and `drop_fn` (drop stream + free handle). Remove the `any_streaming`→`compile_error!` skip.
- **Host** `CdylibExecutor: StreamExecutor`: init → handle; pump thread calls `next()` per item, JSON→`Value`, `free_buffer` each, bounded `tokio::mpsc` → `ChunkStream`; drop → `drop_fn`. Mirrors the ST.3 Python bridge.
- **`PluginHandle::call_streaming`** Cdylib arm dispatches (was the Phase-3 error).
- **Item encoding** = JSON(`Value`) (host needs no compile-time `T`; bincode can't reconstruct a `Value`). Typed-bincode fast path = future optimization.
- **Fixture/E2E**: cdylib `TickerImpl` streaming impl + load + `call_streaming` + bounded/cancel. **Bench**: add `cdylib` to `stream_drain`. Remove/replace the `native_streaming_impl` trybuild compile-fail (cdylib streaming now supported).

## Status Updates **[REQUIRED]**

### 2026-06-19 — CS.1 complete ✅ — cdylib is now a streaming peer
Implemented the iterator-handle ABI exactly as planned; it worked.
- **`fidius_guest::stream_ffi`**: `#[repr(C)] FidiusStreamHandle { next, drop_fn, state }` + `stream_item_encode`/`stream_item_decode` (JSON(`Value`), self-describing). Re-exported via `fidius_core` + `fidius`.
- **Macro (`generate_shims`)**: a streaming method emits 3 fns — `init` (in the vtable slot, ordinary `FfiFn` shape, boxes the handle into `*out_ptr` → **no vtable/descriptor/ABI bump**), `next` (pull `Stream::next_item()` → JSON item into a plugin `Box<[u8]>`; `None` → null out_ptr = end), `drop_fn` (drop producer + free handle). Each carries its own `#[cfg(not(target_family="wasm"))]`. Removed the `any_streaming`→`compile_error!`; Arena+streaming rejected at macro time (PluginAllocated required).
- **Host `CdylibExecutor::call_streaming_raw`** (gated `streaming`): init → handle; dedicated pump thread (`SendHandle` wrapper; whole-struct capture to dodge 2021 disjoint-capture of the raw ptr) calls `next()`, JSON→`Value`, `free_buffer`s each item, bounded `tokio::mpsc` → `ChunkStream`; drop → `drop_fn`. `PluginHandle::call_streaming` Cdylib arm bincode-serialises args (no `Value` hop) and dispatches.
- **Fixture/E2E**: `TickerImpl` in test-plugin-smoke; `crates/fidius-host/tests/cdylib_streaming_e2e.rs` — **3 tests green** (`[0..5]`, empty, 10M bounded/cancellable). Removed the obsolete `native_streaming_impl` trybuild compile-fail.
- **Bench**: `cdylib` added to the `stream_drain` comparison.
- **Verified**: macro suite green; `angreal test` (default) 50 ok-blocks, 0 fail; cdylib/wasm/python streaming suites green.

**4-way perf comparison (converged per-item @ N=10k):**
| Backend | per-item | thrpt |
|---|---|---|
| wasm / Rust | **1.37 µs** | ~728 K/s |
| **cdylib** (JSON items) | 1.64 µs | ~609 K/s |
| python | 1.78 µs | ~550 K/s |
| wasm / JS | ~134 µs | ~7.5 K/s |

**Notable finding:** cdylib is **not** the fastest despite being in-process native — because CS.1's items cross as **JSON(`Value`)** (so the host needs no compile-time `T`), and JSON encoding costs more per item than wasm's native component-`Val` lifting. This is the documented tradeoff; the **typed-bincode fast path** that would make cdylib the fastest is filed as a follow-up: [[FIDIUS-T-0137]]. v1 keeps JSON — simple, correct, ~1.6 µs/item (still native-class).