---
id: fidius-python-python-functions-as
level: initiative
title: "fidius-python — Python functions as first-class fidius plugins"
short_code: "FIDIUS-I-0020"
created_at: 2026-04-22T12:37:14.746135+00:00
updated_at: 2026-04-24T12:41:07.940045+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: L
initiative_id: fidius-python-python-functions-as
---

# fidius-python — Python functions as first-class fidius plugins Initiative

## Context

Fidius today targets Rust plugin authors writing cdylibs. That's a powerful substrate — compiled, type-safe, signed, loaded by a host that knows nothing about the plugin's guts — but it excludes a large class of plugin authors who work in Python: data scientists iterating on model inference, analysts writing column transforms, researchers prototyping algorithms before productionizing them.

The whole reason to add Python as a plugin language is to let those authors **skip the compile step**. Any design that forces them to install Rust, run cargo, and build per-architecture cdylibs defeats the purpose. Prior art in `../cloacina/crates/cloacina-python/` confirms the viable path: embed Python in the host via PyO3, load plugin `.py` files directly, dispatch calls from Rust into the embedded interpreter.

A related boundary-overhead exploration (`pluggable-poc/BENCHMARK_REPORT.md`) characterised five transport strategies between Rust and Python. Its relevance here is narrower than originally thought: once we commit to an in-process embedded interpreter, the cost model simplifies to "PyO3 conversion overhead" (single-digit µs per call for small payloads), and the subprocess-isolation tiers it measured become out-of-scope for v1.

## Decisions (design phase closed 2026-04-23)

1. **Delivery format.** Plugins ship as a directory: `package.toml` + `.py` file(s) + optional `requirements.txt` + `package.sig`. No cdylib, no Cargo, no Rust toolchain for the author. Packages distribute as `.fid` archives exactly like Rust plugins today — `fidius_core::package::pack_package` and the hardened `unpack_package` (FIDIUS-T-0084) work unchanged.
2. **Host integration.** Hosts opt into Python support by depending on a new `fidius-python` crate. That crate plugs into `PluginHost` so that during discovery, packages whose manifest identifies a Python runtime are loaded via the Python loader instead of the cdylib loader. A `PluginHandle` backed by Python exposes the same `call_method` / `call_method_raw` API as a cdylib-backed handle — callers cannot tell the difference.
3. **Isolation tier.** **InProcess only.** Host links `libpython` via PyO3; one embedded interpreter serves all Python plugins in the host process. Calling a method is a direct PyO3 call with Python-native argument conversion. No IPC, no subprocess, no msgpack on the per-call hot path. This matches cloacina's proven pattern.
4. **Wire format.** PyO3 type conversions (`IntoPyObject` / `FromPyObject`) handle control-plane args natively. Bulk-payload methods declared `#[wire(raw)]` (FIDIUS-T-0082) cross as raw `bytes` — no encoding step. **msgpack** is the chosen format only for the narrow case of Python plugins that implement a Rust trait method whose args are complex types not covered by PyO3's built-in conversions; in practice most interfaces will not need it.
5. **Python runtime + dependency model.** System Python (bootstrap interpreter provided by deployer). Per-package pip dependencies are **vendored at pack time**, not materialised at load time: `fidius pack` runs `pip install -r requirements.txt --target ./vendor/` (when `requirements.txt` is present and `vendor/` is absent) and archives the result alongside the `.py`. At load time, fidius-python prepends `<plugin_dir>/vendor` and `<plugin_dir>` to `sys.path` and imports the plugin module — no pip, no venv creation, no cache directory. The `.fid` archive is fully self-contained including deps; signing covers the vendored tree because it's part of the package digest. **Constraint:** all Python plugins loaded into one host process share `sys.modules`. If two plugins vendor conflicting versions of the same library, the first one loaded wins for both. This is treated as a deployment error to be resolved by the operator (matches cloacina's existing pattern). Per-plugin sub-interpreter isolation is an out-of-scope follow-on.
6. **Trait ↔ Python method contract.** Plugin `.py` declares which fidius interface it implements in `package.toml` (`interface = "greeter"`, `interface_version = 1`). The `fidius` CLI gains a `python-stub` subcommand that reads a published interface crate and emits a Python stub (type hints + expected interface-hash constant) that plugin authors import. At load time the Python loader checks the embedded hash against the host's expectation — same mismatch protection the cdylib's `interface_hash` provides, plumbed through a constant instead of a descriptor.
7. **Timeouts.** Deferred to a future follow-up feature. InProcess tier cannot safely enforce them (Rust cannot interrupt a thread mid-PyO3-call). Documented via FIDIUS-T-0083.

