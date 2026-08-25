---
id: egresspolicy-api-tcptarget
level: task
title: "EgressPolicy API: TcpTarget + authorize_tcp_target + authorize_dns (default-delegating, byte-identical)"
short_code: "FIDIUS-T-0194"
created_at: 2026-08-18T12:34:22.088353+00:00
updated_at: 2026-08-24T13:01:06.960950+00:00
parent: FIDIUS-I-0034
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0034
---

# EgressPolicy API: TcpTarget + authorize_tcp_target + authorize_dns (default-delegating, byte-identical)

## Parent Initiative

[[FIDIUS-I-0034]]

## Objective

Extend the `EgressPolicy` trait (`crates/fidius-host/src/executor/wasm.rs`) with the name-aware TCP authorization surface — additively, so every existing embedder keeps byte-identical behavior with zero code changes. This slice also flips `socket_addr_check` to route `TcpConnect` through the new method with `host: None` (no pin table yet — that's FIDIUS-T-0195), which proves the delegation default in production wiring, not just in a unit test.

## Design (from FIDIUS-I-0034 — Detailed Design)

- `pub struct TcpTarget<'a> { pub host: Option<&'a str>, pub addr: SocketAddr }` — the target of an outbound TCP connect as the guest expressed it. `host: None` = dialed by IP literal (or pin unavailable).
- `fn authorize_tcp_target(&self, target: &TcpTarget<'_>) -> Result<(), EgressDenied>` — **default delegates to `self.authorize_tcp(&target.addr)`**, so an embedder overriding only the old method observes identical behavior.
- `fn authorize_dns(&self, _name: &str) -> Result<(), EgressDenied>` — **default `Ok(())`** (allow). Deliberately the opposite polarity of `authorize_tcp`'s default-deny: lookup is already open whenever the tcp/udp tier is on, so a deny default would break every existing embedder's hostname dials. Not yet consulted anywhere in this slice — FIDIUS-T-0195 wires it into the shadowed lookup.
- In the `socket_addr_check` closure (`wasm.rs:469-489`), `SocketAddrUse::TcpConnect` now calls `policy.authorize_tcp_target(&TcpTarget { host: None, addr })`.
- Rewrite the `authorize_tcp` doc block (`wasm.rs:99-110`): resolve-and-pin is fidius's mechanism (landing in FIDIUS-T-0195), no longer "an exercise for the embedder"; document `host: None` semantics for IP-literal dials; document `authorize_dns`'s polarity and purpose.

## Acceptance Criteria

## Acceptance Criteria

- [x] `TcpTarget`, `authorize_tcp_target`, and `authorize_dns` exist on `EgressPolicy` with the defaults above; exported alongside `EgressPolicy`/`EgressDenied` in the executor module's public surface.
- [x] `socket_addr_check` routes `TcpConnect` through `authorize_tcp_target` (with `host: None` for now); UDP paths unchanged.
- [x] Unit test: a policy overriding **only** `authorize_tcp` sees identical allow/deny results through `authorize_tcp_target` (delegation proven).
- [x] Existing suite green (`angreal test`), notably `tcp_egress_e2e.rs` unchanged and passing — the behavioral proof that old embedders are unaffected.
- [x] Doc blocks updated per above; `angreal lint` + `angreal license-header` clean.

## Implementation Notes

Pure API slice — no linker/shadow work, no new dependencies. Keep `TcpTarget` non-exhaustive-friendly (it's a plain struct with public fields per the FR; if we want room for later fields like exhaustive name candidates, note it in docs rather than `#[non_exhaustive]`, which would break the literal-construction ergonomics embedders need for tests).

Dependencies: none. FIDIUS-T-0195 builds on this.

## Status Updates

- 2026-08-18: Created from FIDIUS-I-0034 decomposition (design phase complete, spike green).
- 2026-08-24: Implemented. `TcpTarget` + `authorize_tcp_target` (default-delegates) + `authorize_dns` (default-allow) on `EgressPolicy` in `executor/wasm.rs`; `socket_addr_check` routes `TcpConnect` through `authorize_tcp_target(&TcpTarget { host: None, addr })` (UDP unchanged); `authorize_tcp` docs rewritten to point at the name-aware path. Exported through `executor.rs`, `fidius-host/src/lib.rs`, and the `fidius` facade. Unit tests `egress_policy_tests::*` pass; full `angreal test` green (0 failures; `tcp_egress_e2e.rs` untouched, 6/6); `angreal lint` clean after `cargo fmt`; license headers checked. All acceptance criteria met.