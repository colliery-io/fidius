---
id: remove-json-wire-format-drop
level: task
title: "Remove JSON wire format: drop WireFormat enum, cfg(debug_assertions), and wire_format descriptor field"
short_code: "FIDIUS-T-0077"
created_at: 2026-04-18T00:55:37.820906+00:00
updated_at: 2026-04-18T01:01:22.806394+00:00
parent: FIDIUS-I-0016
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0016
---

# Remove JSON wire format: drop WireFormat enum, cfg(debug_assertions), and wire_format descriptor field

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0016]]

## Objective

Drop JSON entirely as a fidius wire format. Remove the `cfg(debug_assertions)` gate in `fidius-core/src/wire.rs`, the `WireFormat` enum, the `wire_format: u8` descriptor field, and all host-side validation around it. Bincode becomes the single wire format. Per user directive: "it's bit me in the ass too many times; we're stable enough on 'this works' that i'm not debugging manually any more."

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

- [x] `fidius-core/src/wire.rs` — no `cfg(debug_assertions)`, no JSON path, no `WIRE_FORMAT` const. Single bincode `serialize` / `deserialize` pair. `WireError::Json` variant removed.
- [x] `fidius-core/src/descriptor.rs` — `WireFormat` enum removed; `wire_format_kind()` method removed; `wire_format: u8` field removed from `PluginDescriptor`
- [x] `fidius-macro/src/interface.rs` — descriptor builder no longer emits `wire_format`
- [x] `fidius-host/src/error.rs` — `WireFormatMismatch` and `UnknownWireFormat` variants removed
- [x] `fidius-host/src/loader.rs` — PluginInfo construction drops wire_format; `validate_against_interface` signature drops `expected_wire` parameter
- [x] `fidius-host/src/handle.rs` — `from_descriptor` no longer populates `wire_format`
- [x] `fidius-host/src/host.rs` — `expected_wire` field and `.wire_format()` builder method removed; `WireFormat` import gone
- [x] `fidius-host/src/types.rs` — `PluginInfo.wire_format` field removed
- [x] `fidius/src/lib.rs` — `WireFormat` removed from re-exports
- [x] `fidius-cli/src/commands.rs::inspect` — no longer prints wire format
- [x] Tests in `layout_and_roundtrip.rs` updated: `wire_format_layout` test removed; `wire_debug_produces_json` / `wire_release_produces_bincode` replaced with `wire_is_bincode_always`; descriptor field offsets updated (buffer_strategy moves from 41 to 40)
- [x] `docs/reference/abi-specification.md`, `docs/reference/errors.md`, and `docs/explanation/wire-format.md` updated
- [x] Full `angreal test` passes (modulo pre-existing parallel e2e flake)
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

  **Code removal across the workspace:**
  - `fidius-core/src/wire.rs` rewritten to a 30-line bincode-only module. Removed `WIRE_FORMAT` const, `WireError::Json` variant, `serde_json` usage, and all `cfg(debug_assertions)` branches.
  - `fidius-core/src/descriptor.rs` — removed `WireFormat` enum and `Display` impl. Removed `wire_format: u8` field from `PluginDescriptor`. Removed `wire_format_kind()` method.
  - `fidius-macro/src/interface.rs::generate_descriptor_builder` — stopped emitting the `wire_format` field.
  - `fidius-host/src/error.rs` — removed `WireFormatMismatch` and `UnknownWireFormat` variants.
  - `fidius-host/src/loader.rs` — dropped `wire_format` from `PluginInfo` construction. `validate_against_interface` signature drops the `expected_wire: Option<WireFormat>` parameter (callers updated in host.rs).
  - `fidius-host/src/handle.rs` — `from_descriptor` no longer populates `wire_format`.
  - `fidius-host/src/host.rs` — dropped `WireFormat` import, `expected_wire` field on both `PluginHost` and `PluginHostBuilder`, and the `.wire_format(...)` builder method. Updated the two `validate_against_interface` call sites.
  - `fidius-host/src/types.rs` — dropped `wire_format` field from `PluginInfo`.
  - `fidius/src/lib.rs` — removed `WireFormat` from the descriptor re-export list.
  - `fidius-cli/src/commands.rs::inspect` — removed the "Wire format" line from output.

  **Tests updated:**
  - `fidius-core/tests/layout_and_roundtrip.rs`:
    - Deleted `wire_format_layout` test.
    - Replaced `wire_debug_produces_json` + `wire_release_produces_bincode` with a single `wire_is_bincode_always` test asserting the output is bincode regardless of profile.
    - Updated `descriptor_field_offsets` — `buffer_strategy` moved from offset 41 to offset 40 (wire_format u8 removal freed the byte at offset 40); padding before `plugin_name` grew from 6 to 7 bytes; total size unchanged at 80.

  **Docs updated:**
  - `docs/reference/abi-specification.md` descriptor table — removed wire_format row, buffer_strategy moves to offset 40.
  - `docs/reference/errors.md` — removed `WireFormatMismatch` variant section.
  - `docs/explanation/wire-format.md` — full rewrite. Now explains the single-bincode decision, why JSON was dropped (pre-1.0 profile-mixing pain), and how to inspect wire bytes for debugging. Retained the `PluginError.details` stringified-JSON explanation because that's unchanged.

  **Validation:**
  - `angreal test` — all suites pass (minus the pre-existing parallel e2e signing flake, which passes serially). Notable: plugin rebuilt against the new descriptor layout and loads successfully via the full pipeline (discover → load → call via Client).
  - `angreal lint` clean.
  - The workspace still has `serde_json` deps in `fidius-core` (used by `PluginError::details_value()` which intentionally kept the stringified-JSON convention) and `fidius-cli` (crates.io lookup). The JSON dep isn't actually removable without also redesigning `PluginError::details` — out of scope.

  **No ABI version change:** ABI_VERSION stays at 100 (derived from 0.1.0). This change landed as part of the cumulative 0.1.0 ABI batch alongside FIDIUS-I-0019 (descriptor_size) and the upcoming FIDIUS-I-0014 / I-0018.