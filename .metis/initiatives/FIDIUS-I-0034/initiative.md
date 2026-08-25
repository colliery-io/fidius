---
id: hostname-carrying-tcp-egress
level: initiative
title: "Hostname-carrying TCP egress policy — resolve-and-pin for wasi:sockets"
short_code: "FIDIUS-I-0034"
created_at: 2026-08-18T01:55:49.944196+00:00
updated_at: 2026-08-25T02:02:06.147873+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: hostname-carrying-tcp-egress
---

# Hostname-carrying TCP egress policy — resolve-and-pin for wasi:sockets Initiative

*Feature request from embedder **weir** (DB/warehouse connectors on the `tcp` capability tier), 2026-08-17. All fidius code references verified against the checkout on 2026-08-17 (wasmtime pinned at 46.0.1).*

## Context **[REQUIRED]**

`EgressPolicy::authorize_tcp` receives only the **post-resolution** `SocketAddr` (`crates/fidius-host/src/executor/wasm.rs:108`, invoked from the `socket_addr_check` closure at `wasm.rs:469-489`). The hostname the guest actually dialed never reaches the embedder, so hostname-based egress policy for raw TCP is impossible.

**Why this matters:**

1. **Hostname allow-lists are silently dead.** weir's `HostAllowList::authorize_tcp` (weir `crates/weir-runtime/src/lib.rs:246-259`) can only compare `addr.ip().to_string()` / `addr.to_string()` against `allowed_hosts` entries — a DNS-name entry can never match a TCP connect. Any embedder writing `allowed_hosts: ["db.example.internal"]` gets deny-all without knowing why.
2. **IP pinning is operationally broken for managed databases.** RDS / Cloud SQL / Azure endpoints rotate IPs on failover and load-balancing; an IP allow-list that worked yesterday silently blocks (or stops constraining) today.
3. **The current docs promise what the API can't deliver.** The `authorize_tcp` doc (`wasm.rs:99-103`) says the embedder "closes rebinding with resolve-and-pin if it cares" — but resolve-and-pin requires knowing which *name* was resolved, which the policy hook is never told. Only fidius sits in a position to correlate lookup → connect.
4. **Guest-side TLS raises the stakes.** The sockets tier's documented TLS story is rustls layered in-guest (`fidius-guest/src/sockets.rs:21-23, 57-62`), so SNI/name verification lives in the guest. The host's one honest lever over raw-TCP egress is the allow-list — which today can't speak names.

**Current behavior (verified):**
- The `tcp` capability sets `allow_tcp` + `allow_ip_name_lookup(true)` and installs a `socket_addr_check` that routes `SocketAddrUse::TcpConnect` to `policy.authorize_tcp(&addr)` (`wasm.rs:460-490`).
- Name resolution happens **before** the check, inside wasmtime-wasi's `wasi:sockets/ip-name-lookup` (guest `std::net` → wasi-libc → wasmtime-wasi).
- wasmtime's `socket_addr_check` callback signature is `(SocketAddr, SocketAddrUse)` — no hostname parameter, and fidius installs no hook on the lookup path, so the name is consumed entirely inside wasmtime-wasi.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Carry the dialed hostname into TCP egress authorization via an **additive** policy method (`authorize_tcp_target`), backed by a fidius-owned **resolve-and-pin** of `wasi:sockets` name lookups.
- No guest-facing change; existing embedders unaffected (default-method delegation to `authorize_tcp` keeps byte-identical behavior).
- Update `authorize_tcp` docs — resolve-and-pin becomes fidius's mechanism, not an exercise left to the embedder.
- `authorize_dns(&self, name: &str)` hook on the lookup itself (in scope per 2026-08-18 decision) — today `allow_ip_name_lookup` is a blanket boolean, so a guest with the `tcp` capability can probe arbitrary DNS even when every connect would be denied. Default-allow (fail-open to existing behavior).

