---
id: bump-to-0-1-0-derive-abi-version
level: task
title: "Bump to 0.1.0, derive ABI_VERSION, add descriptor_size first field"
short_code: "FIDIUS-T-0076"
created_at: 2026-04-18T00:51:09.797068+00:00
updated_at: 2026-04-18T00:54:35.247316+00:00
parent: FIDIUS-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0019
---

# Bump to 0.1.0, derive ABI_VERSION, add descriptor_size first field

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0019]]

## Objective

Apply ADR-0002 mechanically: bump all workspace crates from 0.0.5 to 0.1.0, replace the hand-maintained `ABI_VERSION` constant with a const-evaluated formula derived from `CARGO_PKG_VERSION_MAJOR/MINOR`, and add `descriptor_size: u32` as the first field of `PluginDescriptor`. Unlocks additive post-1.0 minor releases and clears the prep gate for the three ABI-bumping initiatives (I-0014 Arena, I-0016 drop JSON, I-0018 method metadata).

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

- [x] All workspace `Cargo.toml` files at version `0.1.0`; all cross-crate path deps reference `version = "0.1.0"`
- [x] `parse_u32_const` const helper added to `fidius-core/src/descriptor.rs`; parses `CARGO_PKG_VERSION_MAJOR/MINOR`
- [x] `ABI_VERSION` is derived: `if MAJOR == 0 { MAJOR * 10000 + MINOR * 100 } else { MAJOR * 10000 }`. At 0.1.0 it evaluates to `100`
- [x] `PluginDescriptor.descriptor_size: u32` added at offset 0; `abi_version` now at offset 4; no other field offsets change (the new u32 fits in what was previously 4-byte alignment padding)
- [x] `generate_descriptor_builder` in the macro populates `descriptor_size: std::mem::size_of::<PluginDescriptor>() as u32`
- [x] `fidius-core/tests/layout_and_roundtrip.rs` — offset assertions updated; `version_constants` asserts `ABI_VERSION == 100`
- [x] `fidius-macro/tests/impl_basic.rs::descriptor_fields_are_correct` asserts `desc.abi_version == 100`
- [x] `docs/reference/abi-specification.md` updated with new descriptor layout (descriptor_size field, method_count field, ADR-0002 references)
- [x] Full `angreal test` passes (modulo pre-existing parallel flake on e2e signing tests)
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

- **2026-04-18**: Completed.

  **Version bump:**
  - `sed` across all Cargo.toml files: `0.0.5` → `0.1.0`. Affects 6 package versions and 9 cross-crate path-dep version attributes.

  **`fidius-core/src/descriptor.rs`:**
  - Added `parse_u32_const` — a `const fn` that parses a decimal ASCII string to `u32` by hand (const-eval can't use `.parse()` yet).
  - Added `CRATE_MAJOR`, `CRATE_MINOR` module-level consts derived from `env!("CARGO_PKG_VERSION_MAJOR/MINOR")`.
  - Replaced `pub const ABI_VERSION: u32 = 2;` with the conditional formula from ADR-0002.
  - Added `descriptor_size: u32` as the first field of `PluginDescriptor` with rustdoc explaining the additive-growth mechanism.

  **`fidius-macro/src/interface.rs::generate_descriptor_builder`:**
  - Emits `descriptor_size: std::mem::size_of::<#crate_path::descriptor::PluginDescriptor>() as u32` in the generated builder. Because the builder is `const unsafe fn`, this evaluates at plugin compile time and reflects the plugin's view of the struct (including any future fields the plugin was built with).

  **Tests:**
  - `layout_and_roundtrip.rs::descriptor_size_and_align` — still asserts 80 bytes / 8-byte alignment. The new `descriptor_size` u32 at offset 0 pairs with `abi_version` u32 at offset 4, occupying what was previously 4 bytes padding. Size unchanged.
  - `layout_and_roundtrip.rs::descriptor_field_offsets` — added `descriptor_size` at 0; `abi_version` shifted from 0 to 4; all other offsets unchanged.
  - `layout_and_roundtrip.rs::version_constants` — asserts `ABI_VERSION == 100` (was 2).
  - `impl_basic.rs::descriptor_fields_are_correct` — asserts `desc.abi_version == 100`.

  **Docs:**
  - `docs/reference/abi-specification.md` — updated the Version Constants section with the derivation rule (pre-1.0 vs post-1.0) and ADR-0002 reference. Updated the PluginDescriptor Layout section: added `descriptor_size` at offset 0; fixed the long-stale `method_count` field that was missing from the table (offset 72); updated total size from 72 to 80.

  **Validation:**
  - `angreal test` — all suites pass when run normally, modulo the pre-existing e2e parallel-flake (2 sig tests clobber each other; documented in prior tasks, passes serially)
  - `angreal lint` — clean
  - Manual smoke: `cargo run -p fidius-cli -- inspect tests/test-plugin-smoke/target/debug/libtest_plugin_smoke.dylib` — loads successfully with the new descriptor

  **Unblocks:**
  - I-0014 (Arena), I-0016 (drop JSON), I-0018 (method metadata) — all three can now land against the derived ABI_VERSION (100 at 0.1.0) and add fields at the end of `PluginDescriptor` per the additive discipline.