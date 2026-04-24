---
id: pythonloader-python-backed
level: task
title: "PythonLoader + Python-backed PluginHandle dispatcher"
short_code: "FIDIUS-T-0089"
created_at: 2026-04-24T00:09:50.320314+00:00
updated_at: 2026-04-24T20:28:06.311782+00:00
parent: FIDIUS-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0020
---

# PythonLoader + Python-backed PluginHandle dispatcher

## Parent Initiative

[[FIDIUS-I-0020]]

## Objective

The big one. Read an unpacked Python package directory, validate the interface hash, prepend `vendor/` and the plugin directory to `sys.path`, import the entry module, and build a `PluginHandle` whose dispatcher calls Python by method index. Support both typed methods (PyO3 conversion of args/returns) and `#[wire(raw)]` methods (raw bytes both ways).

## Scope

- `PythonLoader::load(package_dir, interface_descriptor) -> Result<PluginHandle, LoadError>`.
- The `interface_descriptor` argument carries everything the dispatcher needs at call time: expected interface hash, ordered list of method names + per-method wire mode (typed vs raw). Sourced from the host's interface crate via the same `__fidius_<Trait>` companion module that today drives the cdylib path.
- `sys.path` management: insert `<plugin_dir>/vendor` and `<plugin_dir>` at index 0 before import. Track inserted paths so a later unload (if added) can pop them.
- Interface-hash validation: read `__interface_hash__` from the imported entry module; compare against the descriptor's hash. Mismatch → `LoadError::InterfaceHashMismatch`.
- New `PluginHandle` flavour: backed by `PythonHandle { entry_module: Py<PyAny>, method_callables: Vec<Py<PyAny>> }`. The `_library: Option<Arc<Library>>` field stays `None`; a parallel `_python: Option<PythonHandle>` field keeps the runtime alive.
- Typed `call_method`: msgpack-encode the bincode-decoded arg tuple → pass to the Python callable as a `bytes` arg unpacked by the SDK on the Python side, OR convert the args directly via PyO3's `IntoPyObject` chain (decision deferred to implementation; PyO3-native conversion is preferred when the arg types support it). Returns msgpack-encoded or PyO3-converted result back to bincode.
- Raw `call_method_raw`: pass `&[u8]` as a Python `bytes` directly, expect `bytes` back. Zero encoding hops.
- Exception → `PluginError` mapping via the helper from T-0085. `fidius.PluginError` raises map to typed code/message/details; other exceptions map to `code = "PYTHON_ERROR", message = str(exc), details = {"traceback": ...}`.
- Integration test: small Python plugin implementing a simple existing interface (e.g. `Greeter` from the test plugin), loaded via `PythonLoader`, called via the standard `Client`. Passing a string round-trips correctly. Raising a `PluginError` surfaces with code/message preserved.
- Raw integration test: a Python plugin implementing the `BytePipe` trait's `reverse` method via `#[wire(raw)]`. Verify a 2 MB payload round-trips byte-identical (mirroring the cdylib-side test from T-0082).
- Document the shared-`sys.modules` constraint in module docs and walkthrough.

## Acceptance Criteria

## Acceptance Criteria

