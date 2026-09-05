---
id: response-hook-policy-surface
level: task
title: "Response-hook policy surface: ResponseDirective + on_response/observes_responses + tee/replay body (no behavior change)"
short_code: "FIDIUS-T-0198"
created_at: 2026-08-31T18:20:11.170880+00:00
updated_at: 2026-09-01T10:22:14.994736+00:00
parent: FIDIUS-I-0035
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0035
---

# Response-hook policy surface: ResponseDirective + on_response/observes_responses + tee/replay body (no behavior change)

## Parent Initiative

[[FIDIUS-I-0035]]

## Objective **[REQUIRED]**

Add the response-observation API surface to `EgressPolicy` and the tee/replay body machinery, with **zero behavior change** — nothing consumes the new surface until FIDIUS-T-0199 wires the dispatch path. See the parent initiative's Detailed Design for the full contract; this task implements the `## API surface` half plus the tee body.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] `ResponseDirective { Forward, RetryOnce }` is public in `fidius-host` and re-exported from the facade alongside the existing egress types (`crates/fidius/src/lib.rs:183`).
- [ ] `EgressPolicy::observes_responses(&self) -> bool` added, default `false`.
- [ ] `EgressPolicy::on_response(&self, request: &http::request::Parts, status, headers, retry_available: bool) -> ResponseDirective` added, default `Forward`. Rustdoc states the full contract: post-authorize parts, head-only observation, `retry_available` semantics (false on second observation and for non-replayable bodies), per-attempt timeouts under retry, deny-on-retry → `HttpRequestDenied`.
- [ ] Tee body wrapper over `HyperOutgoingBody`: data frames pass through to the inner body unchanged; bytes are copied into a side buffer capped at 64 KiB; capture state is observable by the dispatch task as one of fully-captured / overflowed / trailers-present / incomplete.
- [ ] Unit tests: tee capture of empty body, small body (replayable), body exceeding cap (overflowed), body with trailers (not replayable); default trait impls return `false` / `Forward`.
- [ ] No behavior change: existing test suites pass untouched (`angreal test`, `angreal lint`, `angreal check`).

## Status Checklist

- [ ] `ResponseDirective` + trait methods landed with rustdoc
- [ ] Tee body + capture state landed
- [ ] Facade re-export landed
- [ ] Unit tests green

## Implementation Notes

### Technical Approach
- Trait lives at `crates/fidius-host/src/executor/wasm.rs:107`; add the two methods next to the existing `authorize_*` defaults, matching their rustdoc style (deny/allow defaults documented per method).
- Tee body: implement `http_body::Body` over the inner `HyperOutgoingBody` (`BoxBody<Bytes, ErrorCode>`); on each data frame, extend a shared buffer (e.g. `Arc<Mutex<CaptureState>>`) unless already overflowed; mark terminal state on end-of-stream/trailers/error. Keep it in `wasm.rs` or a small sibling module — match the file's existing organization.
- `http::request::Parts` is `Clone` (extensions clone what's clonable) — the pre-authorize clone in T-0199 relies on this; no work needed here beyond not blocking it.
- Facade re-export: extend the existing `pub use fidius_host::{...}` at `crates/fidius/src/lib.rs:183` with `ResponseDirective`.

### Dependencies
None — first task of FIDIUS-I-0035. FIDIUS-T-0199 builds directly on this.

### Risk Considerations
Signature is API-frozen once released; the `retry_available: bool` parameter is deliberate future-proofing (initiative Detailed Design, decision 2) — don't drop it for "simplicity".

## Status Updates **[REQUIRED]**

**2026-09-01 — COMPLETE.** Full `angreal test` suite passed (all crates + doc-tests, exit 0) — zero behavior change confirmed. All acceptance criteria met.

**2026-09-01 — implemented; awaiting full-suite confirmation.**

- `ResponseDirective` + `observes_responses()`/`on_response(parts, status, headers, retry_available)` (both defaulted) landed in `crates/fidius-host/src/executor/wasm.rs`, full contract in rustdoc.
- Tee/replay machinery landed as new module `crates/fidius-host/src/executor/body_tee.rs`: `TeeBody::wrap(HyperOutgoingBody) -> (HyperOutgoingBody, CaptureHandle)`, `CaptureState` (Incomplete/Complete/Overflowed/Trailers), `CaptureHandle::{state, replayable}`, `replay_body(Bytes)`, `REPLAY_CAP = 64 KiB`. **Implementation finding:** `HyperOutgoingBody` is `UnsyncBoxBody`, so boxing uses `BodyExt::boxed_unsync()`, not `boxed()`.
- Deps: `bytes`/`http-body`/`http-body-util` added to fidius-host as optional deps folded into the `wasm` feature (same majors wasmtime-wasi-http pins — no new tree).
- Re-export chain updated: `executor.rs`, fidius-host `lib.rs:44`, facade `crates/fidius/src/lib.rs:183` now all carry `ResponseDirective`.
- Temporary module-level `#![allow(dead_code)]` on body_tee with a comment — **FIDIUS-T-0199 must remove it** when the dispatch path consumes the tee.
- Tests: 6 body_tee unit tests (empty/small/overflow/trailers/undrained/replay) + `response_hook_defaults_are_opt_out` in `egress_policy_tests`. Full lib: 45 passed. `angreal check` + `angreal lint` clean (the wasm.rs:1082 clippy type-complexity warning is pre-existing from I-0034). Full `angreal test` running.