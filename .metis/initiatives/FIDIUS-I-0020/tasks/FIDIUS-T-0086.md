---
id: python-side-fidius-sdk-module
level: task
title: "Python-side fidius SDK module (decorators + registry + error mapping)"
short_code: "FIDIUS-T-0086"
created_at: 2026-04-24T00:09:31.205059+00:00
updated_at: 2026-04-24T15:33:14.042954+00:00
parent: FIDIUS-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0020
---

# Python-side fidius SDK module (decorators + registry + error mapping)

## Parent Initiative

[[FIDIUS-I-0020]]

## Objective

Ship the Python-side `fidius` SDK module that plugin authors `from fidius import method`. Provides the `@method` decorator that registers a Python function as a callable plugin method, the registry the loader uses to look up callables, and a `PluginError` exception class plugin code can `raise` for typed error returns.

## Scope

- New directory `python/fidius/` with `__init__.py`, `_registry.py`, `_errors.py`, `pyproject.toml`.
- `@method` decorator: registers function under its `__name__` in a module-level registry. Duplicate registrations raise.
- `get_method(name)` accessor used by the Rust-side loader (PyO3) to look up callables by name.
- `PluginError` Python exception with `code`, `message`, and optional `details` (dict). The PyO3 error-mapper recognises this class and round-trips it as a fidius `PluginError` with the same fields rather than a generic exception.
- pytest suite: decorator registration, duplicate detection, `raise PluginError(...)`, importing the module from a vendored `sys.path` location.
- Module-level docstring documents author-facing usage.
- Structured so it can be vendored into a plugin's `vendor/` directory by `pip install fidius --target vendor/` once published, or by hand-copying `python/fidius/` for local dev.

## Acceptance Criteria

## Acceptance Criteria

- [x] `angreal python-test` passes (8 tests; runs pytest in a managed venv via `@venv_required`).
- [x] `@method` works on multiple functions in one module; duplicates raise.
- [x] `raise PluginError("BAD_INPUT", "msg", details={"k": "v"})` carries fields correctly in pure-Python tests (full host round-trip deferred to T-0089).
- [x] `python/fidius/` is importable when its parent directory is on `sys.path` (vendor pattern verified by `test_module_importable_from_vendor_layout`).

## Dependencies

None — runs in parallel with T-0085.

## Implementation Notes

- Pin no external runtime deps in `pyproject.toml`. The module is intentionally pure-stdlib so authors can vendor it without pulling other packages.
- Reserve the `fidius` PyPI name for this module, or pick an alternative consistent with the broader fidius naming question already noted in user memory.

## Status Updates

### 2026-04-24 — landed

- New `python/fidius/` package: `__init__.py`, `_registry.py`, `_errors.py`, `pyproject.toml`, `README.md`. Pure stdlib, zero runtime deps.
- `@method` decorator registers under `func.__name__`, raises `ValueError` on duplicates, returns the function unchanged so it stays transparent.
- `PluginError(code, message, details=None)` exception with `code`, `message`, `details` attributes; `str(exc)` returns the message.
- `get_method`, `list_methods`, `reset_registry` accessors for the host-side dispatcher (T-0089) and tests.
- 8 `unittest`-style tests (using pytest harness) covering: name registration, identity preservation, multi-method, duplicate-raise, unknown-key, PluginError fields with/without details, and the **vendored-load pattern** (copy `fidius/` into a temp dir, prepend to `sys.path`, decorate from the vendored copy).
- New angreal task `python-test` at `.angreal/task_python_test.py` using `@venv_required(python/.venv, requirements=["pytest"])`. First run creates the venv + installs pytest; subsequent runs reuse. Keeps the project-root `.venv` (uv-managed for other purposes) untouched.
- `angreal python-test` → 8 passed. `angreal lint`, `angreal check` clean.