## Goals

- Plugin author ships **no Rust**. Directory of `.py` files + manifest is the entire artifact.
- Host API unchanged. `PluginHost`, `PluginHandle`, `Client`, signing, packaging, `.fid` archives — all work identically whether the plugin is Rust or Python.
- One shared embedded interpreter per host process; per-package vendored dependencies via `<plugin>/vendor` on `sys.path`.
- Host/plugin disagreement on interface surfaces at load time as a clear error, never as silent data corruption.
- Plugin-raised Python exceptions become `PluginError` with the exception message and traceback preserved.
- Documentation includes a "write your first Python fidius plugin" walkthrough.

## Non-Goals

- Subprocess / Process-tier isolation for v1. Recorded as an out-of-scope follow-on.
- Built-in timeout / cancellation semantics. Deferred to a separate future feature.
- Changing fidius's cdylib ABI to accommodate Python. Python support is additive — the cdylib loader path is unaffected.
- A "universal Python cdylib" runtime-configured adapter. If demand emerges, it can be added as an alternative packaging form later.
- Bundling a standalone Python interpreter with fidius-python. System Python is the v1 assumption; bundling can be added as a host-side opt-in later.
- Other embedded languages (Node, Ruby, Lua). The interface manifest format should permit future `runtime = "..."` values, but no second language ships in this initiative.
- Hot-reload of a running plugin's `.py` without a host-level load cycle.

## Dependencies

- **FIDIUS-T-0082** (raw wire mode) — **landed**. Bulk-payload Python methods will use `#[wire(raw)]` to skip bincode at the host/fidius-python boundary.
- **FIDIUS-T-0083** (timeout documentation) — **landed**. The deferred "Process-tier timeouts" follow-up is the natural home for timeout semantics if/when a Process tier is added.
- **FIDIUS-T-0084** (safe archive extraction) — **landed**. `.fid` packages containing `.py` trees are extracted via the hardened path.
- No structural changes required to `fidius-core`, `fidius-host`, `fidius-macro`, or the ABI spec.

## Architecture Sketch

```
host binary
└── fidius-host PluginHost
    ├── (existing) cdylib loader → PluginHandle (vtable dispatch)
    └── fidius-python PythonLoader → PluginHandle (PyO3 dispatch)
        ├── embedded PyO3 interpreter (one per host process; shared sys.modules)
        ├── sys.path manager (prepend <plugin>/vendor + <plugin> on load)
        └── per-plugin module + per-method PyObject callables
```

Package on disk:

```
my-greeter/
├── package.toml         # runtime = "python", interface = "greeter", ...
├── requirements.txt     # msgpack==1.0.*  (optional, source of truth)
├── vendor/              # populated by `fidius pack` from requirements.txt
│   ├── msgpack/
│   └── ...
├── greeter.py           # @fidius.method def greet(name: str) -> str: ...
└── package.sig          # existing fidius signing over directory digest
```

## Success Criteria

