---
id: p1-2-unify-error-model-fold
level: task
title: "P1.2 — Unify error model: fold PythonCallError into CallError with a backend variant"
short_code: "FIDIUS-T-0095"
created_at: 2026-06-17T03:23:56.042853+00:00
updated_at: 2026-06-17T03:40:10.880606+00:00
parent: FIDIUS-I-0021
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P1.2 — Unify error model: fold PythonCallError into CallError with a backend variant

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Collapse the two divergent call-error types into one so the unified `PluginExecutor` trait can return a single error type. `fidius-host` returns `CallError`; `fidius-python` returns `PythonCallError`. Fold Python's error into `CallError`, adding a backend/runtime-specific variant that preserves the exception message and traceback.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `CallError` represents a Python exception (via `Plugin(PluginError)`, traceback preserved in `details`) and a future WASM trap (new `Backend { runtime, message }` variant), plus a backend-agnostic `WireModeMismatch` variant — without losing any existing cdylib variant.
- [x] Existing `CallError` variants and meanings preserved (`Plugin`, `InvalidMethodIndex`, `UnknownStatus`, `Serialization`, `Deserialization`, `Panic`, `BufferTooSmall`, `NotImplemented`) — change is purely additive.
- [x] `PythonCallError` reduced to a layer-agnostic internal type with `From<PythonCallError> for CallError` defined host-side under the `python` feature (fidius-python cannot depend on fidius-host, so the conversion can't live in fidius-python).
- [x] Mapping preserves traceback: `P::Plugin(err) → CallError::Plugin(err)` keeps the `PluginError.details` traceback built by `pyerr_to_plugin_error`. (The conversion is *consumed* by the Pyo3Executor in FIDIUS-T-0098; the mapping itself lands here.)
- [x] Compiles with and without the `python` feature; `cargo test -p fidius-host` green. Change is additive so existing error-path tests are unaffected; Python exception-path tests still pass against `PythonCallError` (unchanged).

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Audit `crates/fidius-host/src/error.rs` (`CallError`) and `crates/fidius-python/src/handle.rs` (`PythonCallError`). Extend `CallError` (e.g. a `Backend { runtime, message, details }` variant) and provide a `From<PythonCallError>` bridge during migration, then inline. Keep `PluginError` structured semantics intact.

### Dependencies
None. Prerequisite for FIDIUS-T-0096 (the `PluginExecutor` trait returns `CallError`).

### Risk Considerations
Do not flatten distinct cdylib status codes into a generic string — preserve the structured variants callers match on. The Python traceback must survive into `details`.

## Status Updates **[REQUIRED]**

**2026-06-16 — COMPLETE.** Added two variants to `CallError` in `crates/fidius-host/src/error.rs`: `WireModeMismatch { method, declared, attempted }` (backend-agnostic raw/typed enforcement) and `Backend { runtime, message }` (runtime faults like a future WASM trap, distinct from plugin-raised `PluginError`). Added `#[cfg(feature = "python")] impl From<PythonCallError> for CallError` mapping all five `PythonCallError` variants onto unified ones, preserving `Plugin(PluginError)` (and its traceback `details`).

**Key architecture note for downstream tasks:** fidius-python intentionally does NOT depend on fidius-host (the host optionally depends on it — reverse would cycle). So `PythonCallError` stays in fidius-python as the layer-agnostic error; the unification lives host-side. The future `Pyo3Executor` (T-0098) will call into `PythonPluginHandle` and `?`/`.map_err(CallError::from)` to surface `CallError`.

**Also discovered (informs T-0096/T-0098):** the Python path already serialises via **serde_json**, not bincode (`call_typed_json`), with a `value_bridge` module mapping `serde_json::Value ↔ PyObject`. The `call_typed(bincode)` name is a documented holdover that just forwards to `call_typed_json`. This is direct precedent for the self-describing `fidius_core::Value` design.

Verified: `cargo check -p fidius-host` (default) and `--features python` both compile; `cargo test -p fidius-host` green. Purely additive — no existing behaviour changed.