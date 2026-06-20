---
id: pc-1-rich-wit-type-mapping-maps
level: task
title: "PC.1 — Rich WIT type mapping (maps, tuples, nesting)"
short_code: "FIDIUS-T-0152"
created_at: 2026-06-20T15:39:18.699224+00:00
updated_at: 2026-06-20T15:44:01.988061+00:00
parent: FIDIUS-I-0031
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: FIDIUS-I-0031
---

# PC.1 — Rich WIT type mapping (maps, tuples, nesting)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0031]]

## Objective **[REQUIRED]**

Extend `fidius-wit`'s type mapping (`crates/fidius-wit/src/lib.rs`, `wit_type_with`) so WIT-projected plugin interfaces support **maps, tuples, and nested generics**. Today only `Vec<T>`, `Option<T>`, `Result<T, PluginError>`, primitives, `String`, and `Vec<u8>` map; `HashMap`, tuples, and deeper nesting are rejected. Map `HashMap<K,V>`/`BTreeMap<K,V>` → `list<tuple<k,v>>`; tuples `(A, B, …)` → `tuple<a, b, …>`; and let `Vec`/`Option` nest (`Vec<Record>`, `Vec<Option<T>>`, `Option<Vec<T>>`). Extend the `Value`↔`Val` marshalling in `crates/fidius-host/src/executor/wasm.rs` to round-trip the new shapes (map as `list<tuple>`, tuple as `Val::Tuple`), and remove/narrow the "non-string-keyed maps not yet supported" rejection (`wasm.rs` ~993). Foundation for PC.2 ([[FIDIUS-T-0153]]).

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

- [ ] `fidius-wit` maps `HashMap<K,V>`/`BTreeMap<K,V>` → `list<tuple<k,v>>`, tuples → `tuple<...>`, and nested `Vec`/`Option` compose; unit tests in `crates/fidius-wit` cover each.
- [ ] The WASM `Value`↔`Val` path round-trips a map (`list<tuple>`) and a tuple; the "non-string-keyed maps not yet supported" rejection in `wasm.rs` is removed or narrowed.
- [ ] A records-style WASM fixture/test exercises a method with a map and a tuple arg/return end to end (extend `records_wasm` or add a fixture).
- [ ] The WIT type-mapping doc (`docs/explanation/wasm-component-abi.md`) lists the new mappings.
- [ ] `angreal test` and `angreal lint` are green.

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

**WIT side — DONE.** `crates/fidius-wit/src/lib.rs`: `HashMap`/`BTreeMap<K,V>` →
`list<tuple<k,v>>`, non-empty tuples → `tuple<...>`, nesting already recursed.
Added `two_generics` + `maps_tuples_and_nesting` test; fixed `generate.rs`
`unsupported_type_errors` (HashMap→`Box`). `cargo test -p fidius-wit` green (16).

**Marshalling — design.** `wasm.rs` `value_to_val`/`val_to_value` are structural/
type-blind. Inherent ambiguities: a Rust tuple serializes to `Value::List` (no
`Value::Tuple`) but WIT wants `Val::Tuple`; and WIT projects BOTH `HashMap<K,V>` and
`Vec<(K,V)>` to `list<tuple>`. Plan: (1) args type-directed via `func.params()`
(`Type::Tuple`→`Val::Tuple`, `Value::Map` vs `list<tuple>` type → `Val::List<Val::Tuple>`);
remove the `Value::Map` rejection. (2) returns: keep `val_to_value` structural, override
`deserialize_map` in `fidius-guest/src/value.rs` so a `Value::List` of pairs deserializes
to a map (so `from_value::<HashMap>` + `<Vec<(K,V)>>` both work). (3) fixture/E2E + doc.