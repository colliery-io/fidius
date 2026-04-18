---
id: metadata-types-descriptor-fields
level: task
title: "Metadata types, descriptor fields, macro attr parsing and codegen"
short_code: "FIDIUS-T-0078"
created_at: 2026-04-18T01:03:25.702688+00:00
updated_at: 2026-04-18T01:09:11.740468+00:00
parent: FIDIUS-I-0018
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0018
---

# Metadata types, descriptor fields, macro attr parsing and codegen

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0018]]

## Objective

Ship the declaration-side surface for method and trait metadata per FIDIUS-I-0018:

- `MetaKv` and `MethodMetaEntry` types in `fidius-core`
- 3 new additive fields on `PluginDescriptor` (method_metadata, trait_metadata, trait_metadata_count)
- `#[fidius::method_meta("k", "v")]` and `#[fidius::trait_meta("k", "v")]` inert attributes parsed by `plugin_interface`
- Validation: string literals, non-empty keys, no duplicate keys, reserved `fidius.*` namespace, no leading/trailing whitespace in keys
- Codegen: emit static `MetaKv` arrays and `MethodMetaEntry` table; wire pointers into the descriptor builder
- Tests (round-trip via registry) + compile-fail cases

Host-side accessors, CLI inspect output, and user-facing docs are FIDIUS-T-0079.

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

- [x] `MetaKv`, `MethodMetaEntry` types added to `fidius-core/src/descriptor.rs` with `Send + Sync` impls
- [x] `PluginDescriptor` gains `method_metadata: *const MethodMetaEntry`, `trait_metadata: *const MetaKv`, `trait_metadata_count: u32` fields at the end (additive, 80 → 104 byte descriptor)
- [x] Layout tests updated (`descriptor_field_offsets`, `descriptor_size_and_align`)
- [x] `#[method_meta("k", "v")]` and `#[trait_meta("k", "v")]` attribute parsing in `fidius-macro/src/ir.rs` with validation (literals only, non-empty keys, no duplicates, no `fidius.*` reserved, no whitespace)
- [x] `strip_optional_attrs` extended to strip `#[method_meta]` + `#[trait_meta]` from emitted trait
- [x] `generate_metadata` emits static `__FIDIUS_METHOD_META_<UPPER>` arrays + `__FIDIUS_METHOD_META_TABLE` + `__FIDIUS_TRAIT_META` into the companion module (only when non-empty)
- [x] `generate_descriptor_builder` wires pointers into the new descriptor fields; null when no metadata declared
- [x] `fidius-macro/tests/metadata.rs` round-trip test: trait with trait_meta + per-method method_meta; assert fields populated correctly via registry + CStr reads
- [x] `interface_hash_unaffected_by_metadata` test confirms metadata does not participate in the hash
- [x] Compile-fail tests: `duplicate_method_meta_key.rs`, `reserved_fidius_namespace.rs`
- [x] Existing macro tests unchanged (no metadata declared → null pointers in descriptor; behavior unchanged)
- [x] `angreal test` passes; `angreal lint` clean

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

  **`fidius-core/src/descriptor.rs`:**
  - Added `MetaKv` struct (`*const c_char` key/value) with `Send + Sync` impls.
  - Added `MethodMetaEntry` struct (`*const MetaKv` + `u32` count) with `Send + Sync` impls.
  - Extended `PluginDescriptor` with `method_metadata`, `trait_metadata`, `trait_metadata_count` fields at the end.

  **`fidius-macro/src/ir.rs`:**
  - Added `MetaKvAttr { key: String, value: String }`.
  - Added `trait_metas: Vec<MetaKvAttr>` to `InterfaceIR` and `method_metas: Vec<MetaKvAttr>` to `MethodIR`.
  - New `parse_meta_attrs(attrs, ident)` function handles both `method_meta` and `trait_meta`. Validation rules:
    - Two comma-separated string literals required
    - Non-empty key
    - No leading/trailing whitespace in key
    - Reserved `fidius.*` namespace rejected
    - Duplicate keys rejected (per method, per trait)
  - `parse_interface` now walks `method.attrs` + `item.attrs` and populates the metas fields.

  **`fidius-macro/src/interface.rs`:**
  - `strip_optional_attrs` renamed conceptually (still named this) to cover three fidius-helper attrs: `optional`, `method_meta`, `trait_meta`. Strips them from both the trait and its methods before re-emission.
  - New `generate_metadata(ir)` function:
    - If any method has metadata, emits `__FIDIUS_METHOD_META_<UPPER>: [MetaKv; N]` per-method and `__FIDIUS_METHOD_META_TABLE: [MethodMetaEntry; method_count]` with empty entries (null kvs, zero count) for methods without annotations.
    - If trait has metadata, emits `__FIDIUS_TRAIT_META: [MetaKv; N]`.
    - String literals are null-terminated via `concat!(key, "\0").as_ptr() as *const c_char` — standard fidius pattern for static CStr-like pointers.
  - `generate_descriptor_builder` now emits the three new descriptor fields. When no metadata declared anywhere, fields default to `std::ptr::null()` and `0`.

  **Tests added:**
  - `fidius-macro/tests/metadata.rs` — 3 tests: trait_metadata populated, per-method metadata populated (including empty entry for methods without annotations), interface_hash unaffected by metadata annotations.
  - `fidius-macro/tests/compile_fail/duplicate_method_meta_key.rs` + `.stderr`
  - `fidius-macro/tests/compile_fail/reserved_fidius_namespace.rs` + `.stderr`

  **Layout test update:**
  - `descriptor_size_and_align` — descriptor now 104 bytes (was 80), still 8-byte aligned.
  - `descriptor_field_offsets` — new fields at offsets 80 / 88 / 96; trailing padding to 104.

  **Validation:**
  - `cargo test -p fidius-macro --test metadata` — 3/3 pass.
  - `cargo test -p fidius-macro --test trybuild` — 5/5 compile_fail tests pass (3 existing + 2 new).
  - Full `angreal test` — all suites pass.
  - `angreal lint` clean.

  **Notes:**
  - The pre-existing cfg(feature = "host") warning in non-host-feature test binaries remains (harmless — Client code is gated out). Not introduced by this task.
  - T-0079 (host accessors + inspect + docs) now unblocked.