---
id: p3-4-docs-write-your-first-wasm
level: task
title: "P3.4 — Docs: write-your-first-WASM-plugin (Rust) + non-Rust walkthrough + capability guide"
short_code: "FIDIUS-T-0109"
created_at: 2026-06-17T09:50:15.637150+00:00
updated_at: 2026-06-17T09:50:15.637150+00:00
parent: FIDIUS-I-0021
blocked_by: ["FIDIUS-T-0106", "FIDIUS-T-0107", "FIDIUS-T-0108"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P3.4 — Docs: write-your-first-WASM-plugin (Rust) + non-Rust walkthrough + capability guide

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Document the WASM plugin author experience end-to-end: a Rust "write your first WASM fidius plugin" tutorial, a non-Rust (componentize-py) walkthrough, and a capability-declaration/security guide — closing out the initiative's user-facing story.

## Acceptance Criteria **[REQUIRED]**

- [ ] A tutorial/how-to: author a Rust WASM plugin with the macro → build the component → `fidius pack` → `fidius sign` → load via `PluginHost::load_wasm`.
- [ ] A non-Rust walkthrough (componentize-py, mirroring `tests/wasm-fixtures/greeter-py/`) implementing the same interface — the concrete polyglot story.
- [ ] A capability guide: the `[wasm].capabilities` allow-list, deny-by-default, filesystem-never, granting network/env, and the security model (WASI present, zero grants — from [[FIDIUS-T-0104]]).
- [ ] Cross-links to the WASM Component ABI explanation (`docs/explanation/wasm-component-abi.md`, [[FIDIUS-T-0101]]) and the toolchain how-to (`docs/how-to/wasm-component-toolchain.md`, [[FIDIUS-T-0094]]); docs index / nav updated.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Diátaxis: a tutorial under `docs/tutorials/` + how-tos under `docs/how-to/`. Reuse the greeter Rust + Python fixtures as worked examples. Reference ADR [[FIDIUS-A-0003]] for the "why Path B" rationale and the version/compat note (0.3.0).

### Dependencies
[[FIDIUS-T-0106]], [[FIDIUS-T-0107]], [[FIDIUS-T-0108]] — documents the full authored flow, so it lands last. Final Phase 3 task and the initiative's exit deliverable.

### Risk Considerations
Docs must match the real author flow, which depends on the T-0106 ergonomics decision — write after that flow is settled. Keep examples runnable (tie them to the committed fixtures) so they don't rot.

## Status Updates **[REQUIRED]**

Not started — Phase 3 of FIDIUS-I-0021.