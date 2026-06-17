---
id: p1-7-phase-1-regression-gate
level: task
title: "P1.7 — Phase 1 regression gate: existing cdylib + Python suites green, no caller-visible API change"
short_code: "FIDIUS-T-0100"
created_at: 2026-06-17T03:24:03.271756+00:00
updated_at: 2026-06-17T04:25:07.438366+00:00
parent: FIDIUS-I-0021
blocked_by: [FIDIUS-T-0097, FIDIUS-T-0098, FIDIUS-T-0099]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P1.7 — Phase 1 regression gate: existing cdylib + Python suites green, no caller-visible API change

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Prove the Phase 1 refactor is behaviour-preserving: the entire existing cdylib + Python test surface passes unchanged and there is no caller-visible API change. This is the gate that lets Phase 1 merge independently of Phase 2.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `angreal test` (full default workspace) green — 36 `test result: ok` sections, 0 failures (fidius-core/host/macro/test/cli/e2e + the new wasm-discovery test). Plus host `--features python` suites (8) and `fidius-python` tests green. cdylib tests unchanged; Python tests changed **only** for the unified call surface (`call_typed_json`→`call_method`, `call_raw`→`call_method_raw`, error-path `match` since `PluginHandle` isn't `Debug`) — no behavioural edits.
- [x] `angreal python-test` green — 8 passed.
- [x] Public-API diff reviewed: cdylib consumer surface unchanged (`PluginHandle::call_method`/`call_method_raw`/`info`/`has_capability`/`method_metadata`/`trait_metadata`, `from_loaded`/`from_descriptor`/`find_in_process_descriptor`, `Client`, `Client::in_process` all identical). Additive: `from_python`, `PluginExecutor`/`ValueExecutor`/`CdylibExecutor`/`Pyo3Executor`, `fidius_core::{Value,to_value,from_value,ValueError}`. Breaking (for the 0.3.0 bump): new pub enum variants (`PluginRuntimeKind::Wasm`, `PackageRuntime::Wasm`, `CallError::{WireModeMismatch,Backend}`) + `load_python` now returns `PluginHandle`.
- [x] Note added to FIDIUS-I-0021 ("Phase 1 — DONE") confirming Phase 1 merges independently of Phase 2.
- [x] Perf: the `I→Value` hop is **moot for cdylib** (its typed path stays bincode-direct; `Value` never touches cdylib). Python adds one small Value↔JSON round-trip on control-plane args only (bulk uses `call_raw`); the 2 MB raw-wire test confirms bulk throughput is unaffected. No benchmark warranted.
- [x] `angreal lint` (fmt + clippy `-D warnings`) exit 0, zero warnings.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Run via the angreal tasks (`test`, `python-test`, `lint`, `check`). Capture any required call-site churn. Compare exported symbols / public docs for the affected crates before vs after.

### Dependencies
Depends on FIDIUS-T-0097, FIDIUS-T-0098, FIDIUS-T-0099. Last task of Phase 1.

### Risk Considerations
If any existing test needs a behavioural (not mechanical) change to pass, that signals the refactor altered semantics — treat as a defect to fix, not a test to edit.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE. Phase 1 gate green.**
- `angreal test`: full default workspace, 36 `test result: ok`, 0 failures.
- `angreal lint`: exit 0, zero warnings (fmt + clippy `-D warnings`). One round of `cargo fmt --all` was needed on the new files.
- `angreal python-test`: 8 passed.
- `cargo test -p fidius-host --features python`: host python suites pass (8). `cargo test -p fidius-python`: pass.
- Public-API diff captured in AC + the initiative's "Phase 1 — DONE" note. cdylib consumer surface unchanged; breaking surface limited to new pub enum variants + `load_python` return type (a 0.3.0-bump concern, documented).

Phase 1 is behaviour-preserving and merges independently of Phase 2. Version intentionally left at 0.2.1 in-tree (bumping to 0.3.0 changes `ABI_VERSION` and would break the dev test plugins mid-stream; do it at release).