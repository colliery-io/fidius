---
id: capability-gated-wasi-http-egress
level: initiative
title: "Capability-gated wasi:http egress for sandboxed WASM connectors"
short_code: "FIDIUS-I-0027"
created_at: 2026-06-19T18:34:53.812845+00:00
updated_at: 2026-06-19T18:48:22.186509+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/design"


exit_criteria_met: false
estimated_complexity: M
initiative_id: capability-gated-wasi-http-egress
---

# Capability-gated wasi:http egress for sandboxed WASM connectors Initiative

> Promoted from [[FIDIUS-T-0135]] (weir feature request). Anticipated by [[FIDIUS-A-0004]] (brokered network I/O fenced as an adopter-driven track) and flagged by [[FIDIUS-I-0026]]. Decided with Dylan: **own initiative**, and v1 ships **per-host allow-list + SSRF guard** (not coarse on/off).

## Context **[REQUIRED]**

The WASM backend instantiates every guest with a **deny-all `WasiCtx`** (`build_wasi_ctx` in `crates/fidius-host/src/executor/wasm.rs`): no FS, no env, no stdio, no network unless a `[wasm].capabilities` entry opts in. That isolation is the entire reason the sandboxed/community-connector tier exists.

But there is **no way to grant outbound HTTP**. The adopter (weir) builds manifest-driven REST connectors whose generated WASM `read` must fetch records from an API — and today it **cannot**: live HTTP only works on the trusted in-process **dylib** path; the WASM `read` is a stub. So the differentiated value of the WASM tier ("Airbyte-style connectors without Docker, including untrusted ones") is blocked.

The existing `"network"/"sockets"` capability is **coarse raw `wasi:sockets`** (`inherit_network()`) — no per-host filtering, no credential seam. This initiative is specifically the higher-level **`wasi:http` outgoing-handler** path, which is both the right layer for REST connectors and the natural seam for **host-brokered credential injection** (WEIR-A-0013).

## Goals & Non-Goals **[REQUIRED]**

> **SCOPE REVISED 2026-06-19 (Dylan): mechanism, not policy.** The allow-list, SSRF denylist, DNS-rebinding defense, and credential sourcing are **deployment-specific application policy** — fidius cannot know the right answer and a partial built-in guard would imply a guarantee it can't keep. So fidius ships **only the mechanism**: the `wasi:http` capability + a **required embedder-supplied egress hook**; the embedder (e.g. weir) implements all policy in that hook. Matches FIDIUS-A-0004 (mechanism not policy). Two decisions locked: (1) **egress cannot be enabled without the embedder providing an egress hook** — no hook = fail closed, a two-key model (package declares intent + host grants & polices); (2) fidius ships **no policy code/API**; a reference allow-list+SSRF hook lives in docs/the integration test as a worked example + a security checklist, never as a semver surface.

