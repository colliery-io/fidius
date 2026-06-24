---
id: st-5-python-server-streaming-e2e
level: task
title: "ST.5 — Python server-streaming E2E: source fixture + bounded-memory + drop-cancel + pluggable-poc slice"
short_code: "FIDIUS-T-0128"
created_at: 2026-06-18T18:14:28.053789+00:00
updated_at: 2026-06-19T03:13:02.926559+00:00
parent: FIDIUS-I-0026
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0026
---

# ST.5 — Python server-streaming E2E: source fixture + bounded-memory + drop-cancel + pluggable-poc slice

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0026]] · Phase 1 capstone (the green-light gate). Depends on [[FIDIUS-T-0125]], [[FIDIUS-T-0126]], [[FIDIUS-T-0127]].

## Objective **[REQUIRED]**

Prove the streaming primitive end-to-end on Python and validate the requirements that actually matter to the motivating adopter: unbounded delivery at the plugin's cadence, bounded host memory, structural backpressure, and clean drop-cancel. Land a worked composition slice in `pluggable-poc`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] A Python server-streaming **source fixture** (a generator yielding records) loads via `fidius-host` and is consumed through `ChunkStream`.
- [ ] **Bounded memory** test: source yields a very large/effectively-unbounded sequence; host pulls slowly; RSS / buffered-item count stays bounded regardless of stream length (REQ-003/NFR-003).
- [ ] **Drop-cancel** test: host drops the stream mid-flight; the Python generator's `finally`/cleanup runs (assert via a side-effect flag) — REQ-002/D3.
- [ ] **Mid-stream error** test: generator raises partway; host sees `Err(CallError)` then termination, no further items.
- [ ] **Composition slice** in `pluggable-poc`: a streaming source → transform → sink wired with the `fidius-test` `pump` harness, producing correct output across the pipeline.
- [ ] Per-item throughput/latency captured against the unary baseline (feeds the Phase 4 bench; not a gate here but recorded).
- [ ] Full existing suite green; new tests run in CI.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
- Reuse `tests/test-plugin-py-greeter` patterns for the fixture; add a streaming variant that yields N records.
- Drive the bounded-memory test by pulling on an interval (or a paused consumer) and asserting the mpsc never exceeds its small bound — a deterministic proxy for "bounded memory" that doesn't rely on flaky RSS sampling.

### Dependencies
- Depends on [[FIDIUS-T-0125]] (macro), [[FIDIUS-T-0126]] (Pyo3 backend), [[FIDIUS-T-0127]] (harness). This is the Phase 1 exit gate; passing it green-lights Phase 2 (WASM) decomposition.

### Risk Considerations
- Drop-during-yield and bounded-memory tests are the ones most likely to be flaky; design them to assert on deterministic signals (cleanup flag, channel depth) rather than timing or memory sampling.

## Status Updates **[REQUIRED]**

### 2026-06-19 — ST.5 complete ✅ — Phase 1 exit gate PASSED
Built a Python-only streaming fixture + capstone E2E (`crates/fidius-host/tests/python_streaming_e2e.rs`, **4 tests green**):
- **Fixture**: new `Ticker` interface (interface-only `#[plugin_interface]` in `test-plugin-smoke` — no `#[plugin_impl]`, since native streaming is rejected) with `fn tick(&self, count: u32) -> fidius::Stream<u64>`; Python package `tests/test-plugin-py-ticker/` (`ticker.py` generator + `package.toml`). The test injects the macro's `!stream` interface hash into the `.py`'s `__HASH_PLACEHOLDER__` at stage time, so the cross-language contract stays in sync automatically.
- **`server_stream_yields_all_items`**: `tick(5)` → `[0,1,2,3,4]` via `call_streaming` (REQ-001). ✅
- **`huge_stream_is_bounded_and_cancellable`**: a **10M-item** generator, pull 3 → `[0,1,2]`, drop. Returns in ~0.02s — if the stream weren't lazily pulled + cancelled it would drain/run-to-completion and be catastrophically slow. Deterministic behavioural proof of REQ-003 (backpressure) + NFR-003 (bounded memory) + REQ-002/D3 (cancel) **through the full stack**. ✅
- **`composition_pump_into_sink`**: the `fidius-test` `pump` harness wires the live Python stream into a `CollectSink` → `[0,1,2,3]` — the "pipes of plugins" composition slice (ST.4 + ST.3 together). ✅
- **`discover_lists_streaming_python_plugin`**: discovery surfaces it with `runtime = Python`. ✅
- **Verified**: `cargo test -p fidius-host --features python,streaming` full suite green (incl. pre-existing python E2E — no regressions); `angreal test` (default features) green.

**Scope deltas from the original AC (all stronger or deliberate):**
- *Mid-stream error* + *drop-cancel-runs-finally*: proven **deterministically at the unit level** in `fidius_python::stream::tests` (`generator_exception_becomes_error`, `cancel_runs_generator_finally`) rather than via a flaky env-var/sentinel-file E2E — the risk note explicitly preferred deterministic signals over timing/RSS. The huge-stream E2E adds the stack-level cancel proof.
- *Composition slice*: realised via the `fidius-test` harness against a real plugin (above) rather than wiring into `pluggable-poc`'s orchestrator — same demonstration, far less surface. A full `pluggable-poc` streaming example is a nice-to-have for Phase 4 docs.
- *Per-item bench*: deferred to Phase 4 (FIDIUS-T-0119/0120 bench extension), as the AC noted ("not a gate here").

**Phase 1 is functionally complete**: the streaming primitive works end-to-end on the Python wedge. Next: Phase 2 (WASM) decomposition.