---
id: r-02-add-method-count-to
level: task
title: "R-02: Add method_count to PluginDescriptor and bounds-check vtable"
short_code: "FIDIUS-T-0043"
created_at: 2026-03-29T16:29:52.770524+00:00
updated_at: 2026-03-29T16:50:22.982838+00:00
parent: FIDIUS-I-0007
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0007
---

# R-02: Add method_count to PluginDescriptor and bounds-check vtable

**Addresses**: COR-01, COR-02, SEC-04, LEG-02, EVO-01, API-01, OPS-10

## Parent Initiative

[[FIDIUS-I-0007]]

## Objective

Add a `method_count: u32` field to `PluginDescriptor`, generate it from the macro, validate it at load time, and bounds-check in `call_method`. Also check for null function pointers before calling. This is the single most cross-cutting issue, affecting 9+ findings across all 7 review lenses. An out-of-bounds vtable index currently causes undefined behavior, and calling an unimplemented optional method dereferences a null function pointer.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `method_count: u32` field added to `PluginDescriptor` (appended at end)
- [ ] Layout tests updated in `fidius-core/tests/layout_and_roundtrip.rs` with new size/offset assertions
- [ ] `ABI_VERSION` incremented to 2
- [ ] `method_count` generated in descriptor builder by counting all methods (required + optional) in `fidius-macro/src/interface.rs`
- [ ] Method count passed when constructing descriptor in `fidius-macro/src/impl_macro.rs`
- [ ] `method_count` copied from descriptor to `PluginHandle` at construction time in `fidius-host/src/handle.rs`
- [ ] Bounds check added in `call_method`: `if index >= self.method_count as usize { return Err(CallError::MethodNotFound { index }); }`
- [ ] Null function pointer check added after reading fn_ptr: `if fn_ptr.is_null() { return Err(CallError::NotImplemented { bit: index as u32 }); }`
- [ ] Static assertion added in generated code: `const _: () = assert!(std::mem::size_of::<Option<FfiFn>>() == std::mem::size_of::<FfiFn>());`
- [ ] Tests added: out-of-bounds index returns error, unimplemented optional method returns `NotImplemented`

## Implementation Notes

### Technical Approach

1. Add `method_count: u32` to `PluginDescriptor` in `fidius-core/src/descriptor.rs` (appended at end to minimize layout disruption).
2. Update `fidius-core/tests/layout_and_roundtrip.rs` with new size/offset assertions.
3. Increment `ABI_VERSION` to 2 (acceptable at alpha with no deployed plugins).
4. In `fidius-macro/src/interface.rs`, generate `method_count` in the descriptor builder by counting all methods (required + optional).
5. In `fidius-macro/src/impl_macro.rs`, pass the method count when constructing the descriptor.
6. In `fidius-host/src/handle.rs`, add `method_count` to `PluginHandle` (copied from descriptor at construction time).
7. In `call_method`, before the unsafe pointer read: `if index >= self.method_count as usize { return Err(CallError::MethodNotFound { index }); }`.
8. After reading the function pointer, check for null: `if fn_ptr.is_null() { return Err(CallError::NotImplemented { bit: index as u32 }); }`. This activates the existing dead `NotImplemented` variant.
9. Add a static assertion in generated code: `const _: () = assert!(std::mem::size_of::<Option<FfiFn>>() == std::mem::size_of::<FfiFn>());`.
10. Add tests: out-of-bounds index returns error, unimplemented optional method returns `NotImplemented`.

### Dependencies

Breaking ABI change. Coordinate with any test fixtures. FIDIUS-T-0037 (R-01) should be done first (simpler, independent).

## Status Updates

*To be added during implementation*