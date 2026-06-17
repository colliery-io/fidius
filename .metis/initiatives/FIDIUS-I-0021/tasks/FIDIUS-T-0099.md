---
id: p1-6-unify-runtime-routing
level: task
title: "P1.6 — Unify runtime routing: PluginRuntimeKind {Cdylib, Python, Wasm} + PluginHost dispatch via executor"
short_code: "FIDIUS-T-0099"
created_at: 2026-06-17T03:24:02.105648+00:00
updated_at: 2026-06-17T04:20:48.092969+00:00
parent: FIDIUS-I-0021
blocked_by: [FIDIUS-T-0097, FIDIUS-T-0098]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P1.6 — Unify runtime routing: PluginRuntimeKind {Cdylib, Python, Wasm} + PluginHost dispatch via executor

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Make `PluginHost` route by runtime through the executor abstraction. Extend `PluginRuntimeKind` to `{ Cdylib, Python, Wasm }` and ensure discovery selects the right executor by manifest `runtime`, leaving a clean seat for the Phase 2 WASM executor.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `PluginRuntimeKind` has `Cdylib`, `Python`, `Wasm` (+ `is_wasm()`); `PackageRuntime` gains `Wasm` with lenient + strict parsing (`runtime_strict` errors list `rust`/`python`/`wasm`), `as_str`/`Display`, and `validate_runtime`. Unknown runtimes are no longer silently treated as `wasm`/`rust`. The `Wasm` loader is unimplemented (Phase 2); discovery surfaces wasm packages but there is no silent wasm load path.
- [x] `PluginHost` discovery maps manifest `runtime` → kind via `discover_package` (Python→Python, Wasm→Wasm, Rust source dirs skipped — discovered via their dylib). `load_python` yields a unified `PluginHandle` over `Pyo3Executor`; cdylib via `load`/`from_loaded` over `CdylibExecutor`.
- [x] `PluginInfo.runtime` set correctly: Python via the unified `load_python`/discovery path (no separate side type — returns `PluginHandle` now), cdylib via the descriptor.
- [x] Cross-backend routing covered: `integration.rs` calls cdylib through `PluginHandle` via `PluginHost`; `python_routing::load_python_dispatches_through_host` calls Python through `PluginHandle` via the same `PluginHost` API; new `integration::discover_surfaces_wasm_package_with_wasm_runtime` proves the third (Wasm) routing seat in the default suite. (A single heavyweight dual-load test was judged redundant given this coverage.)

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Touch `crates/fidius-host/src/host.rs` (discovery/loader routing — extends FIDIUS-T-0090) and `types.rs` (`PluginRuntimeKind`). Keep a loader-plug pattern so Phase 2 adds the real `Wasm` arm without further host churn.

### Dependencies
Depends on FIDIUS-T-0097 and FIDIUS-T-0098 (both executors must exist). Feeds FIDIUS-T-0100.

### Risk Considerations
Don't reintroduce backend-specific branches in caller-facing code — routing stays inside the host. The `Wasm` arm must fail clearly, never silently no-op.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.**
- `fidius-core/src/package.rs`: `PackageRuntime::Wasm` + `as_str`/`Display`/`runtime()`/`runtime_strict()`/`validate_runtime()` arms. "wasm" now parses to `Wasm` (was silently `Rust`).
- `fidius-host/src/types.rs`: `PluginRuntimeKind::Wasm` + `is_wasm()`.
- `fidius-host/src/host.rs`: renamed `discover_python_package` → `discover_package`, routing by `runtime()` (Python/Wasm surfaced; Rust dirs skipped). (Python's unified `load_python` return was done in T-0098.)
- Test: `integration::discover_surfaces_wasm_package_with_wasm_runtime` (default suite) — stages a `runtime = "wasm"` package, asserts discovery surfaces it with `PluginRuntimeKind::Wasm`.

Note: much of the python-side routing (unified `PluginHandle`, `runtime=Python`) already landed in T-0098; this task added the Wasm seat + generalised discovery + the default-suite routing test.

Verified: `cargo test -p fidius-host --test integration` (16, incl. new wasm test) + `cargo test -p fidius-core --lib` (46) pass; `cargo clippy` clean on core + host (default and `--features python`).