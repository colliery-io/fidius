---
id: egress-response-hook-on-response
level: initiative
title: "Egress response hook — on_response observation + bounded auth-retry for wasi:http"
short_code: "FIDIUS-I-0035"
created_at: 2026-08-31T18:18:35.746008+00:00
updated_at: 2026-08-31T18:27:29.652089+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: S
initiative_id: egress-response-hook-on-response
---

# Egress response hook — on_response observation + bounded auth-retry for wasi:http Initiative

## Context **[REQUIRED]**

Feature request from weir (WEIR-T-0180, 2026-08-30; full FR in `FR-egress-response-hook.md` at repo root). Relates to FIDIUS-I-0027 (egress policy seam) and FIDIUS-I-0034 (hostname TCP egress).

`EgressPolicy::authorize` is a **request-only** seam: it can inspect and decorate every outbound `wasi:http` request (weir uses it for host-side credential injection — API keys, OAuth bearers — so secrets never enter the guest). But the policy never sees **responses**: `EgressHooks::send_request` (`crates/fidius-host/src/executor/wasm.rs:200`) dispatches via `default_send_request` and hands the `HostFutureIncomingResponse` straight back to the guest.

The gap: a host-injected credential can expire mid-run. The target API answers 401, the guest (which knows nothing about credentials, by design) fails the request, and the whole plugin call fails. The embedder's credential provider — the one component that could mint a fresh token and retry — never learns the request died. weir's current workarounds (proactive TTL re-mint; run-level retry from checkpoint) are both second-best: one prevents rather than heals, the other throws away a whole attempt per stale token.

What we want is the standard HTTP-client pattern: *on 401, refresh the credential and replay that one request, once* — owned by the embedder's `EgressPolicy`, right where the credential lives.

**Feasibility verified against wasmtime-wasi-http 46**: `default_send_request` is a thin wrapper that spawns the public async `default_send_request_handler` and wraps the join handle in `HostFutureIncomingResponse::pending(...)`. We can spawn our own task (via `wasmtime_wasi::runtime::spawn`) that dispatches, observes the response head, calls the policy, and on retry re-authorizes and dispatches again. Dropping the first `IncomingResponse` aborts its worker (`AbortOnDropJoinHandle`), so discarding the 401 is clean. No wasmtime fork needed.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Optional response observation on `EgressPolicy`: `on_response(request_parts, status, headers) -> ResponseDirective`, seeing status + headers (never the body) before the guest does.
- A bounded retry directive (`RetryOnce`): discard the response, re-run `authorize` on a fresh pre-authorize clone of the request parts (so the policy can inject a fresh credential), dispatch again, forward the second response. At most one retry per original guest request, enforced by fidius.
- Invisible to the guest on the happy path: exactly one response per request, same as today.
- Byte-identical behavior for existing embedders: the whole observation/tee path is gated on an opt-in (`observes_responses() -> bool`, default `false`), and `on_response` has a default `Forward` impl.
- Two-key rule unchanged: only requests that already passed `authorize` are observed; the retry re-passes `authorize`.
- Body replay via **tee** (not buffer-then-dispatch): the request streams to the wire exactly as today while frames are copied into a side buffer up to a 64 KiB cap. Retry is eligible only when the body was fully captured and ≤ cap at decision time; otherwise the response forwards untouched.

**Non-Goals:**
- Response body inspection or rewriting (head-only observation).
- Retrying TCP-tier (`wasi:sockets`) traffic — that tier is an opaque byte stream; this is the `wasi:http` seam only.
- Backoff/retry *policies* (that stays the embedder's runtime's job — weir has worker-level retries); this is strictly the single-shot auth-refresh pattern.
- Buffer-then-dispatch body handling (rejected — see Alternatives).

## Detailed Design **[REQUIRED]**

### API surface (`crates/fidius-host/src/executor/wasm.rs`)

