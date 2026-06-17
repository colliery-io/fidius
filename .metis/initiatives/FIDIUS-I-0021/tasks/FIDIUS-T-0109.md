---
id: p3-4-docs-write-your-first-wasm
level: task
title: "P3.4 — Docs: write-your-first-WASM-plugin (Rust) + non-Rust walkthrough + capability guide"
short_code: "FIDIUS-T-0109"
created_at: 2026-06-17T09:50:15.637150+00:00
updated_at: 2026-06-17T12:31:37.560469+00:00
parent: FIDIUS-I-0021
blocked_by: [FIDIUS-T-0106, FIDIUS-T-0107, FIDIUS-T-0108]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P3.4 — Docs: write-your-first-WASM-plugin (Rust) + non-Rust walkthrough + capability guide

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Document the WASM plugin author experience end-to-end: a Rust "write your first WASM fidius plugin" tutorial, a non-Rust (componentize-py) walkthrough, and a capability-declaration/security guide — closing out the initiative's user-facing story.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] Tutorial `docs/tutorials/your-first-wasm-plugin.md`: author a Rust WASM plugin with the macros (`crate = "fidius_guest"`) → `cargo build --target wasm32-wasip2` → `fidius package pack` → `sign` → `load_wasm`.
- [x] Non-Rust walkthrough `docs/how-to/wasm-python-plugin.md` (componentize-py, mirroring `tests/wasm-fixtures/greeter-py/`) implementing the same `greeter` WIT — the polyglot story, "no fidius dependency in the guest".
- [x] Capability guide `docs/explanation/wasm-capabilities.md`: the allow-list, deny-by-default, "WASI present / zero grants", filesystem-never, coarse network, fail-closed unknowns, and the deployer trust workflow (inspect + signature).
- [x] Cross-links to the ABI explanation + toolchain how-to throughout; `mkdocs.yml` nav updated with all five WASM docs (the pre-existing ABI + toolchain pages were also missing from nav — now added).

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Diátaxis: a tutorial under `docs/tutorials/` + how-tos under `docs/how-to/`. Reuse the greeter Rust + Python fixtures as worked examples. Reference ADR [[FIDIUS-A-0003]] for the "why Path B" rationale and the version/compat note (0.3.0).

### Dependencies
[[FIDIUS-T-0106]], [[FIDIUS-T-0107]], [[FIDIUS-T-0108]] — documents the full authored flow, so it lands last. Final Phase 3 task and the initiative's exit deliverable.

### Risk Considerations
Docs must match the real author flow, which depends on the T-0106 ergonomics decision — write after that flow is settled. Keep examples runnable (tie them to the committed fixtures) so they don't rot.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.** Three docs written + wired into `mkdocs.yml` nav:
- Tutorial: `tutorials/your-first-wasm-plugin.md` (Rust, full author→load flow).
- How-to: `how-to/wasm-python-plugin.md` (componentize-py polyglot walkthrough).
- Explanation: `explanation/wasm-capabilities.md` (sandbox/security model).
Examples are tied to the committed `greeter` / `greeter-py` / `macro-greeter` fixtures so they don't rot. Also added the pre-existing `wasm-component-abi.md` + `wasm-component-toolchain.md` to the nav (they'd never been linked). All five nav targets verified to exist (mkdocs not installed locally for a strict build).