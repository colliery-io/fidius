---
id: host-accessors-for-metadata-fidius
level: task
title: "Host accessors for metadata + fidius inspect output + docs"
short_code: "FIDIUS-T-0079"
created_at: 2026-04-18T01:03:26.706931+00:00
updated_at: 2026-04-18T01:13:41.304699+00:00
parent: FIDIUS-I-0018
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0018
---

# Host accessors for metadata + fidius inspect output + docs

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0018]]

## Objective

Expose the metadata fields emitted by T-0078 to host-side consumers:
- `PluginHandle::method_metadata(index) -> Vec<(&str, &str)>`
- `PluginHandle::trait_metadata() -> Vec<(&str, &str)>`
- `fidius inspect` CLI output that surfaces both trait and per-method metadata
- Integration test that round-trips metadata through the full dylib load path
- `docs/how-to/method-metadata.md` tutorial + spec + mkdocs nav entry

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

- [x] `LoadedPlugin` carries `descriptor: *const PluginDescriptor` (populated in `validate_descriptor`)
- [x] `PluginHandle` stores the descriptor pointer; populated by `from_loaded` and `from_descriptor`
- [x] `PluginHandle::method_metadata(method_id: u32) -> Vec<(&str, &str)>` reads via descriptor, returns empty for out-of-range index, null method_metadata, or methods with no annotations
- [x] `PluginHandle::trait_metadata() -> Vec<(&str, &str)>` reads trait_metadata array via descriptor
- [x] `fidius inspect` CLI output includes trait metadata and per-method metadata sections when present
- [x] test-plugin-smoke declares trait_meta + method_meta annotations so integration tests and `fidius inspect` exercise the full flow
- [x] New integration test `trait_and_method_metadata_readable_through_handle` in `fidius-host/tests/integration.rs` asserts values round-trip through dylib load
- [x] `docs/how-to/method-metadata.md` written — covers attribute syntax, host-side reading, validation rules, what fidius doesn't define, when not to use metadata
- [x] `docs/reference/abi-specification.md` descriptor layout table updated with new fields at offsets 80/88/96; total size now 104 bytes
- [x] `mkdocs.yml` nav updated with the new how-to entry
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

- **2026-04-18**: Completed.

  **Host plumbing (`fidius-host/src/loader.rs`, `handle.rs`):**
  - Extended `LoadedPlugin` with `descriptor: *const PluginDescriptor`. Populated in `validate_descriptor` from the descriptor reference the loader already has.
  - Extended `PluginHandle` with the same field. Populated in both `from_loaded` and `from_descriptor`.
  - Added `PluginHandle::method_metadata(method_id: u32) -> Vec<(&str, &str)>`:
    - Returns empty for out-of-range `method_id`, null descriptor's `method_metadata`, or methods with no annotations (null `kvs` in the entry)
    - Converts each `MetaKv` via `CStr::from_ptr().to_str()` — panics if metadata strings aren't valid UTF-8 (an ABI contract violation — the macro only emits literals)
  - Added `PluginHandle::trait_metadata() -> Vec<(&str, &str)>` with the same conversion pattern over the descriptor's trait_metadata array.
  - Returned `&str` borrows bound by `&self` — safe because the handle holds `Arc<Library>` (dylib) or the pointer targets binary `.rodata` (in-process). Pattern matches the existing `info()` accessor.

  **Design choice — no descriptor_size gating for these fields:**
  For the 0.1.0 ABI batch, all plugins at this version carry the metadata fields. Strict-equal match on `abi_version == 100` catches any pre-0.1.0 plugin before accessor methods are reached. Gating via `descriptor_size` is for future post-1.0 minor releases that add NEW fields beyond these. Not needed here, and avoiding the overhead keeps accessors simple.

  **CLI (`fidius-cli/src/commands.rs::inspect`):**
  - Upgraded `inspect` to wrap each loaded plugin in a `PluginHandle` (via the existing `from_loaded`).
  - Prints trait metadata as indented `key = value` pairs when present.
  - Prints per-method metadata when any method has annotations, showing `[index]:` headers.
  - Output is empty for plugins with no metadata — no noise for existing plugins.

  **Test fixture (`tests/test-plugin-smoke/src/lib.rs`):**
  - Added `#[trait_meta("kind", "calculator")]` + `#[trait_meta("stability", "stable")]` to the Calculator trait.
  - Added `#[method_meta("effect", "pure")]` to add, add_direct, and multiply. Left `version` without metadata to exercise the empty-entry path.

  **Integration test (`fidius-host/tests/integration.rs`):**
  - `trait_and_method_metadata_readable_through_handle` — loads test-plugin-smoke via the full dylib path, asserts trait metadata is `[("kind","calculator"), ("stability","stable")]`, asserts method 0/1/3 each have `[("effect","pure")]` and method 2 has empty metadata. Also confirms out-of-range indices return empty vec (no panic).
  - All 11 integration tests pass (10 existing + 1 new).

  **Docs:**
  - `docs/how-to/method-metadata.md` (new) — covers attribute syntax, both reading paths (`handle.*_metadata()` API and `fidius inspect` CLI), validation rules, what fidius doesn't define (opaque strings), suggested conventions, and when NOT to use metadata.
  - `docs/reference/abi-specification.md` — descriptor layout table updated with `method_metadata` (offset 80), `trait_metadata` (88), `trait_metadata_count` (96), trailing padding (100), total 104 bytes. Links back to how-to doc.
  - `mkdocs.yml` — added "Method Metadata" under How-To Guides.

  **Validation:**
  - Full `angreal test` — all suites pass (27+ test-result groups).
  - `angreal lint` clean.

  **I-0018 complete. 0.1.0 ABI batch now has 2 of 3 landed (drop JSON, metadata). Arena (I-0014) is the remaining ABI initiative.**