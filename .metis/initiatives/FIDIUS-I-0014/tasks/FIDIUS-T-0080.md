---
id: arena-macro-codegen-vtable-shim
level: task
title: "Arena macro codegen: vtable + shim variants; remove CallerAllocated"
short_code: "FIDIUS-T-0080"
created_at: 2026-04-18T01:15:15.612713+00:00
updated_at: 2026-04-18T01:27:00.597025+00:00
parent: FIDIUS-I-0014
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0014
---

# Arena macro codegen: vtable + shim variants; remove CallerAllocated

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0014]]

## Objective

Ship the macro/plugin-side half of I-0014: remove `CallerAllocated` from the ABI surface, implement the Arena buffer strategy end-to-end in codegen (interface vtable signature + plugin-side shim body), and verify via direct vtable invocation (no host). T-0081 follows with host-side `ArenaPool` + `call_method` dispatch.

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

- [x] `CallerAllocated` variant removed from `fidius_core::descriptor::BufferStrategyKind` (discriminant `0` reserved); `buffer_strategy_kind()` no longer maps `0`
- [x] `CallerAllocated` removed from `fidius_macro::ir::BufferStrategyAttr`; parsing emits a clear error when users try `buffer = CallerAllocated`
- [x] `BufferStrategyAttr` has `#[repr(u8)]` with explicit discriminants (`PluginAllocated = 1`, `Arena = 2`) matching `BufferStrategyKind`
- [x] `generate_interface` no longer rejects `Arena`; `generate_vtable` emits the Arena-variant fn signature `(in_ptr, in_len, arena_ptr, arena_cap, out_offset, out_len) -> i32` when `buffer = Arena`
- [x] `generate_shims` in `impl_macro.rs` branches on `buffer_strategy`: PluginAllocated (existing path) vs Arena (writes into host buffer, returns `STATUS_BUFFER_TOO_SMALL` if too small, panic path returns `out_len = 0`)
- [x] `plugin_impl` macro accepts `buffer = Arena` attribute to tell the macro which shim variant to emit (defaults to `PluginAllocated`)
- [x] Arena descriptors emit `free_buffer: None` (nothing to free — output lives in host arena)
- [x] Obsolete `compile_fail/unsupported_buffer.rs` removed
- [x] New `compile_fail/caller_allocated_removed.rs` verifies the removal error is clear
- [x] New `arena_basic.rs` test: Arena interface compiles, shim round-trips through direct vtable invocation, `STATUS_BUFFER_TOO_SMALL` path is exercised
- [x] Existing tests updated: `multi_plugin.rs`, `smoke_cdylib.rs` assertions of `abi_version == 2` → `100` (caught by these tests rebuilding against the new plugin)
- [x] Full `angreal test` passes (modulo pre-existing parallel e2e signing flake); `angreal lint` clean

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

- **2026-04-18**: Completed.

  **ABI surface changes (fidius-core):**
  - Removed `CallerAllocated` variant from `BufferStrategyKind`. Discriminant `0` reserved with doc note. `buffer_strategy_kind()` no longer handles `0` — rejected via `UnknownBufferStrategy`. Display impl updated.
  - Layout test updated: `CallerAllocated` assertion removed.

  **Macro changes (fidius-macro):**
  - `BufferStrategyAttr` became `#[repr(u8)]` with explicit discriminants matching `BufferStrategyKind` (`PluginAllocated = 1`, `Arena = 2`). This fixes a bug where the macro's `as u8` cast produced `0/1` while the descriptor expected `1/2`.
  - `CallerAllocated` removed from `BufferStrategyAttr` parsing; users get a clear "removed in fidius 0.1.0; use PluginAllocated or Arena" error.
  - `generate_interface` no longer rejects Arena — both strategies dispatch to codegen.
  - `generate_vtable` emits strategy-specific fn signature: PluginAllocated is `(*const u8, u32, *mut *mut u8, *mut u32) -> i32`; Arena is `(*const u8, u32, *mut u8, u32, *mut u32, *mut u32) -> i32`.
  - `plugin_impl` macro grew a `buffer = PluginAllocated | Arena` attribute that the user must set to match the interface's declaration (default PluginAllocated). Mismatches produce a clear vtable fn-pointer type error at compile time.
  - `generate_shims` branches on strategy:
    - PluginAllocated: unchanged (allocate `Box<[u8]>`, hand to host)
    - Arena: serialize output, check vs `arena_cap`, return `STATUS_BUFFER_TOO_SMALL` with required size if too small; otherwise copy into arena and set `out_offset=0`, `out_len=bytes.len()`. Panic path returns `STATUS_PANIC` with `out_len=0` (panic message not transmitted — arena might be too small; this is documented behavior).
  - `generate_descriptor` emits `free_buffer: None` for Arena (nothing to free). The `__fidius_free_buffer_*` fn is not emitted for Arena.

  **Tests:**
  - Removed obsolete `compile_fail/unsupported_buffer.rs` + `.stderr` — Arena is now supported.
  - New `compile_fail/caller_allocated_removed.rs` — verifies the removal error is surfaced clearly.
  - New `arena_basic.rs` with 2 tests:
    - `arena_shim_round_trip_with_sufficient_buffer` — serializes input, calls the echo shim with a 1 KB arena, decodes output from the arena at the reported offset+len, asserts result correctness.
    - `arena_shim_returns_buffer_too_small` — calls with a deliberately tiny 4-byte arena, asserts `STATUS_BUFFER_TOO_SMALL` (-1), asserts `out_len` exceeds the arena capacity (so the host knows how much to grow).
  - Updated `multi_plugin.rs:64` and `smoke_cdylib.rs:75` — `assert_eq!(desc.abi_version, 100)` (was 2 pre-0.1.0).

  **Validation:**
  - `cargo test -p fidius-macro --test arena_basic` — 2/2 pass.
  - `cargo test -p fidius-macro --test trybuild` — 5/5 compile_fail tests pass.
  - Full `angreal test` passes (modulo pre-existing parallel e2e signing flake).
  - `angreal lint` clean.

  **Unblocks T-0081** — host-side `ArenaPool`, `PluginHandle::call_method` dispatch, retry-on-too-small, integration test, docs.