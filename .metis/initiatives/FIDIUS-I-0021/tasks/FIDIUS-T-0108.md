---
id: p3-3-signing-inspect-for-wasm
level: task
title: "P3.3 — Signing + inspect for .wasm components (Ed25519 artifact-agnostic; inspect understands wasm runtime)"
short_code: "FIDIUS-T-0108"
created_at: 2026-06-17T09:50:14.312918+00:00
updated_at: 2026-06-17T12:26:27.539844+00:00
parent: FIDIUS-I-0021
blocked_by: [FIDIUS-T-0107]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P3.3 — Signing + inspect for .wasm components (Ed25519 artifact-agnostic; inspect understands wasm runtime)

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Ensure Ed25519 signing covers `.wasm` component packages (artifact-agnostic — nearly free) and `fidius inspect` understands the wasm runtime, so a wasm `.fid` signs/verifies/inspects like cdylib and Python.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `fidius sign`/`verify` work on a wasm package — signing is over `package_digest` (excludes `*.sig`, covers all files incl. the `.wasm`), no artifact-type assumptions. Tampering the component fails verification (CLI `sign_verify_and_tamper_wasm_package`).
- [x] `fidius inspect` on a wasm package reports `runtime = wasm`, interface + version, the component file, the `.cwasm` (or "none — JIT"), and the `[wasm].capabilities` allow-list (surfaced prominently). *Note:* the interface hash isn't in the manifest (it lives in the component + descriptor, validated at load), so inspect doesn't print it.
- [x] Load policy enforced for wasm identically to cdylib — new `signing::verify_package_signature`; `load_wasm` (and `load_python`, for true parity) call it when `require_signature`. Verified: signed loads, unsigned → `SignatureRequired`, tampered → `SignatureInvalid`.
- [x] E2E: CLI sign→verify→tamper→verify-fails (`fidius-cli/tests/wasm_pack.rs`); host signed/unsigned/tampered `load_wasm` (`fidius-host/tests/wasm_executor.rs`); inspect output asserted.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Extend `fidius-cli` `inspect` to render `[wasm]` fields (mirror the Python inspect fields from FIDIUS-T-0087); confirm `PackageRuntime::Wasm` flows through inspect/validation. Signing should need no changes (digest-based) — add wasm coverage to the signing tests. Verify the `LoadPolicy` path treats wasm packages like the others.

### Dependencies
[[FIDIUS-T-0107]] (a packed wasm `.fid` to sign/inspect). Blocks [[FIDIUS-T-0109]].

### Risk Considerations
`inspect` must not assume cdylib/python-only fields — handle wasm distinctly and fall back gracefully. Confirm the signing digest genuinely makes no artifact-type assumptions. Capability allow-list should be surfaced prominently in `inspect` since it's the security-relevant bit a deployer reviews.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.**
- Signing was already artifact-agnostic (`package_digest` over all non-`.sig` files) — no change needed for sign/verify; added wasm coverage.
- `signing::verify_package_signature(dir, keys)` (new) — package.sig over `package_digest`. `load_wasm` + `load_python` enforce it under `require_signature` (parity with cdylib `load()`).
- `package inspect` renders a `WASM:` block (component, precompiled/`.cwasm` or "none — JIT", capabilities prominently).
- Tests: host (`--features wasm`) signed/unsigned/tampered `load_wasm`; CLI `inspect_renders_wasm_fields` + `sign_verify_and_tamper_wasm_package`.
- Verified: wasm suite 18 ok; CLI wasm_pack 4 ok; python suite 10 ok (load_python change safe); native `cargo test --workspace` 41 ok; `angreal lint` green.