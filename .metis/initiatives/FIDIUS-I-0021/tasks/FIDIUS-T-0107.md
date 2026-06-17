---
id: p3-2-fidius-pack-build-validate
level: task
title: "P3.2 — fidius pack: build + validate the component, precompile .cwasm, archive into .fid"
short_code: "FIDIUS-T-0107"
created_at: 2026-06-17T09:50:07.862360+00:00
updated_at: 2026-06-17T09:50:07.862360+00:00
parent: FIDIUS-I-0021
blocked_by: ["FIDIUS-T-0106"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P3.2 — fidius pack: build + validate the component, precompile .cwasm, archive into .fid

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Extend `fidius pack` to handle `runtime = "wasm"` packages: build (or accept a prebuilt) the `.wasm` component, validate it, optionally precompile to `.cwasm`, and archive into a `.fid` exactly like Rust/Python packages.

## Acceptance Criteria **[REQUIRED]**

- [ ] `fidius pack` on a wasm package produces a `.fid` containing the `.wasm` component (+ optional `.cwasm` + `world.wit`), reusing `fidius_core::package::pack_package` / the hardened `unpack_package`.
- [ ] Pack validates the component (is a Component Model component; exports the declared interface + `fidius-interface-hash`) via `wasm-tools`/wasmtime, and validates the `[wasm]` manifest (component file present, capability names known).
- [ ] Optional `.cwasm` precompile at pack time (`Engine::precompile_component`), written into the package and recorded in `[wasm].precompiled`; load uses the AOT path (already consumed by [[FIDIUS-T-0103]]). The wasmtime version is stamped so a stale `.cwasm` is ignored (fall back to JIT).
- [ ] A prebuilt `.wasm` is accepted without the component build toolchain (pack doesn't force `cargo component`).
- [ ] E2E: pack a wasm plugin → `.fid` → unpack → `load_wasm` → call.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Extend the `fidius-cli` pack command + the `fidius-core::package` pack path; mirror the Python vendor-at-pack structure (FIDIUS-T-0088). Validate via `wasm-tools` (library or subprocess). `.cwasm` precompile needs a wasmtime `Engine` — either a `fidius-host` helper exposed for the CLI or a `wasmtime` dep on the CLI behind a feature.

### Dependencies
[[FIDIUS-T-0106]] (a macro-built component to pack). Phase 2 load path DONE. Blocks [[FIDIUS-T-0108]] and [[FIDIUS-T-0109]].

### Risk Considerations
`.cwasm` is engine/version-specific — stamp the wasmtime version and have load reject/ignore a mismatch (fall back to JIT, per the T-0103 note). Keep pack usable without the toolchain when a prebuilt `.wasm` is supplied. Don't bloat `.fid` by archiving both `.wasm` and a large `.cwasm` unless precompile is requested.

## Status Updates **[REQUIRED]**

Not started — Phase 3 of FIDIUS-I-0021.