**Goals:**
- A `PluginHost` can enable `wasi:http` egress for a sandboxed guest **only** by supplying an **egress hook** (the embedder's `WasiHttpHooks::send_request`-backed callback). Fail-closed by default; no hook = no egress.
- fidius wires the hook into wasmtime-wasi-http and calls it before every dispatch (the embedder decides: allow/deny, rewrite, inject headers). fidius itself makes **no** allow-list / SSRF / credential decisions.
- The package still **declares** it wants egress (a capability) so connector vetting can see it; fidius ANDs "package declares" + "host supplies hook".
- Composes with [[FIDIUS-I-0026]]: a REST source becomes a real `read() -> Stream<Record>` once the embedder grants egress.

**Non-Goals:**
- **Any policy in fidius**: no built-in allow-list, no SSRF denylist, no DNS-rebinding defense, no credential store — all are the embedder's hook (illustrated in docs/tests, not shipped as API).
- Inbound HTTP / serving (`wasi:http/incoming-handler`).
- Replacing the coarse `"network"/"sockets"` raw-socket capability (left as-is).
- Connection pooling / perf tuning of the egress path.

## Requirements **[REQUIRED]**

### Functional
- REQ-001: A new capability (`"http"` coarse and/or `"net:<host>"` scoped) is recognized by `validate_capabilities`; unknown/typo'd names fail loud at load (existing behaviour).
- REQ-002: When granted, the guest's `wasi:http/outgoing-handler` imports resolve and a real GET/POST reaches an allow-listed host and returns status/body.
- REQ-003: Without the grant, importing `wasi:http/outgoing-handler` fails closed (instantiation or call error) — no silent allow.
- REQ-004: The host enforces the per-host allow-list and an SSRF denylist on the request authority **before dispatch**; a request to a non-allow-listed or link-local/metadata host is rejected.
- REQ-005: A host-side hook can mutate the outgoing request (inject headers) without the guest's involvement.

### Non-Functional
- NFR-001 (security): deny-all default preserved; the allow-list + SSRF guard are the load-bearing controls; coarse grants (if any) document their blast radius.
- NFR-002 (versioning): `wasmtime-wasi-http` tracks the pinned `wasmtime`/`wasmtime-wasi` 45 line in lockstep.
- NFR-003 (additive): existing non-HTTP WASM plugins and the deny-all default are unchanged.

## Use Cases **[REQUIRED]**

### UC1: Sandboxed REST source fetches records
- **Actor**: a community/untrusted WASM connector granted `net:api.example.com`.
- **Scenario**: host instantiates with the grant → guest `read()` issues `GET https://api.example.com/v1/records?page=N` via `wasi:http` → host filter allows the authority, injects `Authorization` → response streams back → guest yields records.
- **Outcome**: records fetched from inside the sandbox; the connector never held the API key; it cannot reach any other host.

### UC2: Hostile connector is contained
- **Actor**: a connector that tries `GET http://169.254.169.254/latest/meta-data/` or a non-allow-listed host.
- **Scenario**: host filter rejects the authority pre-dispatch.
- **Outcome**: request fails closed; no internal/metadata egress.

## Architecture **[REQUIRED]**

### Overview
Add `wasmtime-wasi-http` (45) and wire it **conditionally on the capability**:
- `HostState` gains a `WasiHttpCtx` + `impl WasiHttpView`; the http linker imports (`add_only_http_to_linker_sync`-style) are added **only when** the package grants the capability — otherwise the imports are absent and a guest that needs them fails closed.
- The security control lives in the **outgoing-request hook** (`wasmtime-wasi-http`'s `send_request` / `OutgoingRequestConfig` seam): the host inspects the request authority, enforces the per-host allow-list + SSRF denylist, and decorates headers (credentials) before dispatch.
- Capability parsing extends `KNOWN_CAPABILITIES` / `validate_capabilities` with the `"http"` / `"net:<host>"` grammar; `build_wasi_ctx`’s sibling builds the http ctx + allow-list.

## Detailed Design **[REQUIRED]**

### Verified `wasmtime-wasi-http` 45.0.2 API (researched 2026-06-19, docs.rs)
The whole security model hangs off the **`WasiHttpHooks::send_request`** seam — confirmed present and overridable on the pinned line:

```rust
// wasmtime_wasi_http::p2
pub fn add_to_linker_sync<T>(linker: &mut Linker<T>) -> Result<()> where T: WasiHttpView + 'static;
// (also add_only_http_to_linker_sync — adds ONLY wasi:http, not the rest of wasi)

pub trait WasiHttpView {                 // implement on HostState
    fn http(&mut self) -> WasiHttpCtxView<'_>;   // the ONLY required method
}
pub struct WasiHttpCtxView<'a> {
    pub ctx:   &'a mut WasiHttpCtx,
    pub table: &'a mut ResourceTable,
    pub hooks: &'a mut dyn WasiHttpHooks,        // <- embedder seam (trait object)
}
pub trait WasiHttpHooks: Send {          // ALL methods have defaults
    fn send_request(&mut self, request: http::Request<HyperOutgoingBody>,
                    config: OutgoingRequestConfig) -> HttpResult<HostFutureIncomingResponse> { /* default_send_request */ }
    fn is_forbidden_header(&mut self, name: &HeaderName) -> bool { ... }
    // + outgoing_body_buffer_chunks / outgoing_body_chunk_size
}
pub fn default_send_request(request, config) -> ...;   // call after our checks pass
// p2::types::{OutgoingRequestConfig, HostOutgoingRequest}
```

**Wiring:** `HostState` gains `http_ctx: WasiHttpCtx` + `hooks: EgressHooks` (our impl); `impl WasiHttpView for HostState { fn http(&mut self) -> WasiHttpCtxView { WasiHttpCtxView { ctx: &mut self.http_ctx, table: &mut self.table, hooks: &mut self.hooks } } }`. `add_to_linker_sync` is called **only when the package grants the capability** — otherwise the `wasi:http` imports are absent and a guest that needs them fails closed at instantiation (REQ-003). `EgressHooks::send_request` reads `request.uri()` (authority/host/port/scheme), enforces the allow-list + SSRF denylist (reject -> return an `HttpResult` error = guest sees an error code), injects credential headers, then calls `default_send_request`.

### Reference guidance for the embedder's hook (NOT fidius policy — docs/example only)
Post-rescope, the following is **what a good embedder hook does**, shipped as the Phase-2 worked example + checklist (and the integration-test hook), never as fidius code/API:
1. **Allow-list grammar** — a `net:<host>` style per-host allow-list (e.g. `net:api.stripe.com`) is a sane embedder default; the *package's* declared `http` capability is just intent — the embedder's hook decides actual reachability.
2. **SSRF denylist** — reject loopback (127/8, ::1), link-local (169.254/16 incl. `169.254.169.254`, fe80::/10), RFC-1918 (10/8, 172.16/12, 192.168/16), ULA (fc00::/7); the integration test's hook opts loopback back in so the mock server on 127.0.0.1 works. **DNS-rebinding** of an allow-listed name to an internal IP is the residual the embedder closes with resolve-and-pin.
3. **Credential injection** — the embedder's hook can inject host-keyed auth headers (secrets from host config, never the `.wasm`) and use `is_forbidden_header` to stop the guest setting them. Entirely embedder-side.
4. **wasmtime-wasi-http 45 API** — confirmed above: fidius's only job is to call the embedder's `WasiHttpHooks::send_request` before every dispatch and make egress impossible without a hook.

## Alternatives Considered **[REQUIRED]**

- **Coarse `"http"` on/off, scope later** — rejected for v1 (Dylan's call): ships sandbox-defeating arbitrary egress in the interim; the honest "untrusted connector" story needs scoping from day one.
- **Reuse the existing `"network"/"sockets"` raw-socket grant** — rejected: `wasi:sockets` is the wrong layer (no per-host filter, no request hook for credential injection); REST connectors want `wasi:http`.
- **Host-side proxy process the guest talks to over a socket** — rejected: heavier, reintroduces a socket grant, and `wasi:http` already gives a clean host-brokered seam in-process.

## Implementation Plan **[REQUIRED]**

Reduced to mechanism + docs (decompose after sign-off — human-in-the-loop):

- **Phase 1 — Egress mechanism (the whole feature).**
  - `wasmtime-wasi-http` 45 dep (lockstep), gated on the `wasm` feature.
  - An **egress-hook API**: `PluginHost`/`WasmComponentExecutor` accepts an embedder-supplied hook (a `WasiHttpHooks` impl, or a thin fidius trait/closure adapter — e.g. `fn(&http::Request) -> Decision` + decorate — that fidius wraps into `WasiHttpHooks::send_request`). fidius supplies **no policy**.
  - **Two-key gating**: the package declares an `http` capability (intent); `add_to_linker_sync` is wired **iff** the capability is declared **and** the host supplied an egress hook. Missing either -> `wasi:http` imports absent -> guest fails closed. `validate_capabilities` learns the `http` capability name (unknown still fails loud).
  - `HostState` gains `http_ctx: WasiHttpCtx` + the embedder hook; `impl WasiHttpView`.
  - **Integration tests** against a local mock server, where the *test* supplies a policy hook: with a permissive test hook -> GET/POST to the mock works; with a deny-all test hook -> request rejected; capability declared but **no host hook -> fails closed**; no capability -> imports absent.
- **Phase 2 — Docs + worked reference.** WASM capabilities docs: the `http` capability, the two-key model, and **"your egress hook IS the security boundary"** with a checklist (metadata IPs `169.254.169.254`, loopback/`::1`, RFC-1918, ULA, DNS-rebinding caveat). Ship a **worked example** egress hook (allow-list + IP denylist) — as docs/example/the test's hook, **not** a semver module. Optional: a `read() -> Stream<Record>` REST example composing with [[FIDIUS-I-0026]].

Dropped vs the original plan (now embedder responsibility, not fidius): the per-host allow-list, the SSRF denylist, DNS-rebinding defense, and credential injection — these live in the embedder's hook; fidius only guarantees the hook is called before every dispatch and that egress is impossible without one.