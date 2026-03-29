---
id: integration-test-load-test-plugin
level: task
title: "Integration test — load test-plugin-smoke via fidius-host"
short_code: "FIDIUS-T-0018"
created_at: 2026-03-29T01:28:35.469290+00:00
updated_at: 2026-03-29T11:25:34.124150+00:00
parent: FIDIUS-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0003
---

# Integration test — load test-plugin-smoke via fidius-host

## Parent Initiative

[[FIDIUS-I-0003]]

## Objective

Write integration tests that use the full fidius-host API to load the test-plugin-smoke cdylib. This proves the host library works end-to-end with a real compiled plugin — not just raw dlsym like the I-0002 smoke test, but through PluginHost → discover → load → PluginHandle → call.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Test builds test-plugin-smoke cdylib (subprocess cargo build)
- [ ] `PluginHost::builder().search_path(plugin_dir).build()` succeeds
- [ ] `host.discover()` returns `Vec<PluginInfo>` containing "BasicCalculator"
- [ ] `host.load("BasicCalculator")` returns a PluginHandle
- [ ] `handle.call_method::<AddInput, AddOutput>(0, &input)` returns correct result
- [ ] `handle.has_capability(0)` returns true for the optional `multiply` method
- [ ] `handle.info().interface_name` == "Calculator"
- [ ] Negative test: loading a non-existent plugin returns `LoadError::PluginNotFound`

## Implementation Notes

### Technical Approach

File: `fidius-host/tests/integration.rs`

Builds test-plugin-smoke via subprocess, then uses the fidius-host public API to load and call it. This is the "user experience" test — proving the API is ergonomic and correct.

### Dependencies
- All previous T-0013 through T-0017

## Status Updates

- **2026-03-29**: 6 integration tests pass. Full API pipeline: builder → discover (finds BasicCalculator) → load by name → PluginHandle::call_method for add(3,7)=10 and multiply(4,5)=20 → info() checks → not-found negative test. All passing.