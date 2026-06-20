---
id: guest-side-brokered-http-fidius
level: initiative
title: "Guest-side brokered HTTP — fidius::http for sandboxed WASM connectors"
short_code: "FIDIUS-I-0028"
created_at: 2026-06-20T01:07:24.257487+00:00
updated_at: 2026-06-20T01:41:58.316916+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: guest-side-brokered-http-fidius
---

# Guest-side brokered HTTP — fidius::http for sandboxed WASM connectors Initiative

> Guest-side completion of egress. Host side ([[FIDIUS-I-0027]]) ships the `EgressPolicy` broker; this lets a **macro-authored** connector's `read()` actually make the call. Compatibility decisions of record: [[FIDIUS-A-0005]]. Design settled with Dylan over the prior discussion.

## Context **[REQUIRED]**

A sandboxed WASM connector built with `#[plugin_interface]`/`#[plugin_impl]` can't currently make outbound HTTP — the macro adapter only *exports* the connector interface; nothing imports `wasi:http` or provides a client. Today only the hand-vendored `fetcher` fixture does it. weir's connectors are macro-authored, so they're stuck.

The host already brokers egress (capability + `EgressPolicy`, two-key, fail-closed). This initiative adds the guest client so the loop closes: `read()` calls `fidius::http::get(url)` → the component imports `wasi:http` → the host's policy brokers it.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Ship **`fidius::http`** (wasm-only) — a small client (`get`/`post`-ish over `wasi:http`) a connector's `read()` calls directly. No hand-vendored WIT, no raw bindings.
- Keep the **macro untouched**: the existing `#[plugin_impl]` export adapter stays export-only; `wasi:http` import is provided by `fidius::http`'s usage (the proven fetcher pattern, pinned to the host's 0.2.x instead of the `wasi` crate's skewed version).
- A **macro-authored connector fixture** + E2E proving the full loop guest→host onto the shipped `EgressPolicy` (retires the hand-built fetcher as the canonical example).
- **Ecosystem stability** per [[FIDIUS-A-0005]]: one pinned wasi:http contract, fail-loud host-version mismatch, a drift tripwire, a published compatibility rule.

**Non-Goals:**
- A native (cdylib) HTTP client. `fidius::http` is wasm-only; cdylib/trusted connectors use `reqwest`/etc. directly. (Unify the *contract*, not the *capabilities*.)
- A high-level HTTP framework (retries, JSON helpers, pagination) — that's the connector author's / weir's layer. fidius ships the thin brokered transport.
- Inbound/serving; changing the host `EgressPolicy` (already shipped).

## Requirements **[REQUIRED]**

### Functional
- REQ-001: `fidius::http` exposes a minimal request API (method, url, headers, body) → response (status, headers, body) over `wasi:http/outgoing-handler`, `#[cfg(target_family="wasm")]`.
- REQ-002: A connector using only the macro + `fidius::http` builds for `wasm32-wasip2` and imports `wasi:http@0.2.x` (the pinned version) — no per-connector WIT vendoring, no second `generate!` conflict with the macro's export adapter.
- REQ-003: The host loads such a connector via `load_wasm` and, given the `http` capability + an `EgressPolicy`, the guest's request is brokered (allow/deny/inject) — end to end.
- REQ-004 (FIDIUS-A-0005): a host-incompatible wasi:http version is rejected at load with a clear, actionable message (not a raw instantiate trap).

### Non-Functional
- NFR-001 (stability): the guest wasi:http pin is governed centrally and bumped only deliberately; host-forward upgrades don't break distributed connectors.
- NFR-002 (versioning): vendored WIT == the `wasmtime-wasi-http` the host pins; a tripwire fails on drift.
- NFR-003 (additive): non-HTTP plugins + all existing backends unchanged; `fidius::http` is absent on non-wasm builds.

## Use Cases **[REQUIRED]**

### UC1: weir's REST source, end to end
- **Actor**: a macro-authored connector with `fn read(&self, cfg) -> fidius::Stream<Record>` that paginates an API.
- **Scenario**: `read()` calls `fidius::http::get(page_url)` → component imports `wasi:http` → host (with `net:api.example.com` policy + the `http` capability) brokers + injects the API key → records stream back.
- **Outcome**: a sandboxed, signed, untrusted connector fetches from a REST API it's policy-bound to reach — the "Airbyte-without-Docker" loop, fully macro-authored.

## Architecture **[REQUIRED]**

### Overview
- **`fidius-guest::http`** (`#[cfg(target_family="wasm")]`): a single `wit_bindgen::generate!` over the **vendored, host-matched wasi:http WIT** (the 0.2.6 set already extracted for the fetcher, world-stripped), wrapped in a thin `Request`/`Response` + `get`/`post`. Re-exported as `fidius::http`. This is the *only* place the wasi:http import/version lives.
- **Macro: unchanged.** `#[plugin_impl]`'s adapter remains export-only; the `wasi:http` import propagates from `fidius::http`'s usage at link time (proven by the fetcher, which combined a separate wasi:http provider with one export `generate!`). Key constraint to verify: fidius-guest's `generate!` (import-only wasi:http world) and the connector crate's macro `generate!` (export world) must coexist without a duplicate-`wasi`/type conflict — they're disjoint packages, so they should, but this is the load-bearing spike.
- **Host: a wasi:http version check** in `load_wasm` (or the executor) per [[FIDIUS-A-0005]] — read the component's `wasi:http` import version, compare to the host's, reject loud if host < plugin.

### Open spike (Phase 1 first task)
Confirm two `wit_bindgen::generate!` (fidius-guest import-only + connector export) compose in one `wasm32-wasip2` component without conflict. If they *don't*, fall back to **committed (pre-generated) bindings** in fidius-guest (the literal `wasi`-crate shape) so there's only one `generate!` in the connector crate.

## Alternatives Considered **[REQUIRED]**

See [[FIDIUS-A-0005]] for the full table. Chosen: centralized pinned `fidius::http` in fidius-guest. Rejected: `emit_wit`-per-connector (N drifting contracts, macro complexity, ecosystem-fragility); the upstream `wasi` crate (pins its own skewed version — the fetcher bug); chasing wasmtime each release (turns a stable ABI into a moving target).

## Implementation Plan **[REQUIRED]**

Decompose after sign-off (human-in-the-loop):
- **GH.1 — `fidius::http` client.** Vendor + world-strip the host-matched wasi:http WIT into fidius-guest; the spike (two-`generate!` coexistence, else committed bindings); the `Request`/`Response` + `get`/`post` wrapper, wasm-only; `fidius::http` re-export. Guest-side unit/build proof.
- **GH.2 — Macro-authored connector fixture + E2E.** A `#[plugin_interface]`/`#[plugin_impl]` connector whose `read()` uses `fidius::http`; built for wasip2; loaded via `PluginHost::load_wasm` with the `http` capability + a test `EgressPolicy`; E2E (allow/deny/fail-closed) against the loopback mock — the full guest→host loop. Retire/repoint the hand-built fetcher.
- **GH.3 — Stability guards + docs (FIDIUS-A-0005).** The fail-loud host wasi:http version check at load; the vendored-WIT-vs-dep drift tripwire; the published compatibility rule + a connector how-to in the docs.