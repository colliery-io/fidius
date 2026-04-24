---
id: package-toml-python-runtime-fields
level: task
title: "package.toml Python runtime fields + fidius-cli inspect validation"
short_code: "FIDIUS-T-0087"
created_at: 2026-04-24T00:09:35.691483+00:00
updated_at: 2026-04-24T15:45:13.184936+00:00
parent: FIDIUS-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0020
---

# package.toml Python runtime fields + fidius-cli inspect validation

## Parent Initiative

[[FIDIUS-I-0020]]

## Objective

Extend `package.toml` so a package can declare itself as a Python plugin: a `runtime` field on the package header and a `[python]` section for entry-module + requirements path. Wire `fidius-cli inspect` to validate and surface these fields. No loading happens here — only manifest schema + tooling.

## Scope

- `fidius_core::package::PackageHeader` gains `runtime: Option<String>` (serde default `"rust"`, allowed values `"rust"` and `"python"`).
- New `PythonPackageMeta { entry_module: String, requirements: Option<String> }` deserialized from a `[python]` section when `runtime = "python"`. Default `requirements` path is `"requirements.txt"`.
- Validation: `runtime = "python"` with no `[python]` section, or `[python]` with no `entry_module`, fails parsing with a clear error.
- `fidius-cli inspect` prints `runtime` and (when python) `entry_module` + resolved requirements path.
- Tests: existing manifest tests still pass; new tests cover python-runtime parsing, missing-entry_module rejection, unknown-runtime rejection.

## Acceptance Criteria

## Acceptance Criteria

- [x] All existing `package::tests` pass unchanged (25 still pass; 7 new added).
- [x] New tests cover python-runtime happy path, requirements-default fallback, and four validation failures (missing python section, stray python section, unknown runtime, runtime display).
- [x] `fidius package inspect` on a python package prints `runtime`, and under a `Python:` block the `entry_module` + resolved `requirements` path (default included).
- [x] `cargo doc -p fidius-core` builds clean.

## Dependencies

None — runs in parallel with T-0085 and T-0086.

## Implementation Notes

- `runtime` is on the `[package]` header (not a separate top-level field) so it's discoverable in the same place as `name`, `version`, `interface`. Matches cloacina's pattern (`[metadata] language = "python"`) but promotes it to the header so it doesn't depend on the host's metadata schema type.
- Reject unknown `runtime` values explicitly rather than treating them as the default — better forward compatibility (a future `runtime = "node"` package shouldn't silently fall back to rust loading).

## Status Updates

### 2026-04-24 — landed

- `PackageHeader.runtime: Option<String>` (serde default), with `runtime()` (lenient → defaults to Rust on unknown) and `runtime_strict()` (errors on unknown — used by load_manifest).
- New `PackageRuntime { Rust, Python }` enum with `as_str()` + `Display`.
- New `PythonPackageMeta { entry_module, requirements }` with `requirements_path()` defaulting to `"requirements.txt"`.
- `PackageManifest<M>` gains `python: Option<PythonPackageMeta>` + `validate_runtime()`. Cross-section invariants enforced after serde parse.
- New `PackageError::InvalidManifest(String)` variant for the validator's failures.
- `load_manifest` now calls `runtime_strict()` then `validate_runtime()` after parsing — so unknown runtimes and stray/missing `[python]` sections fail at load.
- `fidius package inspect` prints `Runtime: ...` always, plus a `Python:` block (entry_module + resolved requirements path) for python packages.
- 7 new tests added (rust-default, python-happy-path, requirements-default, missing-python-section, stray-python-section, unknown-runtime, display). All 32 `package::tests` pass.
- `angreal lint`, `angreal check`, `angreal test` all clean across the workspace (33 test groups green).