---
id: p2-4-capability-policy-deny-fs-by
level: task
title: "P2.4 — Capability policy: deny-FS-by-default Linker + manifest allow-list → typed WASI imports"
short_code: "FIDIUS-T-0104"
created_at: 2026-06-17T04:33:18.083415+00:00
updated_at: 2026-06-17T05:46:27.325728+00:00
parent: FIDIUS-I-0021
blocked_by: [FIDIUS-T-0103]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0021
---

# P2.4 — Capability policy: deny-FS-by-default Linker + manifest allow-list → typed WASI imports

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0021]]

## Objective **[REQUIRED]**

Implement the sandbox capability policy. **Refined by the T-0102 finding (human-ratified 2026-06-17):** real WIT components built from std import `wasi:cli/io/clocks/filesystem` even when unused, so an *empty* `Linker` can't instantiate them. The sandbox is therefore **WASI wired into the `Linker` but with a zero-grant `WasiCtx`** — no filesystem preopens, no env, no inherited stdio, no sockets. Imports satisfied; capabilities denied (FS-denied-by-default holds). A manifest-declared allow-list then opens *specific* capabilities (the `WasiCtx`/host-import wiring is the policy point). The locked-down `WasiCtx` baseline lands with the executor in T-0102; this task adds the allow-list on top.

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `[wasm].capabilities` allow-list (already on `WasmPackageMeta` from T-0103); a known set is defined (`env`, `args`, `stdout`, `stderr`, `stdin`, `network`/`sockets`, `clocks`, `random`) and unknown names are rejected at load (`validate_capabilities` → `LoadError::WasmLoad("unknown wasm capability …")`).
- [x] Default `WasiCtx` grants **nothing** — `WasiCtxBuilder::new().build()` with no preopens/env/stdio/sockets; the compute greeter component instantiates and runs under it (WASI in the `Linker`, capabilities denied).
- [x] Each capability maps to a specific `WasiCtxBuilder` grant in one place (`build_wasi_ctx`): `env`→`inherit_env`, `args`→`inherit_args`, `stdout`/`stderr`/`stdin`→`inherit_*`, `network`/`sockets`→`inherit_network`+`allow_ip_name_lookup`; `clocks`/`random` are always-available no-ops. **Filesystem is never grantable** (`filesystem` is not a known capability → rejected; no preopens ever).
- [x] Enforcement is **runtime** via the `WasiCtx` (not link-time — `add_to_linker_sync` adds all of WASI p2; grants live in the ctx). An ungranted op is denied at runtime; a *typo'd* capability fails **at load** with a clear error. (The original "fails to instantiate/link" framing corrected per the T-0102 finding.)
- [x] Tests (`tests/wasm_executor.rs`): `env_capability_denied_by_default` (env var invisible without grant → false), `env_capability_granted_via_allowlist` (visible with `["env"]` → true), `unknown_capability_rejected_at_load`. 11/11 pass.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
Use `wasmtime-wasi` preview2 (`WasiCtxBuilder`) and add only the declared imports to the `Linker`. Deny filesystem by supplying no preopens. Map manifest capability strings → import wiring in one place (the policy point). Surface link-time failures as a clear `CallError`/`LoadError`.

### Dependencies
[[FIDIUS-T-0103]] (manifest `[wasm]` section + executor + loader). Blocks [[FIDIUS-T-0105]].

### Risk Considerations
The WASI preview2 surface is large — scope to the connector-relevant set (clocks, random, sockets) and explicitly exclude filesystem. Socket egress ideally allow-listed by host:port; if full egress control is heavy, document granting `sockets` coarsely in v1 as a known limitation with per-host:port filtering as a follow-on. Keep the default genuinely empty so "forgot to grant" fails closed.

## Status Updates **[REQUIRED]**

**2026-06-17 — COMPLETE.**
- `executor/wasm.rs`: `KNOWN_CAPABILITIES`, `validate_capabilities` (load-time reject of unknown names), `build_wasi_ctx` (deny-all baseline + per-capability `WasiCtxBuilder` grants; FS never). `WasmComponentExecutor` carries the `capabilities` list and builds a fresh `WasiCtx` per call.
- Constructors + `load_wasm` thread `[wasm].capabilities` through.
- Greeter fixture gained `probe-env` (returns whether `FIDIUS_TEST_CAP` is visible) to give a clean granted-vs-denied behavioural test; guest rebuilt; descriptor/method_count updated (3→4).
- Tests: env denied-by-default, env granted-via-allowlist, unknown-capability-rejected → 11/11 in `tests/wasm_executor.rs`. clippy clean.

**Model note:** enforcement is runtime via the `WasiCtx` (all of WASI p2 is in the `Linker`; grants live in the ctx), not link-time. Filesystem is not a grantable capability. Per-host:port socket filtering is a documented follow-on (v1 grants `network` coarsely).