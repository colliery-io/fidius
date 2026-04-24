---
id: fidius-python-crate-skeleton
level: task
title: "fidius-python crate skeleton + embedded interpreter"
short_code: "FIDIUS-T-0085"
created_at: 2026-04-24T00:09:27.072791+00:00
updated_at: 2026-04-24T12:52:57.470056+00:00
parent: FIDIUS-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0020
---

# fidius-python crate skeleton + embedded interpreter

## Parent Initiative

[[FIDIUS-I-0020]]

## Objective

Stand up the `fidius-python` crate skeleton with PyO3, a single shared embedded-interpreter initialiser, and the error-conversion plumbing that translates `PyErr` into fidius's `CallError` / `PluginError`. No loader, no dispatcher, no manifest parsing — pure foundation that subsequent tasks build on.

## Scope

- New crate `crates/fidius-python` added to the workspace.
- Cargo.toml: depend on `pyo3` (with `auto-initialize`), `fidius-core`, `fidius-host`, `tracing`, `thiserror`.
- Module structure: `lib.rs`, `interpreter.rs` (idempotent `ensure_initialized()`), `error.rs` (PyErr → PluginError helper preserving code, message, traceback in `details`).
- A smoke integration test that initialises the interpreter and evaluates `1 + 1` to prove PyO3 is wired correctly.
- The crate compiles cleanly under `angreal lint` / `angreal check` and uses no host-feature on `fidius` (it depends on `fidius-host` directly because it's a host-side crate).

## Acceptance Criteria

## Acceptance Criteria

- [x] `crates/fidius-python` is in the workspace and builds.
- [x] An integration test runs Python code via the embedded interpreter.
- [x] `pyerr_to_plugin_error` converts a Python exception into a `PluginError` with code = exception class name, message = `str(exception)`, details containing the formatted traceback.
- [x] `angreal lint` and `angreal check` pass.

## Dependencies

None — root of the initiative's dependency chain.

## Implementation Notes

- The `auto-initialize` PyO3 feature is fine for v1 (host process owns the interpreter for its lifetime). Manual interpreter management can come later if a host wants to control lifecycle.
- `ensure_initialized()` should be idempotent so any entry point can call it without coordination.
- Don't introduce a global `Mutex<PyInterpreter>` — PyO3's `Python::with_gil` is the right concurrency primitive.

## Status Updates

### 2026-04-24 — landed

- New crate `crates/fidius-python` with `lib.rs`, `interpreter.rs`, `error.rs`, `tests/smoke.rs`, and `build.rs`.
- Workspace `Cargo.toml`: added the crate as a member and `pyo3 = { version = "0.25", features = ["auto-initialize"] }` as a workspace dep.
- `interpreter::ensure_initialized()` is idempotent and uses `Python::with_gil` to trigger PyO3's auto-initialize. No `Mutex<PyInterpreter>`.
- `error::pyerr_to_plugin_error()` extracts `__class__.__name__` → `code`, `str(exc)` → `message`, and `traceback.format_tb(...)` → `details` as a JSON object.
- `build.rs` mirrors the cloacina-build pattern: emits `cargo:rustc-link-arg=-Wl,-rpath,...` so macOS framework Python builds can find their dylib at runtime. Without this the test binary aborted on launch with `dyld: Library not loaded: @rpath/Python3.framework/...`.
- 3 tests pass (1 inline error-mapping test + 2 smoke tests for interpreter + traceback).
- `angreal lint`, `angreal check` clean. Existing crates unaffected (default-off feature plumbing arrives in T-0090).