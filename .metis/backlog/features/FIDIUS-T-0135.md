---
id: capability-gated-outbound-http-for
level: task
title: "Capability-gated outbound HTTP for WASM guests (wasi:http outgoing-handler)"
short_code: "FIDIUS-T-0135"
created_at: 2026-06-19T16:13:17.007756+00:00
updated_at: 2026-06-19T16:13:17.007756+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
initiative_id: NULL
---

# Capability-gated outbound HTTP for WASM guests (wasi:http outgoing-handler)

> **Feature request** raised by the **weir** team (downstream adopter). Against fidius (the plugin/host framework), not weir. Relates to **[[FIDIUS-A-0004]]** (which explicitly fences "brokered network I/O" out of core as a separate, adopter-driven track) and **[[FIDIUS-I-0026]]** (the streaming initiative flagged brokered network I/O as a Phase-2-adjacent dependency to "flag before Phase 2 ships externally"). Upstream refs from the requester: WEIR-A-0002 (deny-all WASM sandbox), WEIR-A-0013 (credential injection rides the same host-import surface).

## Objective **[REQUIRED]**

Expose a **host-controlled, capability-gated outbound-HTTP import** (`wasi:http` `outgoing-handler`) that `PluginHost` can grant to a sandboxed WASM guest. Today the WASM backend wires WASI into the linker with a **deny-all `WasiCtx`** (no FS, no env, no inherited stdio) — capability isolation is the entire point of the open/community-connector tier. There is currently **no way to grant a guest the ability to make outbound HTTP requests**, so a WASM connector whose job is to fetch from a REST API literally cannot run.

The same host-import surface is the natural place for **credential injection** (the host rewrites/decorates the outbound request — adds auth headers, rotates tokens — so the guest never sees the secret), making this one mechanism for both egress and secret brokering.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P1 - High (important for user experience)
  - Not P0 for the adopter *today* only because they have a working fallback (the trusted in-process dylib path carries live HTTP). But it **blocks the entire sandboxed/community-connector tier**, which is the differentiated reason to use the WASM backend at all.

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: weir's v0 manifest-driven connectors are REST/HTTP **sources**. Their generated WASM `read` must make outbound HTTP calls to fetch records. With no HTTP capability to grant, the recorded state is: *"HTTP-in-sandbox needs a `wasi:http` grant (so generated wasm `read` is a stub; dylib carries live HTTP)."* → a REST connector runs live only on the **dylib (trusted, in-process)** path; the **WASM build's `read` is a stub** — the sandboxed/community path can't actually fetch yet.
- **Business Value**: unlocks "Airbyte-style connectors without Docker" / untrusted community connectors — the core pitch of the sandboxed WASM tier. Combined with FIDIUS-I-0026 server-streaming, a REST source becomes a real `read() -> Stream<Record>` that paginates an API from inside the sandbox.
- **Effort Estimate**: **L** (linker + host-state wiring is small; the security-meaningful capability model — per-host/port allow-listing — and the credential-injection hook are the substantive parts; plus integration tests against a mock server).

## Acceptance Criteria **[REQUIRED]**

- [ ] `PluginHost`/`WasmComponentExecutor` can grant `wasi:http` outgoing-handler to a guest **only** when the package's `[wasm].capabilities` allow-list opts in (e.g. a new `"http"` capability, or finer `"net:api.example.com"` host scoping); **default remains deny-all** — a guest with no grant that imports `wasi:http/outgoing-handler` fails closed (instantiation or call error), exactly like FS today.
- [ ] A WASM guest granted the capability can perform a real outbound GET/POST and receive the response body/status; verified with an integration test against a **local mock HTTP server** (no external network in CI).
- [ ] A guest **without** the grant cannot reach the network — negative test asserts the call fails (no silent allow).
- [ ] **Scoping**: at minimum coarse on/off; ideally a per-host (and optionally per-port/scheme) allow-list so a connector to `api.stripe.com` can't exfiltrate elsewhere. Unknown/typo capability names fail loud at load (matching the existing `validate_capabilities` behaviour).
- [ ] **Credential-injection hook** (can be a follow-on sub-task, but design for it now): a host-side seam to decorate/rewrite the outbound request (inject `Authorization`, etc.) so the guest never handles the secret (satisfies WEIR-A-0013's "one capability mechanism").
- [ ] Docs: capability name(s), manifest example, and the security model (host brokers egress; guest is deny-all by default) added to the WASM capabilities explanation.
- [ ] Additive: existing non-HTTP WASM plugins and the deny-all default are unchanged.

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach
- Touch point is `crates/fidius-host/src/executor/wasm.rs`:
  - `KNOWN_CAPABILITIES` / `validate_capabilities` / `build_wasi_ctx` — add an `"http"` (and/or `"net:<host>"`) capability.
  - The linker is currently `wasmtime_wasi::p2::add_to_linker_sync(&mut linker)`. Add **`wasmtime-wasi-http`** (`add_only_http_to_linker_sync`-style) **gated on the capability**, and give `HostState` a `WasiHttpCtx` + `impl WasiHttpView` alongside the existing `WasiView`. (New dep: `wasmtime-wasi-http` at the pinned wasmtime 45 line.)
  - For per-host scoping + credential injection, implement the outgoing-request hook (wasmtime-wasi-http exposes a `send_request`/`OutgoingRequestConfig` seam) so the host can filter by authority and decorate headers before dispatch.
- **Distinct from the existing `"network"`/`"sockets"` capability**, which does `inherit_network()` + `allow_ip_name_lookup` — that's coarse **raw WASI sockets** (`wasi:sockets`), not the higher-level host-brokered `wasi:http`, and has no per-host filtering or credential-injection seam. This ticket is specifically the `wasi:http` path (the right layer for REST connectors + secret brokering).
- Mirror the deny-all-by-default discipline and the "unknown capability fails loud" pattern already in place for FS/env.

### Dependencies
- None hard. Composes with [[FIDIUS-I-0026]] (a streaming REST source = server-streaming + this egress capability). Per [[FIDIUS-A-0004]], this is the "brokered I/O" track that ADR deliberately scoped as adopter-driven — this ticket is that track being requested.

### Risk Considerations
- **Security is the whole point**: a coarse on/off grant that allows arbitrary egress weakens the sandbox value. The per-host allow-list (or at least documenting the coarse grant's blast radius) is what keeps the "untrusted community connector" story honest. Don't ship egress without the scoping story decided.
- `wasmtime-wasi-http` version must track the pinned `wasmtime`/`wasmtime-wasi` (45) — keep them in lockstep.
- SSRF / metadata-endpoint access (e.g. `169.254.169.254`) should be considered in the host-side filter.

## Status Updates **[REQUIRED]**

### 2026-06-19 — filed (backlog)
Raised by the weir team against fidius. Captured as a feature backlog item. Anticipated by FIDIUS-A-0004 (brokered I/O fenced as a separate track) and FIDIUS-I-0026 (flagged as a dependency). Not yet scheduled — needs a human call on whether it becomes its own initiative (likely, given the capability-scoping + credential-injection scope) and prioritization against the remaining Phase-2 streaming work (WS.5/WS.6).

### 2026-06-19 — promoted to an initiative
Dylan's call: this becomes its own initiative — **[[FIDIUS-I-0027]]** (Capability-gated wasi:http egress) — with v1 shipping **per-host allow-list + SSRF guard** (not coarse on/off). This ticket is the origin record; planning + decomposition live in I-0027.