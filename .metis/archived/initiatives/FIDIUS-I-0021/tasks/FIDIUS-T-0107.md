---
id: p3-2-fidius-pack-build-validate
level: task
title: "P3.2 — fidius pack: build + validate the component, precompile .cwasm, archive into .fid"
short_code: "FIDIUS-T-0107"
created_at: 2026-06-17T09:50:07.862360+00:00
updated_at: 2026-06-17T12:17:12.273015+00:00
parent: FIDIUS-I-0021
blocked_by: [FIDIUS-T-0106]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P3.2 — fidius pack: build + validate the component, precompile .cwasm, archive into .fid

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Extend `fidius pack` to handle `runtime = "wasm"` packages: build (or accept a prebuilt) the `.wasm` component, validate it, optionally precompile to `.cwasm`, and archive into a `.fid` exactly like Rust/Python packages.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `fidius pack` on a wasm package produces a `.fid` containing the `.wasm` (+ optional `.cwasm`), reusing `fidius_core::package::pack_package` / the hardened `unpack_package` (archives the whole dir).
- [x] Pack validates the component is a valid Component-Model artifact (`wasmtime::component::Component::new` via `fidius_host::executor::validate_component`) and that the `[wasm].component` file is present. *Scope:* interface-name + `fidius-interface-hash` conformance is enforced at **load** (`load_wasm`), not re-checked at pack.
- [x] Optional `.cwasm` precompile at pack time (`Engine::precompile_component`), written into the package + recorded in `[wasm].precompiled`; load uses the AOT path. A stale `.cwasm` is **ignored** — wasmtime rejects the header and load falls back to JIT (verified by `stale_cwasm_falls_back_to_jit`).
- [x] A prebuilt `.wasm` is accepted without the build toolchain (pack never runs `cargo component`; without the CLI `wasm` feature it warns and archives as-is).
- [x] E2E: `pack_package` → `.fid` → `unpack_package` → `load_wasm` → call (`pack_unpack_load_roundtrip`).

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Extend the `fidius-cli` pack command + the `fidius-core::package` pack path; mirror the Python vendor-at-pack structure (FIDIUS-T-0088). Validate via `wasm-tools` (library or subprocess). `.cwasm` precompile needs a wasmtime `Engine` — either a `fidius-host` helper exposed for the CLI or a `wasmtime` dep on the CLI behind a feature.

### Dependencies
[[FIDIUS-T-0106]] (a macro-built component to pack). Phase 2 load path DONE. Blocks [[FIDIUS-T-0108]] and [[FIDIUS-T-0109]].

### Risk Considerations
`.cwasm` is engine/version-specific — stamp the wasmtime version and have load reject/ignore a mismatch (fall back to JIT, per the T-0103 note). Keep pack usable without the toolchain when a prebuilt `.wasm` is supplied. Don't bloat `.fid` by archiving both `.wasm` and a large `.cwasm` unless precompile is requested.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.**
- `fidius-host` (wasm feature): `validate_component` + `precompile_component` free fns (re-exported from `executor`).
- `load_wasm`: resolves `.cwasm` from `[wasm].precompiled` or an auto-detected sibling `<stem>.cwasm`; tries AOT, **falls back to JIT** on a rejected/stale `.cwasm` (tracing warn).
- `fidius-cli`: `wasm` feature → `fidius-host/wasm`; `pack --precompile`. wasm packages are validated; `--precompile` writes `<stem>.cwasm` + records `precompiled` in `package.toml` (comment-preserving string-insert). Without the `wasm` feature: warns (skips validation) / errors on `--precompile`. Pack never builds the component.
- Tests: `precompiled_cwasm_loads_via_aot_and_calls`, `stale_cwasm_falls_back_to_jit`, `pack_unpack_load_roundtrip` (host, `--features wasm`); CLI `wasm_pack.rs` (no-feature path: archive+warn, `--precompile` errors).
- Verified: native `cargo test --workspace` 41 ok / 0 failed; wasm suite 10 ok; `angreal lint` green.