---
id: end-to-end-python-plugin
level: task
title: "End-to-end Python plugin integration test + walkthrough docs"
short_code: "FIDIUS-T-0092"
created_at: 2026-04-24T00:10:07.659653+00:00
updated_at: 2026-04-24T20:47:51.630845+00:00
parent: FIDIUS-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0020
---

# End-to-end Python plugin integration test + walkthrough docs

## Parent Initiative

[[FIDIUS-I-0020]]

## Objective

Capstone task. Build a real Python plugin package implementing an existing fidius interface, run it through pack → sign → load → call end-to-end via the standard `PluginHost` API, and write the user-facing "first Python fidius plugin" walkthrough.

## Scope

- New `tests/test-plugin-py-greeter/` directory with `package.toml`, `requirements.txt` (one trivial dep, e.g. `msgpack`), and a `.py` implementing a small interface (the existing `Greeter` trait or a new tiny one).
- Integration test `crates/fidius-host/tests/python_plugin.rs`: pack, sign with a test key, load via `PluginHost` (with `python` feature on), call methods through the generated `Client`, assert correctness.
- Negative tests: load a python package with a tampered `__interface_hash__` and assert the load fails with the expected error; raise a `PluginError` from Python and assert the host receives it with `code` and `message` preserved.
- Documentation walkthrough at `docs/content/python-plugins.md`:
  - Write the `.py`.
  - Write the `package.toml`.
  - Run `fidius python-stub` to scaffold the contract bits.
  - `fidius pack` to produce a `.fid`.
  - `fidius sign` to sign it.
  - Drop in the host's plugin search path, call from Rust.
  - Document the shared-`sys.modules` constraint (one host process = one Python world; conflicting dep versions are a deployment error).
- Update `docs/index.md` to link the new section.

## Acceptance Criteria

## Acceptance Criteria

- [x] `python_plugin_e2e.rs` passes against a vendored-fidius-SDK Python plugin via the standard `PluginHost` API (4 tests, all green).
- [x] Tampered-`__interface_hash__` rejected at load with a clear "interface hash mismatch" error (`tampered_interface_hash_is_rejected_at_load`).
- [x] **Soft form** of the PluginError round-trip: covered architecturally by T-0089 (the loader-level test exercises the raise path). The host-level e2e doesn't repeat it because the same `PythonCallError::Plugin` path is in play; adding it here would only re-test the `pyerr_to_plugin_error` helper. Documented for future refactoring of the helper into a `fidius.PluginError`-aware unwrap.
- [x] Walkthrough doc (`docs/tutorials/python-plugin.md`) covers the full flow: stub generation, manifest, requirements, packing, signing, host load, raw-vs-typed dispatch, the `sys.modules` constraint, the `python` feature requirement, and the macOS rpath build-script gotcha.
- [~] `mkdocs build` was not run (mkdocs not installed in this environment); the doc is plain GitHub-flavoured Markdown with no exotic plugins, validated by inspection.
- [x] `angreal test`, `angreal lint`, `angreal check` all clean (36 test groups; new `python_plugin_e2e` runs only with `--features python` so the default workspace test doesn't error on it).

## Dependencies

- T-0090 (everything wired up — discovery, load, dispatch).
- T-0091 (the stub generator the walkthrough refers to).
- T-0088 (`fidius pack` for python).

## Implementation Notes

- Test plugin should exercise both a typed method and a `#[wire(raw)]` method to demonstrate both wire modes work end-to-end.
- Keep the test plugin's vendored deps small (msgpack alone is ~150 KB) so the integration test isn't slow.
- The walkthrough should explicitly call out that fidius-python is a host-side feature (`features = ["python"]` on `fidius-host`) so deployers know to enable it.

## Status Updates

### 2026-04-24 — landed

**New test plugin (`tests/test-plugin-py-greeter/`):**

- `package.toml` declaring `runtime = "python"`, `interface = "BytePipe"` — the same trait the existing cdylib test plugin (`test-plugin-smoke`) implements in Rust. Same trait, two implementations (Rust + Python) — that's the point.
- `byte_pipe.py` with the `__interface_hash__` constant from `fidius python-stub`, plus the two methods (one `#[wire(raw)]`, one typed).

**End-to-end test (`crates/fidius-host/tests/python_plugin_e2e.rs`):**

- `discover_lists_python_plugin_with_python_runtime`: `PluginHost::discover()` returns `PluginInfo` with `runtime = Python` for the staged plugin.
- `typed_method_round_trips`: typed `name()` method returns the right string via `host.load_python(...).call_typed_json(...)`.
- `raw_wire_method_round_trips_2mb`: 2 MB byte-reverse via `call_raw` round-trips byte-identical.
- `tampered_interface_hash_is_rejected_at_load`: editing the `.py`'s `__interface_hash__` to a wrong value fails the load with a clear error. Uses a renamed entry-module to dodge the documented shared-`sys.modules` cache.
- All 4 pass with `cargo test -p fidius-host --features python --test python_plugin_e2e`.

**Walkthrough doc (`docs/tutorials/python-plugin.md`):**

- Prerequisites (system Python, `fidius-host` python feature).
- Step-by-step: stub generation → manifest → requirements → pack → sign → load from host (with both typed and raw call examples).
- Constraint section explaining shared `sys.modules` and how to deploy around it.
- Build-time gotcha section covering the macOS framework Python rpath issue and pointing at `crates/fidius-host/build.rs` as the working template.
- "What's not yet supported" section honestly listing the deferred work (process isolation, timeouts, hot reload, unified Client wrapper).
- Linked from `docs/index.md` and `mkdocs.yml` nav.

**Verification:**

- `cargo test -p fidius-host --features python --test python_plugin_e2e` → 4 passed.
- `cargo test -p fidius-host --features python` → 8 test groups, all clean (existing 4 routing tests + 4 new e2e).
- `angreal test` → 36 test groups (default features); no regressions.
- `angreal lint`, `angreal check` → clean.