```rust
/// What the policy wants done with a response it has observed.
pub enum ResponseDirective {
    /// Hand the response to the guest unchanged (the default).
    Forward,
    /// Discard this response; re-run `authorize` on a fresh clone of the
    /// ORIGINAL (pre-authorize) parts and dispatch again. At most once per
    /// original request; ignored when the body was not replayable.
    RetryOnce,
}

pub trait EgressPolicy: Send + Sync + 'static {
    // ... existing authorize / authorize_tcp / authorize_tcp_target / authorize_dns / authorize_udp ...

    /// Opt-in gate. Default `false` ⇒ the dispatch path is byte-identical to
    /// today (no tee, no observation, zero overhead).
    fn observes_responses(&self) -> bool { false }

    /// Observe the response HEAD for a request this policy authorized, before
    /// the guest sees it. `request` is the as-dispatched (post-authorize) parts.
    /// `retry_available` is false on the post-retry observation (and when the
    /// body was not replayable); a `RetryOnce` returned then is ignored.
    fn on_response(
        &self,
        request: &http::request::Parts,
        status: http::StatusCode,
        headers: &http::HeaderMap,
        retry_available: bool,
    ) -> ResponseDirective { ResponseDirective::Forward }
}
```

`ResponseDirective` is re-exported from the facade alongside the existing egress types (`crates/fidius/src/lib.rs:183`).

### Dispatch mechanics (`EgressHooks::send_request`)

