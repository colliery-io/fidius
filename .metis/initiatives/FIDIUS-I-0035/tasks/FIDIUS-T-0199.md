---
id: observing-dispatch-path-in
level: task
title: "Observing dispatch path in EgressHooks::send_request — single bounded retry via default_send_request_handler"
short_code: "FIDIUS-T-0199"
created_at: 2026-08-31T18:20:14.825610+00:00
updated_at: 2026-09-01T12:32:42.979189+00:00
parent: FIDIUS-I-0035
blocked_by: [FIDIUS-T-0198]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0035
---

# Observing dispatch path in EgressHooks::send_request — single bounded retry via default_send_request_handler

## Parent Initiative

[[FIDIUS-I-0035]]

## Objective **[REQUIRED]**

Wire the observing dispatch path into `EgressHooks::send_request` (`crates/fidius-host/src/executor/wasm.rs:200`): when the policy opts in via `observes_responses()`, dispatch through our own spawned task around `default_send_request_handler`, call `on_response` on the response head, and honor a single bounded `RetryOnce` (re-authorize a pre-authorize clone, replay the captured body, forward the second response unconditionally). Implements the `## Dispatch mechanics` section of the parent initiative's Detailed Design.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] `observes_responses() == false` → the exact pre-existing path (`authorize` + `default_send_request`); existing egress test suites pass without modification (wasm 8, hostname 13, tcp 6, macro 5 — all green, untouched).
- [x] Opt-in path: pre-authorize `Parts` clone saved before `authorize`; body wrapped in the T-0198 tee; dispatch via own `wasmtime_wasi::runtime::spawn` task returning `HostFutureIncomingResponse::pending(handle)`; inside, `default_send_request_handler(request, config)`.
- [x] `on_response` called with the as-dispatched parts, response status/headers, and `retry_available = (fully captured ∧ ≤ 64 KiB ∧ no trailers)`. Transport errors (`ErrorCode`) skip `on_response` and surface to the guest as today.
- [x] `RetryOnce` with retry available: first `IncomingResponse` dropped (worker aborts), fresh clone of pre-authorize parts re-passed through `authorize`; deny → guest gets `ErrorCode::HttpRequestDenied`; allow → re-dispatch with `Full<Bytes>` replay body, `on_response` called with `retry_available = false` (directive ignored), second response forwarded unconditionally.
- [x] `RetryOnce` without retry available → response forwarded untouched (trailered-body test).
- [x] At most one retry per original guest request — structurally impossible to loop (straight-line fn; bounded test sees exactly two requests).
- [x] `angreal check` + `angreal lint` pass; full `angreal test` deferred to T-0200's release gate (all touched suites re-run green here).

## Status Checklist

- [x] Non-observing path verified untouched
- [x] Observing dispatch task landed
- [x] Retry + deny-on-retry semantics landed
- [x] Tests green

## Implementation Notes

### Technical Approach
- Mirror what `default_send_request` does internally (wasmtime-wasi-http 46, `src/p2/mod.rs:555`): `wasmtime_wasi::runtime::spawn(async move { Ok(handler(...).await) })` wrapped in `HostFutureIncomingResponse::pending`. Our task adds the observe/retry logic around `default_send_request_handler` (public, `src/p2/mod.rs:570`).
- The spawned task needs `Arc<dyn EgressPolicy>` (clone), the pre-authorize `Parts` clone, the tee capture handle, and the `OutgoingRequestConfig` (it's `Clone`-able fields; reuse for the second attempt — timeouts are therefore per attempt, documented in T-0198's rustdoc).
- Dropping the first `IncomingResponse` aborts its worker (`AbortOnDropJoinHandle`) — no explicit cleanup needed for the discarded 401.
- Response head inspection: `IncomingResponse.resp` is a `hyper::Response<HyperIncomingBody>`; status/headers are available without touching the body.

### Dependencies
Blocked by FIDIUS-T-0198 (trait surface + tee body).

### Risk Considerations
- The retry's `authorize` sees the PRE-authorize clone — re-authorizing the as-dispatched parts would hand the policy its own stale injected header (initiative Detailed Design, decision 1). Guard this with a test.
- Keep the non-observing path literally the old code, not a degenerate case of the new path — the byte-identical guarantee is contractual.

## Status Updates **[REQUIRED]**

**2026-09-01 — DESIGN FINDING (caught by the dispatch tests, which hung):** hyper h1 **never polls** a request body whose `is_end_stream()` is `true` — so for a bodiless GET the tee never observed end-of-stream, the capture stayed `Incomplete`, and `retry_available` was `false` for exactly the request shape weir most needs to retry. Fix: `TeeBody::wrap` marks the capture `Complete` at wrap time when the inner body is already at end-of-stream (+ regression test `already_ended_body_is_complete_at_wrap_time`). All 4 dispatch tests + 7 body_tee tests green after the fix.

**2026-09-01 — implemented; tests compiling.**

- `EgressHooks::send_request` now forks: `observes_responses() == false` keeps the literal old path (`authorize` + `default_send_request`); the opt-in path clones pre-authorize `Parts`, authorizes, tees the body, and spawns `dispatch_observed` via `wasmtime_wasi::runtime::spawn` wrapped in `HostFutureIncomingResponse::pending` — mirroring `default_send_request`'s own internals.
- New `dispatch_observed` (free async fn in wasm.rs): first dispatch via `default_send_request_handler`; `on_response(dispatched_parts, status, headers, retry_available = capture.replayable().is_some())`; on `RetryOnce`+replayable → drop first response, re-`authorize` the clean original clone (deny → `ErrorCode::HttpRequestDenied`), re-dispatch with `replay_body(bytes)`, observe with `retry_available=false` (directive ignored), forward unconditionally. Structurally loop-free.
- `copy_config` helper works around `OutgoingRequestConfig` not being `Clone` (all fields Copy). Timeouts are per attempt.
- **Verified before coding:** `http::request::Parts` derives `Clone` (http 1.x); `default_send_request_handler` + `runtime::spawn` are public in wasmtime(-wasi-http) 46.
- body_tee's temporary `#![allow(dead_code)]` removed; `CaptureHandle::state()` is now `#[cfg(test)]` (dispatch decides off `replayable()` alone).
- Tests added (`response_hook_dispatch_tests` in wasm.rs): direct `dispatch_observed` against scripted loopback servers — (1) 401→200 re-stamp: server sees stale-then-fresh, `authorize` asserts it gets a CLEAN clone, observations `[(401,true),(200,false)]`; (2) deny-on-retry → `HttpRequestDenied`, one request; (3) always-RetryOnce bounded to exactly two requests; (4) trailered (non-replayable) body forwards the 401, one request. Clippy clean.