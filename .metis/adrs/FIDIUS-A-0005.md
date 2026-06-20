---
id: 001-wasm-guest-wasi-http-pinned-client
level: adr
title: "WASM guest wasi:http — pinned client contract + fail-loud host compatibility"
number: 1
short_code: "FIDIUS-A-0005"
created_at: 2026-06-20T01:07:23.356970+00:00
updated_at: 2026-06-20T01:09:44.668809+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-1: WASM guest wasi:http — pinned client contract + fail-loud host compatibility

## Context **[REQUIRED]**

[[FIDIUS-I-0027]] shipped the **host** side of brokered egress (`EgressPolicy` + the two-key, capability-gated `wasi:http`). [[FIDIUS-I-0028]] adds the **guest** side: a `fidius::http` client so a macro-authored connector's `read()` can actually make the outbound call (today only a hand-vendored fixture does).

That raises an ecosystem-stability question: a third party compiles a `.wasm` connector against fidius-guest's wasi:http bindings and **distributes the binary**. The component's `wasi:http@X` import is a hard contract. If the host's `wasmtime`/`wasmtime-wasi-http` provides an incompatible version, a previously-working third-party plugin breaks with no change on the author's part. Concretely, I already hit the failure mode: the `wasi` crate emitted `wasi:http@0.2.12` imports while the host (wasmtime 45) provides `0.2.6` — guest *newer* than host → instantiation fails.

We need the guest wasi:http contract to be **stable for distributed binaries** across host upgrades, and the rare genuine incompatibility to fail **loud and diagnosable**, not as a cryptic crash.

## Decision **[REQUIRED]**

1. **One pinned wasi:http contract in `fidius-guest`.** `fidius::http` (wasm-only) carries a single vendored wasi:http WIT, copied from the exact `wasmtime-wasi-http` version `fidius-host` pins. Every connector shares it — there is no per-connector WIT vendoring. (This is why we chose the centralized `fidius::http` over an `emit_wit`-per-connector route, see [[FIDIUS-I-0028]].)
2. **Pin LOW and move it rarely.** The guest's wasi:http version is treated as a **published ABI**, not a dependency to keep current. The client surface (`outgoing-handler` GET/POST/types) is stable across all of 0.2.x; we pin a conservative 0.2.x and bump it **only** when a genuinely-needed wasi:http feature requires it — never to chase a wasmtime patch. The host's wasmtime is an implementation detail the operator moves freely.
3. **Host-forward is the safe, supported direction.** A connector importing `wasi:http@0.2.N` runs on any host providing `wasi:http ≥ 0.2.N` within the 0.2 line (WASI 0.2 stability + wasmtime's semver-aware component linking). So the common path — operator upgrades fidius-host, third-party plugins keep running — does not break.
4. **Fail loud on incompatibility (same playbook as `ABI_VERSION`).** When a plugin's required wasi:http exceeds what the host provides (host-behind-plugin, or a 0.2→0.3 major), the loader rejects with a clear message ("plugin requires wasi:http 0.2.N, host provides 0.2.M — upgrade the host or rebuild the plugin against an older fidius-guest"), never a mysterious instantiate trap.
5. **Published compatibility contract.** Documented rule: *a wasm plugin built against fidius-guest X runs on any fidius-host ≥ X within the same wasi:http line; a wasi:http major bump (0.2→0.3) is a breaking change shipped only in a fidius major release.*
6. **Internal drift tripwire.** A workspace check asserts fidius-guest's vendored WIT version == the `wasmtime-wasi-http` dep's WIT version; the macro-authored connector E2E (guest → host round-trip) fails red if they ever diverge.

This is the same mechanism-not-policy spine as the rest of egress: fidius ships a **stable client contract** (mechanism); the host `EgressPolicy` brokers (policy).

## Alternatives Analysis

| Option | Pros | Cons | Risk | Cost |
|--------|------|------|------|------|
| **Pinned `fidius::http` in fidius-guest (chosen)** | One contract for the whole ecosystem; central control of the version; drift caught by one tripwire | A maintained re-vendor step on wasmtime bumps | Low | S |
| `emit_wit`-per-connector imports wasi:http | WIT-as-source-of-truth per plugin | N independently-pinned contracts drifting through the ecosystem; the version baked into each binary at build with no central control; macro/codegen complexity | **High** (ecosystem) | M |
| Use the upstream `wasi` crate for the client | No vendoring | It pins its *own* wasi:http (0.2.12) decoupled from the host — the exact skew that broke the fetcher; not under our control | High | S |
| Chase the host wasmtime version on the guest each release | Always "current" | Every fidius patch potentially breaks distributed binaries; turns a stable ABI into a moving target | High | M |

## Rationale **[REQUIRED]**

A distributed `.wasm` binary's import set is an ABI. fidius already treats its *own* interface contract as a versioned ABI with a fail-loud check (`ABI_VERSION` + interface hash); wasi:http is just another axis of the plugin's contract and earns the same discipline. The stability the ecosystem needs is *not* "track upstream" but "**don't move the contract unless you must, and when host and plugin genuinely mismatch, say so clearly**." WASI 0.2's stability guarantee already protects the common (operator-upgrades) direction; centralizing the pin in fidius-guest is what makes that guarantee *keepable* (one version to govern, not N).

## Consequences **[REQUIRED]**

### Positive
- Third-party connectors keep loading across host upgrades within the 0.2 line — the normal ecosystem path is stable by construction.
- One place governs the wasi:http version; drift is impossible to ship silently (tripwire + E2E).
- Genuine incompatibility is a clear, actionable load error, not a crash.

### Negative
- A maintained re-vendor step on a wasmtime/wasi:http bump (guarded, but manual).
- A connector that *needs* a newer wasi:http feature must wait for fidius-guest to bump its pin (deliberate friction — the point).

### Neutral
- `fidius::http` is wasm-only (`#[cfg(target_family="wasm")]`); cdylib connectors use native HTTP directly. (Unify the *contract*, not the *capabilities* — the trust tiers legitimately differ.)

## Review Schedule

### Review Triggers
- WASI ships `wasi:http 0.3` / the Preview-2→Preview-3 transition.
- An adopter needs a wasi:http capability not present in the pinned 0.2.x.
- wasmtime changes its component-linking semver behavior such that host-forward is no longer guaranteed safe.