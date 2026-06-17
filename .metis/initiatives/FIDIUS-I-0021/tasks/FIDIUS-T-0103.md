---
id: p2-3-manifest-wasm-section-loader
level: task
title: "P2.3 — Manifest [wasm] section + loader + PluginHost routing + Backend::Wasm + .cwasm"
short_code: "FIDIUS-T-0103"
created_at: 2026-06-17T04:33:16.633895+00:00
updated_at: 2026-06-17T05:40:17.540749+00:00
parent: FIDIUS-I-0021
blocked_by: [FIDIUS-T-0102]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P2.3 — Manifest [wasm] section + loader + PluginHost routing + Backend::Wasm + .cwasm

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Wire WASM packages through `PluginHost` end-to-end: a `[wasm]` manifest section, a `PluginHandle` constructor + `Backend::Wasm` variant, `PluginHost::load_wasm` returning the unified `PluginHandle`, and `.cwasm` precompilation. Mirrors the Python plumbing from Phase 1 (per ADR [[FIDIUS-A-0003]], Path B).

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `fidius-core` manifest gains `[wasm]` (`WasmPackageMeta { component, precompiled?, capabilities[] }`); `validate_runtime` requires it for `runtime = "wasm"` and rejects stray `[wasm]` on rust/python (and vice-versa). Core's 46 lib tests still pass.
- [x] `PluginHandle::from_wasm(WasmComponentExecutor)` + `#[cfg(feature="wasm")] Backend::Wasm(..)` arm in every match site (`call_method` via `Value`, `call_method_raw`, `info`, `method_metadata`/`trait_metadata` → empty).
- [x] `PluginHost::load_wasm(name, &WasmInterfaceDescriptor)` finds the package, reads `[wasm]`, builds the executor + `PluginInfo { runtime: Wasm, .. }`, validates the interface hash (mismatch → `LoadError::InterfaceHashMismatch`), and returns the unified `PluginHandle`. (`WasmInterfaceDescriptor`/`WasmMethodDesc` added to fidius-core, mirroring the Python descriptor.)
- [x] `.cwasm` handling: a pre-supplied `[wasm].precompiled` is loaded via `Component::deserialize` (AOT ~83 µs path); otherwise JIT from the `.wasm` component. (Pack-time `.cwasm` *generation* is Phase 3 / T-0014.)
- [x] Load round-trip through `PluginHost` verified: `load_wasm_through_host_and_call` (typed `greet` + raw `echo-bytes` via the unified handle), `load_wasm_rejects_interface_hash_mismatch`, `discover_surfaces_wasm_package` — 8/8 in `tests/wasm_executor.rs`. Default (no-feature) build unaffected.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Mirror the Phase-1 Python loader/manifest/host plumbing (`PythonPackageMeta` → a `WasmPackageMeta`; `load_python` → `load_wasm`; `from_python` → `from_wasm`). Precompile with `Engine::precompile_component`/`Component::serialize`; deserialize the cached `.cwasm`.

### Dependencies
[[FIDIUS-T-0102]] (executor). Phase 1 is DONE (the `PluginExecutor`/`ValueExecutor` traits, the `Backend` enum on `PluginHandle`, and `fidius_core::Value`); the `PackageRuntime::Wasm` seat and wasm discovery from [[FIDIUS-T-0099]]; per ADR [[FIDIUS-A-0003]] (Path B). Blocks [[FIDIUS-T-0104]] and [[FIDIUS-T-0105]].

### Risk Considerations
Keep the `Backend` match arms exhaustive across all `PluginHandle` methods. `.cwasm` is engine/version-specific — document that it must be rebuilt when the wasmtime/toolchain version changes; validate the `.cwasm` matches the current engine or fall back to JIT.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.**
- `fidius-core`: `WasmPackageMeta` + `manifest.wasm` field + `validate_runtime` arms; new `wasm_descriptor` module (`WasmInterfaceDescriptor`/`WasmMethodDesc`).
- `fidius-host`: `Backend::Wasm` + `PluginHandle::from_wasm` + all match arms; `LoadError::WasmLoad`; `PluginHost::{find_wasm_package, load_wasm}` returning the unified `PluginHandle`, with interface-hash validation and the `.cwasm` (AOT) vs `.wasm` (JIT) load split.
- Tests (`tests/wasm_executor.rs`, 8/8): executor round-trips (5) + host load round-trip, hash-mismatch rejection, discovery (3).

Verified: `cargo test -p fidius-host --features wasm --test wasm_executor` 8/8; default `cargo check -p fidius-host` clean; `cargo test -p fidius-core --lib` 46/46. Pack-time `.cwasm` generation deferred to Phase 3 (T-0014); load already consumes a pre-supplied one.