- [x] Loader rejects a package whose `__interface_hash__` doesn't match the descriptor (`interface_hash_mismatch_is_rejected`).
- [x] Loader successfully imports a vendored-deps plugin — `make_plugin` test fixture vendors the in-tree `python/fidius` SDK into `vendor/` and the loader imports it via the prepended `sys.path`.
- [x] Typed call round-trips a primitive (`String` → `String`) and a composite struct with a few fields (`typed_call_round_trip_string`, `typed_call_with_struct_args`).
- [x] Raw `call_raw` round-trips a 2 MB payload byte-identical (`raw_call_round_trip_2mb`).
- [~] A Python `raise PluginError(...)` surfaces with the **message** preserved (✅) and a traceback in `details` (✅), but `code` is currently the class name `"PluginError"` rather than the user-supplied code, and the user-supplied `details` dict isn't flattened into the host's `details` field. Captured as a follow-on enhancement to `pyerr_to_plugin_error` so it special-cases the `fidius.PluginError` class. Documented in the test comment for the deferred work.
- [x] A non-PluginError Python exception (`ValueError`, `KeyError`, etc.) surfaces with `code = <ExceptionClassName>` and the formatted traceback in `details` (verified by T-0085's existing tests; the same path runs for non-PluginError raises).

## Dependencies

- T-0085 (crate skeleton, error helper).
- T-0086 (Python-side SDK — provides `@method` registration and `PluginError`).
- T-0087 (manifest fields — loader needs to read `[python].entry_module`).

## Implementation Notes

- Open question to resolve during implementation: how the `interface_descriptor` reaches the loader. Cleanest path: add an `IntoPythonDescriptor` trait (or a similar small piece of generated metadata) emitted by `#[plugin_interface]`'s companion module. That keeps the interface trait the single source of truth.
- Follow cloacina's `cloacina-python::loader` for the embedded-interpreter + `sys.modules` injection patterns. The "synthetic module" trick they use for `cloaca` may be useful for exposing fidius runtime helpers to plugin code.
- This is the largest task in the initiative — likely 7–14 days of work. If it grows further during implementation, split out the raw-wire dispatch path as a follow-up task rather than letting this one sprawl.

## Status Updates

### 2026-04-24 — landed

**Architecture:**

- New `fidius_core::python_descriptor::{PythonInterfaceDescriptor, PythonMethodDesc}` carries the interface name, hash, and per-method (name + wire_raw) info to the loader. `'static`-shaped so it lives in `.rodata`.
- `#[plugin_interface]` macro emits `<TraitName>_PYTHON_DESCRIPTOR` as a static in the companion module alongside the existing `_INTERFACE_HASH`/`_VTable`/etc. Cdylib plugins get this for free; cost is a few bytes of `.rodata`.
- `fidius` facade re-exports `python_descriptor` so plugin-author crates can reach `MyTraitClient`'s descriptor without touching `fidius-core` directly.

**Loader (`fidius-python::loader`):**

- `load_python_plugin(package_dir, &'static descriptor) -> Result<PythonPluginHandle, PythonLoadError>`.
- Validates `manifest.runtime == "python"` and the `[python]` section is present.
- Prepends `<dir>/vendor` and `<dir>` to `sys.path` (idempotent — repeat loads of the same path don't double-insert).
- Imports the entry module, validates `__interface_hash__` against the descriptor, resolves each method by direct `getattr(module, name)`.

**Handle (`fidius-python::handle`):**

- `PythonPluginHandle` holds the imported module + a `Vec<Py<PyAny>>` aligned with descriptor method order.
- Two dispatch entry points:
  - `call_typed_json(method_index, json_bytes)`: takes JSON-encoded args (the host wire for python plugins is JSON, not bincode — bincode is non-self-describing so it can't pivot through `serde_json::Value`). Converts JSON → Python primitives via `value_bridge`, calls, converts result back to JSON bytes.
  - `call_raw(method_index, &[u8])`: bytes in, bytes out, no encoding.
- Both error on wire-mode mismatch (e.g. calling a typed method via `call_raw`) and out-of-range indices.
- Python exceptions become `PythonCallError::Plugin(PluginError)` via the `pyerr_to_plugin_error` helper from T-0085.

**Value bridge (`fidius-python::value_bridge`):**

- `value_to_pyobject(py, &Value) -> PyResult<Bound<PyAny>>` — recursive serde JSON → Python.
- `pyobject_to_value(&Bound<PyAny>) -> PyResult<Value>` — recursive Python → serde JSON.
- Inline test verifies round-trip for primitives, arrays, nested objects.

**Python SDK refinement (T-0086 follow-up):**

- The `@method` registry is now per-module (`(module_name, method_name) -> Callable`) rather than global. Otherwise multiple plugins loaded into one host (the deliberate v1 behaviour) couldn't both expose `@method def process()`. SDK pytest still passes; the change is backward-compatible for single-module use.
- The loader skips the SDK registry and uses direct `getattr(module, name)` — the @method decorator is just a marker now, the registry survives only as a per-module convenience for SDK users.

**Tests (`crates/fidius-python/tests/loader_e2e.rs` — 7 tests):**

- typed string round-trip
- typed struct-arg round-trip (DoubleIn → DoubleOut)
- raw 2 MB byte-reverse round-trip
- `PluginError` raise → `CallError::Plugin` with traceback
- interface-hash mismatch rejected at load
- wire-mode mismatch errors (typed method called via call_raw)
- out-of-range method index errors
- Each test uses a unique entry-module name (`greeter_t0`, `greeter_t1`, ...) to dodge the shared-`sys.modules` cache — exactly the deployment constraint we documented in the initiative.

**Verification:**

- `cargo test -p fidius-python --test loader_e2e` → 7 passed.
- `cargo test -p fidius-python` → all suites green (smoke + value_bridge + loader_e2e + error mapping).
- `angreal lint`, `angreal check`, `angreal test` (34 test groups) → all clean.

**Deferred to follow-on:**

- `fidius.PluginError`-aware unwrap in `pyerr_to_plugin_error`: today the user's `code` and `details` dict get folded into the generic Python-exception message+traceback shape. A small special-case in the helper would surface them properly. Captured in the test comment.
- PluginHost integration (T-0090) — this task gives the building blocks; T-0090 wires the routing.