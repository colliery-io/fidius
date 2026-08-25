---
id: resolve-and-pin-shadowed-wasi
level: task
title: "Resolve-and-pin: shadowed wasi:sockets/ip-name-lookup + pin table + authorize_dns wiring"
short_code: "FIDIUS-T-0195"
created_at: 2026-08-18T12:34:26.281644+00:00
updated_at: 2026-08-25T01:53:44.336380+00:00
parent: FIDIUS-I-0034
blocked_by: [FIDIUS-T-0194]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: FIDIUS-I-0034
---

# Resolve-and-pin: shadowed wasi:sockets/ip-name-lookup + pin table + authorize_dns wiring

## Parent Initiative

[[FIDIUS-I-0034]]

## Objective

The core mechanism: shadow `wasi:sockets/ip-name-lookup` in the executor's linker with a fidius-owned implementation that (a) consults `EgressPolicy::authorize_dns` before resolving, (b) resolves host-side, and (c) pins `name ↔ IPs` on `HostState`, so `socket_addr_check` can pass `TcpTarget { host: Some(name), addr }` to `authorize_tcp_target`. Feasibility fully proven by the spike (`crates/fidius-host/tests/hostname_pin_spike.rs` — see FIDIUS-I-0034 "Spike questions — ANSWERED").

## Design (from FIDIUS-I-0034 — Detailed Design + spike findings)

**Shadow install** (in `build()`, `wasm.rs`): after `add_to_linker_sync`, and ONLY under the same condition that sets `allow_ip_name_lookup(true)` — `(wants_tcp || wants_udp) && egress.is_some()` — call `linker.allow_shadowing(true)` then `ip_name_lookup::add_to_linker::<HostState, FidiusNameLookup>(&mut linker, accessor)`. Without a grant+policy, upstream's implementation stands and its default-off lookup flag keeps resolution dead (byte-identical for non-granted guests).

