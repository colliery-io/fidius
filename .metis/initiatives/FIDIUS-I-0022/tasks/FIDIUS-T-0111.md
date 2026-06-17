---
id: g-2-gate-plugin-impl-cdylib
level: task
title: "G.2 — Gate #[plugin_impl] cdylib machinery off wasm + point wasm adapter at fidius-guest"
short_code: "FIDIUS-T-0111"
created_at: 2026-06-17T11:25:25.111557+00:00
updated_at: 2026-06-17T11:47:36.712950+00:00
parent: FIDIUS-I-0022
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0022
---

# G.2 — Gate #[plugin_impl] cdylib machinery off wasm + point wasm adapter at fidius-guest

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0022]]

## Objective **[REQUIRED]**

Gate the `#[plugin_impl]` cdylib machinery off `target_family="wasm"` and fix the wasm adapter's interface-hash reference, so a `crate = "fidius_guest"` plugin compiles to a component without the host-only cdylib code.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] cdylib shims/free_buffer/vtable/descriptor/inventory-registration emitted under `#[cfg(not(target_family = "wasm"))]`.
- [x] Native path unaffected (cfg true on native): full workspace + cdylib integration/e2e green (39 ok sections; 16 integration + 6 e2e pass).
- [x] `Client` already `#[cfg(feature="host")]`-gated → auto-excluded for a wasm author crate; companion module compiles on wasm (refs `fidius_guest::descriptor`).
- [x] Adapter `fidius-interface-hash` reads `super::__fidius_<Trait>::<Trait>_INTERFACE_HASH` (bug fix) + `use super::*;` brings the trait into scope. Verified: macro author crate builds to a valid component (T-0112).

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Per-item `#[cfg(not(target_family="wasm"))]` on each cdylib piece (shims distribute the attr across the repetition; vtable/descriptor statics + inventory submit prefixed). Lowest-risk: no module/scoping changes, and the cfg is true on native so native codegen is byte-identical.

### Dependencies
Depends on [[FIDIUS-T-0110]] (the split). Paired with [[FIDIUS-T-0112]] (proves the wasm build). Together they unblock [[FIDIUS-T-0106]].

### Risk Considerations
Risk was to the working native cdylib/Python path — mitigated by the cfg being true on native (no native change) and verified by the full cdylib integration/e2e suites.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.** cdylib machinery gated off wasm; adapter hash-ref fixed (companion module) + `use super::*;`. Native regression green (39 ok; 16 integration + 6 e2e). Proven end-to-end by T-0112: the macro author crate builds to a valid wasm component.