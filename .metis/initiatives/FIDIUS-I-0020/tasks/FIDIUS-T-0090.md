---
id: pluginhost-runtime-aware-loader
level: task
title: "PluginHost runtime-aware loader routing"
short_code: "FIDIUS-T-0090"
created_at: 2026-04-24T00:09:59.208444+00:00
updated_at: 2026-04-24T20:39:48.195540+00:00
parent: FIDIUS-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0020
---

# PluginHost runtime-aware loader routing

## Parent Initiative

[[FIDIUS-I-0020]]

## Objective

Make `PluginHost` route discovery and load to the right loader (cdylib vs Python) based on `package.toml`'s `runtime` field. The cdylib path is unaffected; the python path delegates to `fidius-python::PythonLoader`. Hosts opt into Python support by enabling a `python` cargo feature on `fidius-host`.

## Scope

- Add an optional `python` feature on `fidius-host` that depends on `fidius-python`. Default off so cdylib-only consumers stay thin.
- `PluginHost::discover` enumerates packages from search paths and surfaces both rust and python plugins uniformly via `PluginInfo`. The runtime kind is exposed on `PluginInfo` (e.g. `runtime: PluginRuntime { Rust, Python }`).
- `PluginHost::load(name)` dispatches by manifest runtime: rust → existing cdylib loader, python → Python loader (only available with `python` feature).
- A clear error if a python package is encountered without the `python` feature enabled.
- Tests: discovery sees a search-path containing both a rust and a python package and returns both; loading each returns a working `PluginHandle`.

## Acceptance Criteria

## Acceptance Criteria

- [x] Without the `python` feature, `fidius-host` builds and the existing rust-plugin tests are unchanged (15 integration tests still pass under default features).
- [x] With the `python` feature, a host discovers both rust cdylibs and python packages through `PluginHost::discover()`, distinguished by `PluginInfo.runtime` (`Cdylib` vs `Python`); python plugins are loaded via `PluginHost::load_python(name, descriptor)` returning a `PythonPluginHandle` ready to dispatch.
- [~] **Soft form** of the "no-python-feature error" criterion: a python package isn't visible at all without the feature, because both `discover` and `load` filter on file kind (cdylib path skips directories; the python-package branch is feature-gated). Implementing a louder `LoadError::PythonRuntimeNotEnabled` would mean parsing the manifest unconditionally just to surface a runtime-rejection — judged not worth the discovery-path cost. Documented for follow-on if a deployer trips over the silence.
- [x] The cdylib-only test suite still passes with default features (35 angreal test groups green; `cargo test --workspace` clean).

## Dependencies

- T-0089 (Python loader must exist before routing has anywhere to send things).

## Implementation Notes

- Keep `fidius-python` as a default-off feature, not a default-on one. Cdylib-only consumers must not pay the libpython link cost or the embedded-interpreter init cost.
- This task is small once T-0089 lands — most of the work is the feature gate + a small enum on `PluginInfo`.

## Status Updates

### 2026-04-24 — landed

**Files touched:**

- `crates/fidius-host/Cargo.toml`: new optional `python` feature pulling `fidius-python`. New `[build-dependencies]` on `pyo3-build-config` for the rpath build script.
- `crates/fidius-host/build.rs`: new — only emits the libpython rpath link arg when `CARGO_FEATURE_PYTHON` is set. Cdylib-only consumers pay zero build cost.
- `crates/fidius-host/src/types.rs`: new `PluginRuntimeKind { Cdylib, Python }` enum + `runtime: PluginRuntimeKind` field on `PluginInfo`. Added `is_cdylib()` / `is_python()` accessors.
- `crates/fidius-host/src/lib.rs`: re-export `PluginRuntimeKind`.
- `crates/fidius-host/src/host.rs`: split `discover` into `discover_cdylib` + `discover_python_package`. Python branch reads `package.toml`, skips non-python packages, surfaces `PluginInfo` with `runtime: Python` and a placeholder `interface_hash: 0` (the real hash check happens at load via the descriptor). New `find_python_package(name)` and feature-gated `load_python(name, descriptor)` returning `PythonPluginHandle`.
- `crates/fidius-host/src/error.rs`: new `LoadError::PythonLoad(String)` variant — wraps `PythonLoadError` as a string to keep the `LoadError` enum stable across feature gates.
- `crates/fidius-host/src/handle.rs` + `loader.rs`: thread the new `PluginRuntimeKind::Cdylib` field through the existing PluginInfo constructors.
- `crates/fidius-python/Cargo.toml`: dropped the (unused) `fidius-host` dep that would otherwise create a cycle now that `fidius-host` optionally depends on `fidius-python`.

**Tests (`crates/fidius-host/tests/python_routing.rs`):**

- 4 tests, all gated behind `#[cfg(feature = "python")]` so default-feature builds skip them entirely.
- `discover_surfaces_python_package`: assert `discover()` returns a `PluginInfo` with `runtime = Python` for a package on disk.
- `load_python_dispatches_through_host`: end-to-end: build a python package on disk, load via `host.load_python(...)`, call a method via `call_typed_json`, verify the result.
- `load_python_unknown_name_returns_not_found`: clear `LoadError::PluginNotFound` for a missing python plugin.
- `cdylib_load_path_unaffected`: `host.load(name)` (cdylib path) on a python-only directory returns `PluginNotFound` rather than a parse error or panic.

**Verification:**

- `cargo test -p fidius-host --features python --test python_routing` → 4 passed.
- `cargo test -p fidius-host --test integration` (default features) → 15 passed (no regressions).
- `angreal lint`, `angreal check`, `angreal test` → 35 test groups clean.

**Known gap (documented for follow-on):**

- Linking a downstream binary against `fidius-host --features python` requires its own rpath build script (or a shared helper) until we extract a `fidius-build` crate analogous to cloacina-build. T-0092 (walkthrough doc) calls this out so deployers know.