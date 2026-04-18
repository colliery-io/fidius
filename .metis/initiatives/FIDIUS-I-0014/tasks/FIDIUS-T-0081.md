---
id: arena-host-side-arenapool-call
level: task
title: "Arena host-side: ArenaPool + call_method dispatch + integration + docs"
short_code: "FIDIUS-T-0081"
created_at: 2026-04-18T01:15:16.766594+00:00
updated_at: 2026-04-18T01:32:58.610130+00:00
parent: FIDIUS-I-0014
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0014
---

# Arena host-side: ArenaPool + call_method dispatch + integration + docs

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0014]]

## Objective

Complete I-0014's Arena support: thread-local arena pool, `PluginHandle::call_method` dispatch on buffer strategy, retry-on-too-small, integration test using the Arena-strategy plugin from test-plugin-smoke, and docs.

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

- [x] New `fidius-host/src/arena.rs` module with thread-local `ARENA_POOL` and `acquire_arena`, `release_arena`, `grow_arena` helpers
- [x] `PluginHandle::call_method` dispatches on `info.buffer_strategy`: PluginAllocated (existing path factored into `call_plugin_allocated`) or Arena (`call_arena`)
- [x] Arena path acquires buffer → invokes vtable fn → retries once on `STATUS_BUFFER_TOO_SMALL` (plugin writes required size into `out_len`) → deserializes from `arena[offset..offset+len]` → releases buffer
- [x] Status code handling for Arena: `STATUS_OK`, `STATUS_BUFFER_TOO_SMALL` (second failure = `CallError::BufferTooSmall`), `STATUS_PLUGIN_ERROR`, `STATUS_SERIALIZATION_ERROR`, `STATUS_PANIC` (returns opaque panic message — documented behavior)
- [x] test-plugin-smoke grows a second plugin `ArenaEcho` + `ArenaEchoer` (same dylib, Arena strategy)
- [x] Integration test: `arena_plugin_loads_and_round_trips` via typed Client
- [x] Integration test: `arena_plugin_grows_buffer_on_too_small_retry` with a 10 KB input exceeding the 4 KB default arena
- [x] `smoke_cdylib.rs` updated to find BasicCalculator in a multi-plugin registry (plugin_count is now 2)
- [x] `docs/explanation/buffer-strategies.md` rewritten for the two-strategy world; PluginAllocated default, Arena for hot paths
- [x] `docs/reference/abi-specification.md` updated: BufferStrategyKind table reflects discriminant 0 as reserved, Arena is fully supported, STATUS_BUFFER_TOO_SMALL description mentions Arena retry
- [x] Full `angreal test` passes; `angreal lint` clean

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

- **2026-04-18**: Completed. I-0014 closed — 0.1.0 ABI batch fully shipped.

  **`fidius-host/src/arena.rs` (new):**
  - Thread-local `RefCell<Vec<Vec<u8>>>` arena pool.
  - `acquire_arena(min_cap)` — pops a pooled buffer (growing it if needed) or allocates a fresh one at `max(min_cap, 4096)`.
  - `release_arena(buf)` — pushes back to the pool.
  - `grow_arena(&mut buf, needed)` — in-place grow used by the retry path.
  - `DEFAULT_ARENA_CAPACITY = 4096` constant.

  **`fidius-host/src/handle.rs`:**
  - `PluginHandle::call_method` now dispatches on `self.info.buffer_strategy`.
  - Factored existing PluginAllocated body into `call_plugin_allocated` — behavior unchanged.
  - New `call_arena` path:
    1. Acquire arena via `acquire_arena(DEFAULT_ARENA_CAPACITY)`.
    2. Call vtable fn with `ArenaFn` signature.
    3. On `STATUS_BUFFER_TOO_SMALL`: grow arena to `out_len` (plugin wrote needed size) and retry exactly once. Second too-small returns `CallError::BufferTooSmall`.
    4. On `STATUS_OK`: deserialize from `arena[offset..offset+len]`, release buffer.
    5. `STATUS_PLUGIN_ERROR` reads the error from the arena slice and returns `CallError::Plugin`.
    6. `STATUS_PANIC` returns `CallError::Panic` with a canned message (panic text not transmitted — documented).
  - Released arenas go back to the pool for reuse on the same thread.

  **Test fixture:**
  - `tests/test-plugin-smoke/src/lib.rs` added `ArenaEcho` trait with `buffer = Arena` and `ArenaEchoer` impl. Two plugins in one dylib — exercises the multi-plugin-per-dylib path too.

  **Integration tests (`fidius-host/tests/integration.rs`):**
  - `arena_plugin_loads_and_round_trips` — loads via the typed `ArenaEchoClient`, calls `echo(&"hello".to_string())`, asserts `"arena-echo: hello"`. Verifies `loaded.free_buffer.is_none()` (Arena has no free_buffer).
  - `arena_plugin_grows_buffer_on_too_small_retry` — feeds a 10 KB input, which produces an output exceeding the 4 KB default arena. Host grows and retries automatically, returns the full output. Asserts length + prefix match.

  **Other tests updated:**
  - `fidius-macro/tests/smoke_cdylib.rs` — walks the registry to find `BasicCalculator` by name instead of indexing [0] (plugin_count is now 2 because of ArenaEchoer).

  **Docs:**
  - `docs/explanation/buffer-strategies.md` fully rewritten: PluginAllocated default, Arena for hot paths, comparison table, when-to-pick guidance. Removed all references to CallerAllocated.
  - `docs/reference/abi-specification.md`: BufferStrategyKind table updated — discriminant 0 reserved (formerly CallerAllocated); both PluginAllocated and Arena supported. STATUS_BUFFER_TOO_SMALL description mentions Arena retry flow.

  **Validation:**
  - `cargo test -p fidius-host --test integration` — 13/13 pass including both Arena tests.
  - `cargo test -p fidius-macro --test arena_basic` — 2/2 pass (direct vtable invocation + BUFFER_TOO_SMALL path).
  - `cargo test -p fidius-macro --test smoke_cdylib` — 1/1 pass.
  - Full `angreal test` — all suites pass (modulo pre-existing parallel e2e signing flake unrelated to this work).
  - `angreal lint` clean.

  **I-0014 complete. 0.1.0 ABI batch done** (I-0013 + I-0015 + I-0016 + I-0018 + I-0019 + I-0014).