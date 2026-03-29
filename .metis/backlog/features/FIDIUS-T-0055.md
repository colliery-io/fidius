---
id: r-21-generate-typed-host-side
level: task
title: "R-21: Generate typed host-side proxy from plugin_interface"
short_code: "FIDIUS-T-0055"
created_at: 2026-03-29T18:02:37.531001+00:00
updated_at: 2026-03-29T18:11:10.300162+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# R-21: Generate typed host-side proxy from plugin_interface

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective

Generate a typed client struct in the companion module (`__fidius_{Trait}`) that wraps `PluginHandle` and exposes named methods with correct input/output types. Eliminates raw `call_method(index, &input)` for the common case. Also generate method index constants.

Currently host code looks like:
```rust
let result: AddOutput = handle.call_method(0, &AddInput { a: 3, b: 7 })?;
```

After this task:
```rust
let client = __fidius_Calculator::Client::from_handle(handle);
let result = client.add(&AddInput { a: 3, b: 7 })?;
```

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `#[plugin_interface]` generates `pub struct {Trait}Client` inside the companion module
- [ ] Client has `from_handle(handle: PluginHandle) -> Self`
- [ ] Client has one method per trait method: `fn {method}(&self, input: &{InputType}) -> Result<{OutputType}, CallError>`
- [ ] Each method calls `self.handle.call_method::<I, O>(INDEX, input)` with the correct index
- [ ] Optional methods check `has_capability` first and return `CallError::NotImplemented` if absent
- [ ] Method index constants generated: `pub const METHOD_{NAME}: usize = N` in companion module
- [ ] Raw `call_method` API still available for dynamic/advanced use
- [ ] Tests: use the typed client to call methods, verify correct results
- [ ] E2E test updated to use typed client

## Implementation Notes

### Technical Approach

File: `fidius-macro/src/interface.rs`

In the companion module generation, add:

1. Method index constants from the IR's method list
2. A `{Trait}Client` struct wrapping `fidius_host::PluginHandle`
3. One method per trait method that delegates to `call_method` with the right index and types

The input/output types come from the `MethodIR` — each method's first arg type (after `&self`) is the input, return type is the output. For methods with multiple args, serialize as a tuple.

The client needs `fidius_host` types (`PluginHandle`, `CallError`) — these come through the facade. The generated code should reference `fidius::` paths.

### Dependencies
- R-02 (method_count + bounds checking) — done in Phase 1

## Status Updates

*To be added during implementation*