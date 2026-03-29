---
id: r-10-make-pluginhandle-new-crate
level: task
title: "R-10: Make PluginHandle::new() crate-private"
short_code: "FIDIUS-T-0049"
created_at: 2026-03-29T17:19:45.789515+00:00
updated_at: 2026-03-29T17:30:10.040243+00:00
parent: FIDIUS-I-0008
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0008
---

# R-10: Make PluginHandle::new() crate-private

**Addresses**: API-06, API-14 | **Effort**: 1-2 hours

## Objective

Restrict `PluginHandle::new()` and `LoadedPlugin` fields to crate-private visibility so that users cannot bypass validation by constructing handles with arbitrary function pointers. Provide a convenience method for the common load-and-construct workflow.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `PluginHandle::new()` visibility changed to `pub(crate)`
- [ ] All `LoadedPlugin` fields changed to `pub(crate)`
- [ ] `PluginHost::load_handle(name) -> Result<PluginHandle, LoadError>` convenience method added that combines `load()` + `from_loaded()`
- [ ] Existing internal callers within fidius-host continue to compile
- [ ] Public API users can only construct handles via `PluginHost::load_handle()` or `PluginHandle::from_loaded()`
- [ ] All tests pass

## Implementation Notes

1. In `fidius-host/src/handle.rs`, change `pub fn new(...)` to `pub(crate) fn new(...)`.
2. In `fidius-host/src/loader.rs` (or wherever `LoadedPlugin` is defined), change public fields to `pub(crate)`. Add accessor methods if external read access is needed.
3. Add `PluginHost::load_handle()` in `fidius-host/src/host.rs` (or wherever `PluginHost` is defined) as a convenience wrapper.
4. If `new()` must remain public for advanced/unsafe use cases in the future, mark it `unsafe` with documented safety preconditions rather than leaving it unrestricted.

### Dependencies

- None. May require updating downstream code if `LoadedPlugin` fields are accessed directly outside the crate.

### Files

- `fidius-host/src/handle.rs` -- restrict new() visibility
- `fidius-host/src/loader.rs` -- restrict LoadedPlugin fields

## Status Updates

*To be added during implementation*