- A plugin author writes a `.py` file + `package.toml` (+ `requirements.txt` if needed), runs `fidius pack`, and ships one `.fid` archive. No Rust, no cargo, no per-architecture builds.
- The host loads the package via `PluginHost` and calls methods through the standard `Client` interface with no Python-specific code on the caller side.
- A Python exception raised inside a plugin surfaces as `CallError::Plugin(PluginError { code, message, details })` with the traceback in `details`.
- An interface-hash mismatch between what the `.py` declares and what the host expects is rejected at load time with a clear error.
- A plugin with a `requirements.txt` gets its deps vendored under `vendor/` at pack time; load simply prepends that path and imports — no runtime pip, no cache directory.
- Per-call overhead for small control-plane methods is ≤ 10 µs (PyO3 conversion cost, consistent with cloacina's measurements).
- `angreal test` adds Python-plugin integration tests that exercise load, call, exception, raw-wire, and venv-reuse paths.

## Alternatives Considered

- **Ship plugins as cdylibs with embedded `.py`** (original recommendation in the design phase). Rejected: reintroduces the Rust toolchain requirement for the plugin author, which is exactly the friction we set out to remove. Keeps architectural purity at the expense of the primary user benefit.
- **Expose five POC tiers to the host.** Rejected: the POC exposed tier selection because its measurement setup needed to. A product should not leak implementation tiers into its host-facing API. InProcess is the one tier required for v1.
- **Subprocess (Process) isolation for v1.** Rejected after clarification. Without timeouts (deferred), Process's remaining value is crash containment — real, but not v1-blocking for the trusted-plugin use case. Can be added later as an opt-in tier when the timeout feature justifies it.
- **WASI-Python.** Rejected for v1. The numerical / data-science ecosystem we actually want to target (`numpy`, `pyarrow`, `pandas`, `scikit-learn`) does not run reliably under WASI-Python today. Revisit when the ecosystem catches up.
- **RPC to a long-lived Python service.** Rejected: breaks the "one file you copy" deploy model and forces fidius to manage service lifecycle.
- **Bundled standalone Python** (python-build-standalone). Rejected for v1: unnecessary complexity when `.py` plugins are a host-side integration that the deployer is already provisioning. Revisit if deployer friction shows up.

## Out-of-Scope Follow-Ons

- **Process-tier isolation with timeouts.** Its own future initiative. Justified by untrusted-plugin scenarios and by being the only tier that can safely enforce deadlines.
- **Other embedded languages** (Node, Ruby, Lua) via a similar loader-plug pattern. Manifest format leaves `runtime = "..."` open.
- **Bundled Python runtime** (python-build-standalone) as a host-side opt-in for deployers who want reproducible interpreter versions.
- **Hot-reload** of a plugin's `.py` without a host load cycle.
- **Per-plugin resource quotas** (memory, CPU).
- **Per-plugin sub-interpreter isolation** (CPython 3.12+ PEP 684) so conflicting dep versions don't clash in `sys.modules`. Natural follow-on if the shared-`sys.modules` constraint becomes painful.
- **A GUI or notebook-based plugin authoring experience.**

## Implementation Plan

To be decomposed into tasks as a direct next step. Skeleton:

1. **fidius-python crate skeleton** — new crate, PyO3 dep, feature-gated, shared embedded-interpreter initialisation, error-type plumbing.
2. **Python-side SDK** — `fidius` pip-installable package (or vendored module) with `@method` decorator, registry, exception-to-PluginError mapping.
3. **Package manifest extension** — `runtime = "python"` field in `package.toml` header; `[python]` section for `entry_module` and optional `requirements` path; validation in `fidius-cli inspect`.
4. **Vendor-at-pack support** — `fidius pack` detects Python packages, runs `pip install -r requirements.txt --target vendor/` when `vendor/` is absent, includes the result in the archive. Respects a pre-existing `vendor/` (plugin author may pre-vendor for reproducibility). Small helper in fidius-python (or a simple path-prepend function) handles the load-time sys.path insertion.
5. **PythonLoader** — reads package directory, validates interface hash, prepends `<plugin>/vendor` and `<plugin>` to `sys.path`, imports the entry module, returns a `PluginHandle` whose dispatcher is a Python-runtime implementation.
6. **PluginHandle Python dispatcher** — runs under the embedded interpreter, looks up Python callable for each method index, performs PyO3 conversion for args/returns, translates Python exceptions to `PluginError`.
7. **PluginHost integration** — route packages by `runtime` field to the right loader; keep cdylib path unaffected.
8. **`fidius python-stub` CLI subcommand** — generate a `.py` stub from a published interface crate (method signatures + interface-hash constant).
9. **Raw-wire support** — `#[wire(raw)]` methods dispatch through `call_method_raw`; Python side receives `bytes` natively and returns `bytes`.
10. **End-to-end test** — at least one Python plugin package implementing an existing fidius interface, loaded + called through `PluginHost` in integration tests.
11. **Documentation walkthrough** — "write your first Python fidius plugin," the venv caching story, the interface-hash mismatch error, raw-wire guidance.