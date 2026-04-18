---
id: client-in-process-plugin-name
level: task
title: "Client::in_process(plugin_name) — library-less PluginHandle + typed in-process Client"
short_code: "FIDIUS-T-0075"
created_at: 2026-04-17T19:51:09.400159+00:00
updated_at: 2026-04-17T19:54:01.251482+00:00
parent: FIDIUS-I-0017
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0017
---

# Client::in_process(plugin_name) — library-less PluginHandle + typed in-process Client

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0017]]

## Objective

Ship Option B from the T-0072 deferral discussion: unified typed Client API for in-process testing. Plugin authors write `CalculatorClient::in_process("BasicCalculator")?.add(&input)` to invoke a method on a plugin that's linked into the current test binary as an rlib — no dylib compilation, no subprocess, no raw unsafe.

Three pieces:
1. Make `PluginHandle._library` optional so handles can exist without an `Arc<libloading::Library>`.
2. Add `PluginHandle::from_descriptor(&'static PluginDescriptor)` and `PluginHandle::find_in_process_descriptor(name)` on `fidius-host`.
3. Generate `Client::in_process(plugin_name)` on every typed Client via the `plugin_interface` macro, under the existing `#[cfg(feature = "host")]` gate.

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

- [x] `PluginHandle._library` is `Option<Arc<Library>>` (was `Arc<Library>`); `None` indicates in-process
- [x] `PluginHandle::from_descriptor(desc: &'static PluginDescriptor) -> Result<Self, LoadError>` builds an in-process handle from a registered descriptor
- [x] `PluginHandle::find_in_process_descriptor(name: &str) -> Result<&'static PluginDescriptor, LoadError>` walks the inventory registry and returns the matching descriptor or `LoadError::PluginNotFound`
- [x] Generated Client includes `pub fn in_process(plugin_name: &str) -> Result<Self, LoadError>` under `#[cfg(feature = "host")]`
- [x] The `in_process` method validates `interface_hash` matches and returns `LoadError::InterfaceHashMismatch` on mismatch
- [x] `fidius-test/tests/smoke.rs` exercises `CalculatorClient::in_process("BasicCalculator")?.add(&input)` and asserts the result matches
- [x] `fidius-test/tests/smoke.rs` exercises the not-found path and asserts `LoadError::PluginNotFound`
- [x] No regressions: all other tests pass; `angreal lint` clean

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

  **`fidius-host/src/handle.rs`:**
  - `_library` field type: `Arc<Library>` → `Option<Arc<Library>>`. Documented that `None` indicates in-process use.
  - `PluginHandle::new` and `PluginHandle::from_loaded` wrap in `Some(...)` — no caller changes needed.
  - New `PluginHandle::from_descriptor(&'static PluginDescriptor) -> Result<Self, LoadError>` — copies descriptor metadata into owned `PluginInfo`, sets `_library: None`.
  - New `PluginHandle::find_in_process_descriptor(&str) -> Result<&'static PluginDescriptor, LoadError>` — walks `fidius_core::registry::get_registry()` and matches by `plugin_name_str()`. Returns `LoadError::PluginNotFound` on miss.

  **`fidius-macro/src/interface.rs::generate_client`:**
  - Captures `companion_mod = __fidius_<TraitName>` and `hash_name = <TraitName>_INTERFACE_HASH` idents.
  - Emits a new `in_process(plugin_name: &str) -> Result<Self, #crate_path::LoadError>` method that:
    1. Calls `PluginHandle::find_in_process_descriptor(plugin_name)` — `?` propagates `PluginNotFound`
    2. Compares `desc.interface_hash` against the trait's compile-time `INTERFACE_HASH` const. Returns `LoadError::InterfaceHashMismatch` on divergence.
    3. Builds handle via `from_descriptor` — `?` propagates `UnknownWireFormat` / `UnknownBufferStrategy` (theoretically possible for malformed in-process descriptors, practically never).
    4. Wraps in `Self::from_handle(handle)`.

  **`fidius-test/tests/smoke.rs`:**
  - Imports `CalculatorClient` and `AddInput` from `test_plugin_smoke` (rlib dev-dep).
  - `client_in_process_calls_plugin_without_dylib_load`: builds client via `in_process("BasicCalculator")`, calls `.add(&AddInput { a: 3, b: 7 })`, asserts result == 10.
  - `client_in_process_returns_not_found_for_missing_plugin`: asserts `LoadError::PluginNotFound` on unknown name.

  **Design note on the unified API:**
  The same `CalculatorClient` type now works for both workflows with zero API surface difference beyond the constructor:
  - Dylib: `CalculatorClient::from_handle(PluginHandle::from_loaded(host.load(name)?))`
  - In-process: `CalculatorClient::in_process(name)?`
  After construction, both expose identical method signatures. Plugin authors can write quick unit tests with `in_process` for fast iteration, then verify with the full dylib path in integration tests — zero mental context-switch.

  **Validation:**
  - `cargo test -p fidius-test` — 7/7 tests pass (5 from T-0072 + 2 new).
  - Full `angreal test` — 28 test-result groups, all pass. No flaky e2e this run.
  - `angreal lint` — clean.