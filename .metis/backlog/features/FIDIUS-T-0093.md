---
id: spike-wasm-executor-feasibility
level: task
title: "Spike — WASM executor feasibility (wasmtime cold-start, WASI caps, #[wire(raw)] over linear memory, PluginExecutor seam)"
short_code: "FIDIUS-T-0093"
created_at: 2026-06-17T02:53:25.733740+00:00
updated_at: 2026-06-17T03:05:42.948235+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Spike — WASM executor feasibility (wasmtime cold-start, WASI caps, #[wire(raw)] over linear memory, PluginExecutor seam)

> Backlog spike (no parent initiative yet). Originated as a feature request relayed from another project. Feeds the wire-boundary decision in [[FIDIUS-A-0003]]. When findings land, an initiative ("Pluggable execution backends + sandboxed WASM executor") gets promoted from this spike.

## Objective **[REQUIRED]**

De-risk a sandboxed WASM (wasmtime) execution backend for fidius by answering the open feasibility questions **before** committing to an initiative or a wire-boundary path. Produce evidence and a recommendation, not production code.

The eventual feature has three phases (recorded here for context, **not** in scope for the spike):

1. **Gating — `PluginExecutor` trait.** Refactor so a single `fidius-host::PluginHandle` dispatches through a `PluginExecutor` trait; fold the existing cdylib (vtable/FFI) path and the shipped Python (PyO3) path under it. Today these are two *parallel* `PluginHandle` types (`fidius-host/src/handle.rs` and `fidius-python/src/handle.rs`) — this collapses the duplication.
2. **`WasmExecutor` on wasmtime.** Define the WASM-side boundary — the fork captured in [[FIDIUS-A-0003]] (bincode-over-linear-memory vs Component Model + WIT).
3. **Macro + signing.** `#[plugin_impl]` emits the WASM target; Ed25519 signing extends to `.wasm` (artifact-agnostic, nearly free).

This spike informs phases 1 and 2.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [x] Feature - New functionality or enhancement (research spike feeding a feature)

### Priority
- [x] P2 - Medium (nice to have)

> Spike, not delivery. Promote to an initiative once findings land.

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: A WASM backend gives plugin authors a sandboxed, language-agnostic target with a real capability model — stronger isolation than cdylib (shared address space) or in-process PyO3, without the author shipping native artifacts per architecture.
- **Business Value**: Positions fidius as the substrate for untrusted/third-party connectors; one signed `.wasm` runs anywhere wasmtime does.
- **Effort Estimate**: Spike itself S–M. The downstream feature is L–XL (initiative).

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

Each open question gets a written finding (with measurements where applicable):

- [x] **Raw-wire mapping.** **YES — maps directly.** `(ptr,len)` round trip via guest alloc/free is byte-exact 16 B→1 MiB at ~2 GiB/s. Structurally identical to existing `call_method_raw`; no change to the raw *contract*, only a backend marshalling shim. Caveat: double-copy (sandbox isolation), not zero-copy.
- [x] **WASI capabilities.** **Zero by default.** Compute guest has *no imports*; ran through an **empty `Linker`** — proof a guest gets only what it's granted. FS denied by default (the win over cdylib/PyO3); sockets/clocks/random are opt-in `wasi:*` imports. The host `Linker` *is* the capability boundary — no Component Model required.
- [x] **Cold-start cost.** JIT compile ~6.6 ms (once/process); **AOT `.cwasm` deserialize ~83 µs**; **fresh Store+Instance ~13.8 µs/call**; warm call 0.12 µs + copy. Fresh-instance-per-call viable (≈ PyO3 per-call cost). Recommend precompile-to-`.cwasm` at pack time + cached `InstancePre`.
- [x] **Wire-boundary recommendation.** **Path A (bincode over linear memory) for v1** — written into [[FIDIUS-A-0003]]. Reuses existing raw boundary, no new toolchain (cargo-component/wasm-tools absent), capability model already achievable. Path B (Component Model + WIT) deferred as additive follow-on.
- [x] **`PluginExecutor` seam sketch.** Clean. Trait is **raw-bytes-only** (`call_raw`/`method_count`/`info`); bincode stays in `PluginHandle` above it. cdylib + PyO3 + WASM each become an impl. Cost: unify `PythonCallError`→`CallError`; add `Python`/`Wasm` to existing `PluginRuntimeKind`. No caller-visible API change.
- [x] **Go/no-go + initiative shape.** **GO.** Promote to initiative; land Phase 1 (trait refactor) independently first; Phase 2 = Path A `WasmExecutor`; Phase 3 = macro wasm target + `.wasm` signing. Path B deferred.

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
Throwaway prototype, not in the main workspace (or behind a `wasm` feature gate / scratch crate). A minimal guest exporting one method + an allocator; a host harness on `wasmtime` exercising the `(ptr,len)` round-trip and timing instantiation. The existing `pluggable-poc/` benchmarks native/FFI/PyO3 but **not** WASM — it's a reasonable place to extend with a WASM strategy for apples-to-apples cold-start/throughput numbers.

### Dependencies
- FIDIUS-T-0082 (`#[wire(raw)]`) — landed; the raw path under test.
- FIDIUS-T-0077 (bincode-only wire) — landed; Path A reuses it.
- FIDIUS-I-0020 (fidius-python) — landed; the PyO3 `PluginHandle` is the second backend the `PluginExecutor` trait must absorb.
- Feeds [[FIDIUS-A-0003]] (wire-boundary ADR).

### Risk Considerations
- Component Model / WASI preview2 tooling maturity — if Path B's bindings story is rough, the spike should say so explicitly rather than assume.
- Cold-start could dominate for short-lived per-call instances; pooling/pre-compilation mitigations must be measured, not assumed.
- Scope creep — this is a spike; resist building production plumbing. Output is findings + a recommendation, plus an updated ADR.

## Status Updates **[REQUIRED]**

**2026-06-16 — Spike complete. GO.** Built a throwaway prototype in `wasm-spike/` (untracked, like `pluggable-poc/`): a `wasm32-unknown-unknown` guest exporting `fd_alloc`/`fd_dealloc`/`fd_call_raw(ptr,len)->u64`, and a `wasmtime` 45 host harness. Full writeup + reproduction in `wasm-spike/FINDINGS.md`.

Headline numbers (Apple Silicon, release): JIT compile ~6.6 ms (once/process), AOT `.cwasm` load ~83 µs, fresh instance ~13.8 µs/call, warm call 0.12 µs + copy; raw round trip byte-exact 16 B→1 MiB at ~2 GiB/s; **module imports = none** (empty `Linker` instantiation = full sandbox, FS-denied-by-default).

All six acceptance criteria met (see above). Recommendation: **Path A** for v1, captured in [[FIDIUS-A-0003]]. The `PluginExecutor` trait is raw-bytes-only because both existing handles already do `serialize → raw dispatch → deserialize` — bincode lives in `PluginHandle`, above the trait, so Path A reuses the exact boundary every backend already has.

**Next:** ratify [[FIDIUS-A-0003]] (Path A) and promote a "Pluggable execution backends + sandboxed WASM executor" initiative; land Phase 1 (`PluginExecutor` refactor) first and independently. Both await human sign-off (architectural decision + initiative creation).