**Non-Goals:**
- No `start_tls` / host-side TLS on the sockets tier (investigated and rejected; TLS composes guest-side per `sockets.rs:21-23`).
- No guest API change of any kind — `std::net` dialing is untouched, so no WIT version bump and no component-compat scan implications.
- No change to UDP semantics (though `authorize_udp` could adopt `TcpTarget`'s sibling later for symmetry).

## Detailed Design **[REQUIRED]**

### Proposed API (additive, fail-open to existing behavior)

```rust
/// The target of an outbound TCP connect, as the guest expressed it.
pub struct TcpTarget<'a> {
    /// The hostname the guest dialed, when it dialed by name and fidius could
    /// pin the lookup. `None` = dialed by IP literal (or pin unavailable).
    pub host: Option<&'a str>,
    /// The resolved peer the connect will actually reach (existing semantics).
    pub addr: SocketAddr,
}

pub trait EgressPolicy {
    // ... existing methods unchanged, including authorize_tcp(&SocketAddr) ...

    /// Name-aware TCP authorization. Default delegates to `authorize_tcp`,
    /// so existing embedders keep byte-identical behavior.
    fn authorize_tcp_target(&self, target: &TcpTarget<'_>) -> Result<(), EgressDenied> {
        self.authorize_tcp(&target.addr)
    }
}
```

`socket_addr_check` calls `authorize_tcp_target` instead of `authorize_tcp` for `TcpConnect`, constructing `TcpTarget` from the pin table (below).

### `authorize_dns` (in scope — decided 2026-08-18)

```rust
pub trait EgressPolicy {
    // ...

    /// Authorize one guest DNS lookup, BEFORE resolution. Today
    /// `allow_ip_name_lookup` is a blanket boolean tied to the tcp/udp grant,
    /// so a granted guest can probe arbitrary DNS even when every connect
    /// would be denied; this hook closes that.
    ///
    /// **Defaults to allow** — the opposite polarity of `authorize_tcp`,
    /// deliberately: lookup is already open whenever the tier is on, so a
    /// deny default would break every existing embedder's hostname dials.
    /// The connect itself is still gated by `authorize_tcp_target`.
    fn authorize_dns(&self, _name: &str) -> Result<(), EgressDenied> {
        Ok(())
    }
}
```

Consulted at the top of the shadowed `resolve_addresses`, before any resolution or pinning. Denial surfaces to the guest as `ErrorCode::PermanentResolverFailure` — the same error upstream returns for a lookup denied by `allow_ip_name_lookup`, so no new guest-visible failure mode.

### Pin-table semantics (decided 2026-08-18)

- Names are normalized to ASCII-lowercase before pinning and before `TcpTarget.host` is produced (DNS is case-insensitive); IPs are normalized with `to_canonical()` on both the pin write and the check-side lookup (v4-mapped-v6 must not dodge the pin).
- Two-sided map on `HostState`: `name → {IPs}` and `IP → name`. On **re-resolution of a name**, its previous entry is replaced wholesale — the IPs the old resolution mapped that the new one no longer does are unpinned (a stale pin must not authorize; acceptance criterion 4).
- **Collision** (two pinned names sharing an IP): most-recent resolution wins for `TcpTarget.host`. Embedders needing exhaustive candidates can get a `Vec` in a later rev.
- **Lifetime = the store.** Unary calls get a fresh `Store` per call, so pins are naturally per-call. Configured (resident) instances keep one persistent store; the replace-on-re-resolve rule above is the eviction policy — no TTL, because the pin must reflect *what this instance was told*, and the instance learns of rotation exactly by re-resolving.
- Guests dialing IP literals perform no lookup → no pin → `host: None` (documented; name-only policies deny them, the honest default).

### Production wiring

- The shadow is installed in `build()` **only** under the same condition that today sets `allow_ip_name_lookup(true)`: `(wants_tcp || wants_udp) && egress.is_some()`. Without the shadow, upstream's implementation stands and its private `allow_ip_name_lookup` (default false) keeps lookup dead — behavior for non-granted guests is byte-identical.
- Resolution mirrors upstream: `wasmtime_wasi::runtime::spawn_blocking` + `ResolveAddressStream::Waiting` (non-blocking; the spike's synchronous resolve was a shortcut). Pins are recorded when the resolution completes — i.e. in `resolve_next_address`/`ready` when the `Waiting` future lands, before any address is handed to the guest.
- `wasmtime-wasi-io` (same major as wasmtime-wasi) becomes a real optional dependency of the `wasm` feature (needed for `poll::subscribe` in the shadow).
- The trait-bound glue (`network::Host` + `HostNetwork` on the shadow view) is mirrored from upstream — see spike findings.

### Resolve-and-pin mechanism

The hostname only exists inside wasmtime-wasi's `ip-name-lookup` implementation, so fidius must interpose on the lookup:

1. **Shadow `wasi:sockets/ip-name-lookup`** in the linker after `wasmtime_wasi::p2::add_to_linker_sync` (`wasm.rs:754`) with a fidius-owned implementation of `resolve-addresses` that (a) optionally consults `authorize_dns`, (b) resolves host-side (std `ToSocketAddrs` or the same resolver wasmtime uses), and (c) records `name → {IPs}` in a per-instance **pin table** on the host state.
2. `socket_addr_check` consults the pin table to recover the dialed name for the connect's IP and passes `TcpTarget { host, addr }` to the policy.
3. IP-literal dials perform no lookup → `host: None` (documented; embedders allow-listing by name simply deny those, which is the honest default).

### Spike questions — ANSWERED (spike run 2026-08-17, both tests green)

**Spike artifact:** `crates/fidius-host/tests/hostname_pin_spike.rs` (+ `wasmtime-wasi-io = "46"` dev-dependency). Self-contained — replicates the executor's linker setup without touching production code, drives the real `tcp-echo` fixture. Tests: `hostname_dial_pins_and_authorizes_by_name` (guest dials `localhost:<port>` → shadow pins → check recovers `Some("localhost")` → name-keyed authorize → bytes round-trip) and `ip_literal_dial_has_no_pin_and_is_denied_by_name_policy` (IP dial → no pin → `host: None` → name policy denies). **Both pass.**

- **Shadowing: YES.** `linker.allow_shadowing(true)` after `add_to_linker_sync`, then `ip_name_lookup::add_to_linker::<T, D>(&mut linker, accessor)` with a fidius-owned `HasData` marker + view — the shadow takes effect (proven by pins being recorded and name-keyed authorization succeeding end-to-end). No selective WASI linking needed.
- **Standalone reimplementation: tractable, ~110 lines.** Key facts discovered:
  - The bindings (`wasmtime_wasi::p2::bindings::sockets::ip_name_lookup`) are public and **sync** (shared by the sync and async linker paths), so one shadow serves fidius's sync executor.
  - The `with`-mapped resource type is wasmtime-wasi's own `ResolveAddressStream` — its variants are `pub`, so fidius constructs `Done(Ok(addrs.into_iter()))` directly (or `Waiting(spawn_blocking(...))`; `wasmtime_wasi::runtime::{spawn_blocking, AbortOnDropJoinHandle}` are public too — production should use that to mirror upstream's non-blocking behavior; the spike resolved synchronously).
  - Trait-bound wrinkle: `ip_name_lookup::add_to_linker` bounds the view by `network::Host` (owner of the trappable `error-code` conversion) and its `HostNetwork` supertrait — both tiny, mirrored from upstream (`convert_error_code` = `error.downcast()`, `network_error_code`, resource `drop`). This does NOT re-register the `wasi:sockets/network` instance; it's glue-only.
  - `subscribe` needs `wasmtime_wasi_io::poll::subscribe` → **new direct dependency `wasmtime-wasi-io` (same major as wasmtime-wasi)** for the production impl.
  - `From<IpAddr> for IpAddress` is public; use `.ip().to_canonical()` on both the pin write and the check-side lookup so v4-mapped-v6 forms can't dodge the pin.
  - Rust std on wasm32-wasip2 does route hostname dials through `resolve-addresses` (confirmed live: `localhost` hit the shadow); IP-literal dials perform no lookup.
  - Upstream's private `Network.allow_ip_name_lookup` flag is `pub(crate)` and unreadable — irrelevant: fidius gates by installing the shadow only under the same condition that sets `allow_ip_name_lookup(true)` today (tcp/udp grant + policy), and/or an own flag on `HostState`.
- **Vendored-WIT/compat posture (FIDIUS-A-0005): survives.** The shadow is built from wasmtime-wasi's own bindgen types, so it moves in lockstep with the pinned wasmtime major — same story as wasi-http. A wasmtime major bump can change trait shapes (as 45→46 did for `WasiView`); that's the existing upgrade cost, now with one more touchpoint (note for FIDIUS-T-0159's pin-bump automation).
- **Pin-table lifetime — important existing-architecture fact:** fidius instantiates a **fresh Store per call** (`instantiate()` in `wasm.rs`), so a per-`HostState` pin table is naturally per-call and staleness is a non-issue for unary calls. The TTL/eviction question is real only for the **persistent store** used by configured instances (`configure_from_loaded`, `wasm.rs:927`) — resident guests re-resolving rotated names. Design decision still open (propose: most-recent-resolution-wins per name, plus most-recent wins per IP on collision).
- **TOCTOU** between lookup and connect remains inherent; the pin table narrows it to "an address this instance was actually given for that name" — goes in the docs as designed.

## Alternatives Considered **[REQUIRED]**

- **Embedder-side forward resolution** (no fidius change): the policy resolves its own allow-listed names at authorize time and matches IPs. Works only when embedder and guest resolutions agree — round-robin DNS and split-horizon make false denies routine, and it inverts resolve-and-pin (the pin should be what the *guest* was told). Rejected as the primary, but usable as a stopgap.
- **Brokered-TCP subsystem** (`fidius:tcp-broker` with connect/read/write owned by host functions): gives the host the name *and* the byte path, but duplicates wasi:sockets semantics wholesale and reshapes the guest API. Rejected for this need (see weir's DB-TLS investigation, 2026-08-17 — TLS is going guest-side per the sockets module's own design, so the byte path is not required, only the name).
- **Upstream wasmtime-wasi change** (hostname parameter on `socket_addr_check` or a lookup hook): cleanest long-term; slow and outside fidius's control. Worth filing upstream independently; the shadow approach avoids waiting.

## Acceptance Criteria

## Acceptance Criteria

- [x] A guest dialing `("db.internal", 5432)` reaches the policy as `TcpTarget { host: Some("db.internal"), addr }`; dialing `10.0.0.5:5432` reaches it as `TcpTarget { host: None, addr }`. *(hostname_egress_e2e: `hostname_dial_reaches_policy_with_name_and_echoes`, `ip_literal_dial_reaches_policy_as_none_and_is_denied`)*
- [x] An embedder overriding only the old `authorize_tcp` observes byte-identical behavior (default-method delegation proven by test). *(unit: `egress_policy_tests::authorize_tcp_target_default_delegates_to_authorize_tcp`; e2e: `legacy_policy_hostname_dial_unchanged`; regression: `tcp_egress_e2e.rs` unmodified, 6/6)*
- [x] Name-keyed allow-list e2e: allow `db.internal` → connect succeeds; connect to a non-listed name is denied; a second name resolving to the same IP is denied unless listed (pin correctness, not IP fallthrough). *(`same_ip_second_name_denied_unless_listed` — denied even though that IP was authorized moments earlier under the listed name; `unlisted_name_denied`)*
- [x] Resident-lifetime test: re-resolution after rotation updates the pin; stale pins do not authorize a name the current resolution no longer maps to. *(`rotation_replaces_pin_and_stale_ip_loses_authority` — v4→v6 rotation within one store via the fixture's new `connect-seq`; the post-rotation dial to the old IP arrives `host: None` and is denied)*
- [x] `authorize_tcp` docs updated — resolve-and-pin is now fidius's mechanism, not an exercise left to the embedder. *(trait docs + `docs/explanation/wasm-capabilities.md` hostname allow-list section + regenerated plissken API docs)*
- [x] `authorize_dns`: a policy denying a name makes the guest's lookup fail (`PermanentResolverFailure`) with no resolution and no pin; the default (no override) keeps lookups working unchanged. *(`authorize_dns_denial_fails_lookup_before_connect` + default-allow unit test; every other e2e exercises the default path)*

## Implementation Plan **[REQUIRED]**

1. ~~**Spike**~~ ✅ **DONE 2026-08-17** — shadow works; see "Spike questions — ANSWERED" above. Artifact: `crates/fidius-host/tests/hostname_pin_spike.rs` + `wasmtime-wasi-io` dev-dep (uncommitted spike work, kept as the seed of the eventual e2e tests).
2. **Implementation** (2–4 days): `TcpTarget` + `authorize_tcp_target` on `EgressPolicy`; production shadow in `build()` (installed only under tcp/udp grant + policy, i.e. the same condition as `allow_ip_name_lookup(true)`); pin table on `HostState` (per-call store ⇒ naturally scoped; decide resident-instance semantics for the configured/persistent store); `spawn_blocking`-based resolve to mirror upstream's non-blocking behavior; promote `wasmtime-wasi-io` to a real optional dependency of the `wasm` feature; tests per acceptance criteria; `authorize_tcp` doc update. Optional in same pass: `authorize_dns` hook.
3. Fallbacks (selective WASI linking / upstream wasmtime change) — **not needed**.

**Embedder follow-up (weir, once released):** `HostAllowList` implements `authorize_tcp_target` to match `allowed_hosts` entries against `target.host` (names) and `target.addr` (IPs) — ~an hour at `crates/weir-runtime/src/lib.rs:246`, plus enabling hostname entries in per-connection egress config later (weir tech-debt WEIR-I-0034 is the related "per-connection allow-list" decision ticket).