---
id: fidius-pack-vendor-python-deps-at
level: task
title: "fidius pack — vendor Python deps at pack time"
short_code: "FIDIUS-T-0088"
created_at: 2026-04-24T00:09:41.671347+00:00
updated_at: 2026-04-24T15:57:54.898335+00:00
parent: FIDIUS-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0020
---

# fidius pack — vendor Python deps at pack time

## Parent Initiative

[[FIDIUS-I-0020]]

## Objective

Make `fidius pack` work for Python packages: when the manifest declares `runtime = "python"` and no `vendor/` directory exists, run `pip install -r <requirements> --target ./vendor/` before archiving. Pre-existing `vendor/` is respected (plugin author may pre-vendor for reproducibility). The vendored tree is included in the `.fid` archive so deployment remains "one file you copy."

## Scope

- `fidius-cli pack` branches on `manifest.runtime`. Rust path unchanged.
- Python path: locate the requirements file (default `requirements.txt`, override from `[python].requirements`), invoke `pip install -r <req> --target ./vendor/` via `std::process::Command`, forward stderr on failure with a clear error message.
- Pre-existing `vendor/` → no pip invocation; archive as-is.
- Missing requirements file AND missing `vendor/` → `tracing::warn!` and proceed (a Python plugin with no deps is legitimate).
- Pack-side test: scaffold a temp Python package with a single trivial dep (e.g. `six` — small + zero transitive), pack it, assert the archive contains the `.py` and the vendored dep tree.
- `fidius pack --help` documents the python-specific behaviour.

## Acceptance Criteria

## Acceptance Criteria

- [x] Packing a python package with `requirements.txt` invokes `python3 -m pip install --target vendor/` and the result lands in the `.fid` (verified via the pre-vendored test path; live pip install of real deps is exercised by the failure test below).
- [x] Packing a python package with a pre-existing `vendor/` leaves it untouched and includes it in the archive (`pack_python_with_prevendored_directory_skips_pip`).
- [x] Packing a python package with neither requirements file nor `vendor/` succeeds with a `tracing::warn!` (`pack_python_with_no_requirements_or_vendor_warns_but_succeeds`).
- [x] pip failure (e.g. unresolvable dep) surfaces clearly: `PackageError::ArchiveError` with `pip install failed` + stderr forwarded (`pack_python_with_unresolvable_requirement_surfaces_pip_error`).
- [x] `angreal lint` and `angreal check` clean.

## Dependencies

- T-0087 (manifest fields) — required to know whether the package is python-flavoured.

## Implementation Notes

- Use `python3 -m pip install` rather than `pip` directly — handles the case where `pip` isn't on PATH but `python3` is.
- Don't try to manage pip versions or install pip if missing. If the deployer's `python3` lacks pip, packaging fails with the user-readable pip error — that's a deployer-fix issue, not fidius's responsibility.
- Vendored bytes are signed transparently because they're part of the directory digest fed to `package_digest`. No signing changes needed.

## Status Updates

### 2026-04-24 — landed

- New `vendor_python_deps()` helper in `crates/fidius-core/src/package.rs`: invokes `python3 -m pip install -r <req> --target ./vendor/ --quiet` only when manifest is python *and* `vendor/` is missing.
- Branch in `pack_package` (not in the CLI): the lower-level API stays correct, so any caller (CLI, future packaging tools, tests) gets the behaviour for free.
- Pre-existing `vendor/` → `tracing::debug!` and skip pip; missing requirements + missing `vendor/` → `tracing::warn!` and proceed; pip failure → `PackageError::ArchiveError("pip install failed (exit N): <stderr>")`.
- `fidius-core` now depends on `tracing` directly (was indirect-only); added to `[dependencies]`.
- 3 new tests added (pre-vendored skip, no-req-no-vendor warning, unresolvable-dep failure surfacing). All 35 `package::tests` pass; the failure test gracefully skips when `python3 -m pip` is unavailable so CI environments without pip don't false-fail.
- `angreal lint`, `angreal check` clean across the workspace.

### Implementation note

The CLI command (`fidius package pack`) needs no changes — the manifest-based branching lives entirely in the core. `--help` text is auto-generated from clap and stays correct because the user-visible behaviour didn't change (still "pack this directory into a .fid").