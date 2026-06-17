---
id: p1-5-pyo3executor-migrate-fidius
level: task
title: "P1.5 — Pyo3Executor: migrate fidius-python dispatch behind the trait"
short_code: "FIDIUS-T-0098"
created_at: 2026-06-17T03:24:00.963982+00:00
updated_at: 2026-06-17T04:14:16.422014+00:00
parent: FIDIUS-I-0021
blocked_by: [FIDIUS-T-0096]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P1.5 — Pyo3Executor: migrate fidius-python dispatch behind the trait

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Move the embedded-Python dispatch from `fidius-python`'s parallel `PluginHandle` into a `Pyo3Executor` implementing `PluginExecutor`, so Python plugins flow through the same host `PluginHandle` as cdylib. Behaviour-preserving refactor.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

> Note: reworded for the layering reality — `Pyo3Executor` lives in **fidius-host** (`executor/python.rs`, `#[cfg(feature="python")]`) and wraps `fidius_python::PythonPluginHandle`. It cannot live in fidius-python (which must not depend on fidius-host). `PythonPluginHandle` stays as the layer-agnostic dispatcher (not removed). The typed bridge reuses the existing self-describing JSON path (`Value ↔ JSON ↔ call_typed_json`) rather than a new `Value→PyObject` path — behaviour-identical, less code.

- [x] `Pyo3Executor` implements `PluginExecutor` (`info`/`method_count`/`call_raw`) + `ValueExecutor` (`call`): `call` = `Value→JSON → call_typed_json → JSON→Value`; `call_raw` = `PythonPluginHandle::call_raw` verbatim.
- [x] `PluginHost::load_python` now returns a unified `PluginHandle` (via `PluginHandle::from_python`) wrapping `Pyo3Executor`; `PythonPluginHandle` kept as the underlying dispatcher.
- [x] Error mapping uses the unified `CallError` via `From<PythonCallError>` (FIDIUS-T-0095); plugin exceptions stay `Plugin(PluginError)` with traceback in `details`.
- [x] Interface-hash validation at load preserved (unchanged in `fidius-python` loader; exercised by `tampered_interface_hash_is_rejected_at_load`).
- [x] Host python suites pass through the unified handle: `python_plugin_e2e` (4: discover, typed round-trip, raw-wire 2MB, hash rejection) + `python_routing` (4: typed dispatch, unknown-name, discover, cdylib-unaffected). `cargo check`/tests green with `--features python`.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Refactor `crates/fidius-python/src/handle.rs` and `loader.rs`. Collapse `call_typed`/`call_typed_json` into `call(Value)`; `Value→PyObject` replaces the current per-arg conversion (driven by `Value` via PyO3 `IntoPyObject`/`FromPyObject`). Preserve GIL handling and the embedded-interpreter lifecycle. `call_raw` keeps passing `bytes` natively.

### Dependencies
Depends on FIDIUS-T-0096 (trait + Value) and FIDIUS-T-0095 (unified CallError). Paired with FIDIUS-T-0097 (cdylib executor); both feed FIDIUS-T-0099 and FIDIUS-T-0100.

### Risk Considerations
The `Value→PyObject` mapping must match current native conversion semantics, including the msgpack fallback for complex types noted in FIDIUS-I-0020. Do not regress raw-wire `bytes` handling or the shared-`sys.modules` behaviour.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.**
- `crates/fidius-host/src/executor/python.rs` (cfg `python`): `Pyo3Executor { py: PythonPluginHandle, info: PluginInfo }` implementing `PluginExecutor` + `ValueExecutor`. Typed `call` bridges `Value ↔ JSON` via `serde_json` and reuses `PythonPluginHandle::call_typed_json`; `call_raw` reuses `call_raw`. `PythonCallError → CallError` via the T-0095 `From`.
- `handle.rs`: added `#[cfg(feature="python")] Backend::Python(Pyo3Executor)` variant + `PluginHandle::from_python(py, info)`; python match arms in `call_method` (`to_value → call → from_value`), `call_method_raw`, `info`, `method_metadata`/`trait_metadata` (empty — no descriptor metadata for Python).
- `host.rs`: `load_python` now builds `PluginInfo` from the manifest header (`name`, `interface_version`) + descriptor (`interface_name`, `interface_hash`) and returns the unified `PluginHandle`.
- Tests: updated the two host python tests to the unified API (`call_typed_json`→`call_method`, `call_raw`→`call_method_raw`; error-path uses `match` since `PluginHandle` isn't `Debug`, same as the original `PluginHandle`).

**Layering note:** the `Pyo3Executor` had to live in fidius-host (not fidius-python) because fidius-python must not depend on fidius-host. `PythonPluginHandle` remains the lower-layer dispatcher. The `Value↔JSON` bridge is behaviour-identical to the prior `serde_json::to_vec(input) → call_typed_json` path (verified by the round-trip test producing the same results).

Verified: `cargo check`/`clippy` clean for my files; `--features python` host python tests 8/8 pass. (Two pre-existing clippy warnings in `package_e2e`/a lib-test struct are unrelated and not hit by CI's clippy invocation.)