---
id: response-hook-e2e-suite-401-then
level: task
title: "Response-hook E2E suite (401-then-200 retry + guardrail cases) + docs + bump to 0.5.9"
short_code: "FIDIUS-T-0200"
created_at: 2026-08-31T18:20:18.055354+00:00
updated_at: 2026-09-03T20:29:31.628327+00:00
parent: FIDIUS-I-0035
blocked_by: [FIDIUS-T-0199]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0035
---

# Response-hook E2E suite (401-then-200 retry + guardrail cases) + docs + bump to 0.5.9

## Parent Initiative

[[FIDIUS-I-0035]]

## Objective **[REQUIRED]**

Prove the response hook end-to-end against a real local server, document it wherever egress is documented, and release: new e2e test file in the spirit of `crates/fidius-host/tests/hostname_egress_e2e.rs`, plus workspace version bump 0.5.8 → 0.5.9 (per-crate `version` fields, matching the 0.5.7/0.5.8 release pattern).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

E2E cases (local hyper server, wasm guest fixture doing egress — model on `hostname_egress_e2e.rs`):

- [x] **Auth-retry happy path**: `auth_retry_is_invisible_to_the_guest` — real guest, 401-then-200, stale-then-fresh heads, exactly two wire requests.
- [x] **Non-replayable body**: `oversized_body_forwards` (dispatch layer — the GET-only fetcher fixture can't send a body; constraint is enforced below the guest boundary) + trailered-body variant.
- [x] **No override**: `policy_without_overrides_is_unaffected` + all pre-existing egress e2e suites pass unmodified (full `angreal test`: 90 result blocks, 0 failures).
- [x] **Deny on retry**: `deny_on_retry_surfaces_as_denied` — guest sees ERROR, one wire request.
- [x] **Double RetryOnce**: `second_retry_directive_is_ignored` — second 401's body reaches the guest, exactly two requests.
- [x] Docs: new `### Response observation + auth-retry` section in `docs/explanation/wasm-capabilities.md` (weir-shaped example + guardrails); trait rustdoc landed in T-0198; plissken API docs regenerated (`body_tee.md` new, `wasm.md` + nav updated). CHANGELOG.md gained the 0.5.9 entry.
- [x] All crate versions bumped 0.5.8 → 0.5.9 (incl. internal path-dep pins + Cargo.lock); `angreal build`, `angreal test`, `angreal lint` (after `cargo fmt`), `angreal check` all pass.

## Status Checklist

- [x] E2E suite green
- [x] Docs updated
- [x] Version bump landed

## Implementation Notes

### Technical Approach
- New test file `crates/fidius-host/tests/response_hook_e2e.rs`; reuse the fixture-building and local-server patterns from `hostname_egress_e2e.rs` / `wasm_egress_e2e.rs`. A request-counting server with scripted 401-then-200 responses covers most cases.
- The docs example should show the motivating weir pattern: `on_response` matching 401 → invalidate cached token → `RetryOnce`; the re-run `authorize` mints a fresh token.
- Release convention: single feature commit including the bump, e.g. `feat(egress): response hook + bounded auth-retry + bump to 0.5.9` (see 3f73561 / 09ac1ab history).

### Dependencies
Blocked by FIDIUS-T-0199 (dispatch path).

### Risk Considerations
The "server saw exactly two requests" assertions are the load-bearing ones — they prove both that the retry happened and that the bound held. Count on the server side, not from policy callbacks.

## Status Updates **[REQUIRED]**

**2026-09-03 — COMPLETE.** Docs (`wasm-capabilities.md` response-hook section, CHANGELOG 0.5.9 entry, plissken API docs regenerated), version bump 0.5.8→0.5.9 across all crates + lockfile, and all four angreal gates green (`check`/`lint`/`build`/`test` — full suite 90 result blocks, 0 failures; `cargo fmt` applied). Work is uncommitted on `main` awaiting user review; suggested commit message per release convention: `feat(egress): response hook + bounded auth-retry + bump to 0.5.9`.

**2026-09-01 — e2e suite green after a second (deeper) body-timing finding.**

- New `crates/fidius-host/tests/response_hook_e2e.rs` (4 tests, real fetcher guest + scripted loopback servers): auth-retry invisible to guest (401→200, stale-then-fresh heads, exactly 2 wire requests, observations `[(401,true),(200,false)]`); second RetryOnce ignored (guest gets 2nd 401's body, exactly 2 requests); deny-on-retry → guest sees ERROR, 1 request; no-override policy unaffected. Oversized/trailered non-replayable cases live at the dispatch layer (`response_hook_dispatch_tests`, now 5 tests incl. oversized) — the fetcher fixture is GET-only and the constraint is enforced below the guest boundary.
- **FINDING (e2e initially failed; wrap-time `is_end_stream` check from T-0199 was insufficient):** real wasi-http guest bodies are channel-backed (`BodyImpl`) — `is_end_stream()` is *never* true and end-of-stream is only observable by polling; a fast 401 beats hyper's body-drain, leaving the capture `Incomplete` at decision time. Reproduced deterministically with a channel-shaped empty body. **Fix: `TeeBody::wrap` now *primes* the tee** — polls the inner body with `Waker::noop()` until Pending/end, stashing frames (replayed to hyper first; capture recorded once). A body finished before dispatch (bodiless GETs, small JSON POSTs — the weir shape) reaches `Complete` synchronously at wrap; still-streaming bodies get Pending immediately and stay timing-dependent as documented. Deliberate wire nuance: a primed-complete empty body now advertises `is_end_stream()`, so a bodiless GET goes out with no framing instead of an empty chunked body (documented on `TeeBody`).
- body_tee tests reworked for priming (7 green: finished-at-wrap ×3 shapes, pass-through, streaming-completes-on-drain, undrained-stays-incomplete, oversized, trailers, replay round-trip).

Remaining: docs section in `wasm-capabilities.md`, version bump 0.5.8→0.5.9, full angreal gates.