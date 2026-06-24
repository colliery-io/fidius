---
id: st-4-fidius-test-composition
level: task
title: "ST.4 — fidius-test composition harness: stream_of / collect / pump (async, unsupported)"
short_code: "FIDIUS-T-0127"
created_at: 2026-06-18T18:14:26.675587+00:00
updated_at: 2026-06-19T01:37:23.107480+00:00
parent: FIDIUS-I-0026
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0026
---

# ST.4 — fidius-test composition harness: stream_of / collect / pump (async, unsupported)

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[FIDIUS-I-0026]] · Phase 1 · **unsupported** test-tier per [[FIDIUS-A-0004]]. Depends on [[FIDIUS-T-0124]].

## Objective **[REQUIRED]**

Ship the minimal composition harness in `fidius-test` that makes streaming plugins trivial to test — `stream_of`, `collect`, `pump` — explicitly **not** a semver-committed API. It is the reference `|` for pipes-of-plugins: correct, crude, copyable, and disclaimed.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] `stream_of(items: Vec<Value>) -> ChunkStream` — an in-memory source (test-only by nature) for exercising one streaming plugin without a real producer.
- [ ] `collect(s: ChunkStream) -> Result<Vec<Value>, CallError>` — drain a stream to a vec to assert on; surfaces a mid-stream `ERROR` frame as `Err`.
- [ ] `pump(out: ChunkStream, into: &impl StreamSink) -> Result<…>` — the reference pull-loop wiring a producer to a consumer; **correct** backpressure (pull-paced) and **correct** error propagation (stop on error, run teardown).
- [ ] All three are async and live behind `fidius-test`; module-level docs state plainly: *for tests, not stability-committed; write your own pump in production* (cite FIDIUS-A-0004).
- [ ] They compose: `collect(transform.process(stream_of(rows)))` works as a single-plugin unit-test idiom.
- [ ] Harness is dogfooded by ST.5's tests; no production crate (`fidius-host`/`fidius`) depends on it.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
- Keep it tiny and idiomatic — a contributor should read `pump` and understand the whole composition model in 20 lines. Crude is fine; misleading is not (a wrong backpressure/error pattern here gets copied into production).
- `StreamSink` is whatever a streaming-consuming plugin presents (Phase 1: the Python destination side); keep the trait minimal so it's obvious.

### Dependencies
- Depends on [[FIDIUS-T-0124]] (`ChunkStream`). Used by [[FIDIUS-T-0128]]. Independent of the macro (ST.2) for `stream_of`/`collect`.

### Risk Considerations
- The disclaimer must be load-bearing: resist feature-creeping this into an orchestrator. If it starts growing scheduling/retry, that's a signal it's drifting across the ADR boundary — stop.

## Status Updates **[REQUIRED]**

### 2026-06-18 — ST.4 complete ✅
- New **`fidius_test::stream`** module behind a non-default `streaming` feature (enables `fidius-host/streaming` + `futures`/`async-trait`):
  - `stream_of(Vec<Value>) -> ChunkStream` — in-memory source (test-only by nature).
  - `collect(ChunkStream) -> Result<Vec<Value>, CallError>` — drain; stops at and surfaces the first error.
  - `pump(ChunkStream, &impl StreamSink) -> Result<(), CallError>` — the reference pull-loop; pull-paced (one item awaited at a time = real backpressure), stops on first producer-or-sink error.
  - `StreamSink` trait (async, minimal) + `CollectSink` test double.
- Module docs state plainly: **not semver-stable; write your own pump in production**, citing FIDIUS-A-0004. The disclaimer is load-bearing, not decoration.
- **5 async unit tests**: stream_of→collect round-trip, collect-surfaces-error, pump-delivers-all, pump-stops-on-error (sink saw only pre-error items), and the `collect(transform(stream_of(..)))` single-plugin idiom.
- **Verified**: `cargo test -p fidius-test --features streaming stream::` 5/5; default-feature build of `fidius-test` unaffected (module cfg-gated; smoke test still builds).
- Note: `pump`'s `StreamSink` is generic/in-memory here (no Python dependency), so ST.4 closed fully independently of ST.3 — only `stream_of`/`collect`/`ChunkStream` from ST.1 were needed.