- `observes_responses() == false` → exactly today's path: `authorize`, then `default_send_request`. Byte-identical.
- `observes_responses() == true`:
  1. Clone the pre-authorize `Parts` (the retry's `authorize` input — so the policy re-stamps a clean request, not one carrying its own stale header). Run `authorize` as today.
  2. Wrap the outgoing `HyperOutgoingBody` in a **tee body**: frames pass through to the wire unchanged while being copied into a side buffer, up to a 64 KiB cap. Track terminal state: fully-captured / overflowed / trailers-present (trailers ⇒ not replayable).
  3. Spawn our own task via `wasmtime_wasi::runtime::spawn`, returning `HostFutureIncomingResponse::pending(handle)` — mirroring what `default_send_request` does. Inside the task: `default_send_request_handler(request, config).await`, then on success call `on_response` with the response head and `retry_available = (body fully captured ∧ ≤ cap ∧ no trailers)`.
  4. `Forward` (or retry unavailable) → return the response as-is.
  5. `RetryOnce` (and available) → drop the first `IncomingResponse` (worker aborts via `AbortOnDropJoinHandle`), clone the saved pre-authorize parts, re-run `authorize`; on deny the guest gets `ErrorCode::HttpRequestDenied` (the policy consumed the 401 and then refused — documented, tested). On allow, rebuild the request with a `Full<Bytes>` replay body from the captured buffer, `default_send_request_handler` again, call `on_response` with `retry_available = false` (directive ignored — observability only), and return that second response unconditionally.
- Transport errors (`ErrorCode`) are returned to the guest as today — `on_response` fires only for actual responses.

### Semantics & guardrails

- **Bounded**: one retry per original guest request, enforced by fidius; no policy can loop.
- **Head-only**: the policy never sees a response body; no inbound buffering.
- **Two-key unchanged**: no new reach — observation only for requests that passed `authorize`; the retry re-passes `authorize`.
- **Timeouts are per attempt**: `connect_timeout`/`first_byte_timeout` from `OutgoingRequestConfig` apply to each dispatch, so a retried request can take up to ~2× — documented on `RetryOnce`.
- **Why tee (not buffer-then-dispatch)**: buffering would delay every dispatch until the guest finishes writing its body — a timing change for all requests, uncovered by any configured timeout, and a deadlock for a guest that interleaves body-writing with response-polling. Tee keeps wire behavior identical and degrades gracefully: server 401s before the body finishes ⇒ capture incomplete ⇒ forward.
- **Eligibility at decision time**, not pre-dispatch: request body size is not reliably knowable up front (a bodiless GET has no Content-Length — and small GET/POSTs are weir's primary case). An empty body is trivially "fully captured".

## Alternatives Considered **[REQUIRED]**

- **Buffer-then-dispatch** (collect body, then send): rejected — changes timing for every request even under a Forward-only policy, and can deadlock a guest that streams its body while polling the response. Tee is wire-identical.
- **Pre-dispatch size check (Content-Length ≤ cap)**: rejected — bodiless GETs carry no Content-Length and are the main retry case; capture-state-at-decision-time covers them naturally.
- **No opt-in gate (always tee)**: rejected — cost is only a small memcpy, but "zero change unless you opt in" is the right contract for a security-sensitive seam and makes the fail-open default literal.
- **Generic retry/backoff policy in fidius**: rejected (FR non-goal) — retry *policy* belongs to the embedder's runtime; fidius provides only the single-shot auth-refresh seam.
- **Run-level retry / proactive TTL re-mint (status quo in weir)**: the motivating second-bests; heavyweight or prediction-dependent respectively.

## Testing Strategy

- **Unit** (in `wasm.rs` test modules, matching existing egress unit-test style): tee-body capture states (empty, small, overflow, trailers), directive plumbing, pre-authorize clone semantics.
- **E2E** (new test in the spirit of `crates/fidius-host/tests/hostname_egress_e2e.rs`, local hyper server):
  1. 401-then-200: policy swaps a header on `RetryOnce`; guest's single request succeeds; server saw exactly two requests (stale then fresh credential).
  2. Streaming/oversized body + `RetryOnce` → 401 forwards untouched (documented constraint).
  3. Policy without overrides → byte-identical to today (existing e2e suites keep passing unchanged).
  4. Retry's `authorize` denies → guest gets `HttpRequestDenied`.
  5. Policy returns `RetryOnce` on the second observation → ignored, response forwards.

## Status Updates

**2026-09-03 — ALL TASKS COMPLETE; initiative left in active for user review.**

T-0198/0199/0200 all completed; full `angreal` gates green (check/lint/build/test — 90 suite-result blocks, 0 failures); versions bumped to 0.5.9 (crates + lockfile); docs updated (`wasm-capabilities.md` response-hook section, CHANGELOG 0.5.9 entry, plissken API docs regenerated). Work is **uncommitted on `main`**; suggested commit per release convention: `feat(egress): response hook + bounded auth-retry + bump to 0.5.9`.

Two implementation findings hardened the design beyond the sketch above (full detail in the task docs):

1. **hyper never polls a body whose `is_end_stream()` is true** — end-of-stream must be recognized at wrap time, not only in `poll_frame` (T-0199).
2. **Real wasi-http guest bodies are channel-backed** — `is_end_stream()` is never true and end-of-stream is only observable by polling, so a fast 401 races hyper's body drain and would leave bodiless GETs (the motivating case!) silently non-retryable. `TeeBody::wrap` therefore **primes** the tee: it polls the inner body with a noop waker until Pending/end, stashing frames for hyper. Any body finished before dispatch is now deterministically replayable (≤ 64 KiB); still-streaming bodies remain timing-dependent as documented (T-0200).

One deliberate wire nuance vs the "wire-identical tee" above: under an *observing* policy, a primed-complete empty body advertises `is_end_stream()`, so a bodiless GET goes out unframed instead of as an empty chunked body. Non-observing policies stay literally byte-identical.

## Implementation Plan **[REQUIRED]**

Three tasks, sequential:

1. [[FIDIUS-T-0198]] — **Policy surface + tee/replay body**: `ResponseDirective`, `observes_responses`, `on_response` (default impls, rustdoc with the full contract), the tee body wrapper + capture-state tracking, facade re-export. Unit tests. No behavior change yet.
2. [[FIDIUS-T-0199]] — **Retry dispatch in `EgressHooks::send_request`** (blocked by T-0198): the observing dispatch path (own spawned task around `default_send_request_handler`), pre-authorize parts clone, single-retry enforcement, deny-on-retry mapping, per-attempt timeout documentation.
3. [[FIDIUS-T-0200]] — **E2E suite + docs + release** (blocked by T-0199): the five e2e cases above, doc updates where egress is documented, workspace version bump 0.5.8 → 0.5.9.