**Shadow implementation** (new module, e.g. `executor/name_lookup.rs`):
- View over `HostState`: `&mut ResourceTable` (shared with the rest of WASI so `network` handles and the stream resource interoperate) + pin table + `Arc<dyn EgressPolicy>`.
- Trait glue (spike-proven): `HasData` marker; `ip_name_lookup::{Host, HostResolveAddressStream}`; plus `network::Host` (`convert_error_code` = `error.downcast()`, `network_error_code`) and `network::HostNetwork` (`drop`) — required by the bindgen bound, glue-only, does NOT re-register the network instance.
- `resolve_addresses`: validate the network handle; **`authorize_dns(name)` first — denial returns `ErrorCode::PermanentResolverFailure`** (same error upstream uses for a denied lookup — no new guest-visible failure mode), no resolution, no pin; otherwise resolve via `wasmtime_wasi::runtime::spawn_blocking` + `ResolveAddressStream::Waiting` (mirror upstream's non-blocking shape — the spike's sync resolve was a shortcut; reuse wasmtime-wasi's `ResolveAddressStream`, its variants are pub).
- Pins are recorded when the resolution **completes** (where `Waiting` → `Done` lands: `resolve_next_address`'s poll and the pollable's `ready`), before any address reaches the guest.
- Resolver seam: the actual resolve fn (`&str -> io::Result<Vec<IpAddr>>`) injectable (`#[doc(hidden)]` or pub(crate)) so FIDIUS-T-0196 can test multi-name/same-IP and rotation deterministically without real DNS; default = std `ToSocketAddrs` on `(name, 0)`, matching upstream.

**Pin table** (on `HostState`; decided semantics):
- Normalization: names ASCII-lowercased at pin time and in `TcpTarget.host`; IPs via `to_canonical()` on both pin write and check-side lookup (v4-mapped-v6 must not dodge the pin).
- Two-sided: `name → {IPs}` and `IP → name`. Re-resolution of a name replaces its entry wholesale (stale IPs unpinned — a stale pin must not authorize). Collision on an IP: most-recent resolution wins.
- Lifetime = the store: fresh per unary call (naturally scoped); the persistent configured-instance store relies on replace-on-re-resolve as its eviction policy (no TTL — the pin reflects what this instance was told; it learns of rotation exactly by re-resolving).

**socket_addr_check**: on `TcpConnect`, look up `addr.ip().to_canonical()` in the pin table → `TcpTarget { host, addr }` → `authorize_tcp_target`. UDP paths unchanged. Note the closure currently captures only the policy — it needs access to per-store pins; the natural shape is checking via store data (the check closure receives no store handle — see Implementation Notes).

**Dependency**: promote `wasmtime-wasi-io` (same major as wasmtime-wasi, "46") from dev-dependency to a real optional dependency of the `wasm` feature (needed for `poll::subscribe`).

## Acceptance Criteria

## Acceptance Criteria

- [x] Guest dialing `("db.internal", 5432)`-style names reaches the policy as `TcpTarget { host: Some(name), addr }`; IP-literal dials as `host: None` (spike tests, now against the production path).
- [x] `authorize_dns` denial → guest lookup fails with `PermanentResolverFailure`, nothing resolved or pinned; default policy → lookups work unchanged.
- [x] Shadow installed only under grant+policy; without either, prior behavior byte-identical (existing `tcp_egress_e2e.rs` green unmodified).
- [x] Pin normalization (lowercase names, canonical IPs) and replace-on-re-resolve implemented per design.
- [x] Resolution is non-blocking (spawn_blocking + Waiting), matching upstream's pollable behavior.
- [x] `wasmtime-wasi-io` an optional dep of the `wasm` feature; `angreal test` / `lint` / `license-header` clean.

## Implementation Notes

The one open wiring question: the `socket_addr_check` closure gets `(SocketAddr, SocketAddrUse)` only — no store access — while pins live per-store. Options: (a) keep pins behind an `Arc<Mutex<…>>` created per store in `instantiate()` and clone one handle into the per-store `WasiCtx`'s check closure (the ctx is built fresh per call already — `build_wasi_ctx` gains a pins param); (b) hold pins on `HostState` AND mirror the `Arc` into the closure (same thing, spelled via state). (a) is the spike's shape and fits `build_wasi_ctx`'s per-call construction; prefer it. For the persistent configured store, the same `Arc` lives as long as that store — semantics fall out correctly.

Keep the spike test compiling until FIDIUS-T-0196 replaces it (it pins the standalone-shadow behavior while the production module is introduced).

Dependencies: FIDIUS-T-0194 (API surface). FIDIUS-T-0196 builds the full e2e suite on top.

## Status Updates

- 2026-08-18: Created from FIDIUS-I-0034 decomposition (design phase complete, spike green).
- 2026-08-24: Implemented. New module `executor/name_lookup.rs`: `PinState` (two-sided, lowercase names + canonical IPs, replace-on-re-resolve with collision-safe unpin) + `PinTable`/`Resolver` types + `NameLookupView`/`FidiusNameLookup` with the full trait glue. `resolve_addresses`: authorize_dns first (deny → `PermanentResolverFailure`), IP-literal shortcut (no pin), else `spawn_blocking` + `ResolveAddressStream::Waiting`. One deviation from the task sketch, for the better: pins are written **inside the blocking task** (after resolution succeeds, before the future completes) instead of at the Waiting→Done observation points — strictly earlier, no address reaches the guest un-pinned, and it avoids re-implementing upstream's `Pollable`. Wiring in `wasm.rs`: shadow installed in `build()` under `(tcp||udp grant) && policy` with `allow_shadowing(true)` toggled around the one call; `HostState` gained `pins` + `resolver` (policy rides the existing `EgressHooks`); `build_wasi_ctx` takes the pins Arc (option (a) as planned) and the check recovers `host` via `host_for(addr.ip().to_canonical())`; executor holds a default resolver with `#[doc(hidden)] set_resolver` test seam. `wasmtime-wasi-io` promoted to optional dep of `wasm`. Tests: 4 new PinState unit tests; new `tests/hostname_egress_e2e.rs` (production path — hostname dial seen as `Some("localhost")` + name-authorized echo; IP literal seen as `None` + denied; authorize_dns denial never reaches authorize_tcp_target) 3/3 green; spike suite still green; `tcp_egress_e2e.rs` untouched 6/6; full `angreal test` 90 result blocks 0 failed; lint + license-header clean. All acceptance criteria met.