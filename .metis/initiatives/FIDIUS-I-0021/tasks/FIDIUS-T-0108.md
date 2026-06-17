---
id: p3-3-signing-inspect-for-wasm
level: task
title: "P3.3 — Signing + inspect for .wasm components (Ed25519 artifact-agnostic; inspect understands wasm runtime)"
short_code: "FIDIUS-T-0108"
created_at: 2026-06-17T09:50:14.312918+00:00
updated_at: 2026-06-17T09:50:14.312918+00:00
parent: FIDIUS-I-0021
blocked_by: ["FIDIUS-T-0107"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P3.3 — Signing + inspect for .wasm components (Ed25519 artifact-agnostic; inspect understands wasm runtime)

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Ensure Ed25519 signing covers `.wasm` component packages (artifact-agnostic — nearly free) and `fidius inspect` understands the wasm runtime, so a wasm `.fid` signs/verifies/inspects like cdylib and Python.

## Acceptance Criteria **[REQUIRED]**

- [ ] `fidius sign` / `fidius verify` work on a wasm `.fid` (signing is over the package digest — confirm no `.so`/`.py`-specific assumptions; add tests). Tampering a wasm package fails verification.
- [ ] `fidius inspect` on a wasm package reports `runtime = wasm`, interface + version, the component file, the `[wasm].capabilities` allow-list, the interface hash, and the `.cwasm` if present.
- [ ] Load policy (signature required vs lenient) is enforced for wasm packages identically to cdylib/python (`LoadPolicy`).
- [ ] E2E: sign a wasm `.fid`, verify OK, tamper → verify fails; `inspect` output is correct for a wasm package.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Extend `fidius-cli` `inspect` to render `[wasm]` fields (mirror the Python inspect fields from FIDIUS-T-0087); confirm `PackageRuntime::Wasm` flows through inspect/validation. Signing should need no changes (digest-based) — add wasm coverage to the signing tests. Verify the `LoadPolicy` path treats wasm packages like the others.

### Dependencies
[[FIDIUS-T-0107]] (a packed wasm `.fid` to sign/inspect). Blocks [[FIDIUS-T-0109]].

### Risk Considerations
`inspect` must not assume cdylib/python-only fields — handle wasm distinctly and fall back gracefully. Confirm the signing digest genuinely makes no artifact-type assumptions. Capability allow-list should be surfaced prominently in `inspect` since it's the security-relevant bit a deployer reviews.

## Status Updates **[REQUIRED]**

Not started — Phase 3 of FIDIUS-I-0021.