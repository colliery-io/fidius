---
id: 001-capability-gated-per-variable
level: adr
title: "Capability-gated, per-variable environment access for WASM guests (env:VAR_NAME)"
number: 1
short_code: "FIDIUS-A-0009"
created_at: 2026-06-20T15:13:12.857139+00:00
updated_at: 2026-06-20T15:13:54.569306+00:00
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

# ADR-1: Capability-gated, per-variable environment access for WASM guests (env:VAR_NAME)

> **Retroactive record.** The scoped-env capability shipped as FIDIUS-T-0142; this ADR
> documents that decision after the fact so the WASM capability surface has a uniform
> paper trail alongside egress (FIDIUS-I-0027) and filesystem ([[FIDIUS-A-0008]]).

## Context **[REQUIRED]**

The WASM sandbox is **deny-all**: a guest sees no environment unless granted. But real
connectors are configured through the environment — an API base URL, a region, a log
level, a feature flag. The naive grant ("let the guest read the environment") would
hand an untrusted guest **every** host variable: `AWS_SECRET_ACCESS_KEY`,
`DATABASE_URL`, session tokens — i.e. all the operator's secrets. We need env access
that is useful for configuration without being a wholesale secret leak.

## Decision **[REQUIRED]**

Environment access is **per-variable**, via one capability form:

- `env:<NAME>` — expose exactly the host variable `NAME` (read-only) to the guest.

Mechanics (`crates/fidius-host/src/executor/wasm.rs`):
1. `build_wasi_ctx` calls `b.env(NAME, value)` for each `env:NAME` grant, reading the
   value from the host environment. The guest reads it with plain `std::env::var` — no
   fidius-specific guest API.
2. **Bare `env` (inherit the whole host environment) is rejected at load** with a clear
   error pointing at the scoped form — a coarse grant is a footgun.
3. `env:` with an **empty** name is rejected at load.
4. A variable **unset** on the host is skipped silently (the guest's `var()` returns
   `Err`, as it would natively).
5. **Deny-all default is unchanged** — no `env:*` grant ⇒ no environment.
6. fidius ships **mechanism, not policy** — the host/manifest decides *which* variables
   to expose; there is no built-in allow/secret-list.

## Alternatives Analysis

| Option | Pros | Cons | Risk | Cost |
|--------|------|------|------|------|
| **Per-variable `env:NAME` (chosen)** | Least-privilege; mirrors `fs:`/`http` gating; secrets stay host-side unless named | Author must list each variable | Low | S |
| **Bare `env` (inherit all)** | Trivial; one word | Hands the guest **every** secret; defeats the sandbox | High | none |
| **No env at all (config via args only)** | Smallest surface | Forces all config through method args/manifest; awkward for ambient deploy config | Low | none |
| **Prefix grant (`env:STRIPE_*`)** | Fewer entries for grouped vars | Glob risk (accidental over-grant); harder to audit | Med | M |

## Rationale **[REQUIRED]**

Naming each variable makes least-privilege the *only* way to express the grant — the
guest gets precisely the configuration it was given and cannot enumerate or read
anything else. Rejecting bare `env` keeps the dangerous "all secrets" grant from being
a one-word typo. This is the same shape as the filesystem ([[FIDIUS-A-0008]]) and egress
(FIDIUS-I-0027) decisions, so the whole capability allow-list reads with one mental
model: **deny-all, grant narrowly by name, host owns policy.** Prefix/glob grants were
rejected as an auditing hazard; they can be revisited if demand appears.

## Consequences **[REQUIRED]**

### Positive
- Connectors get ambient configuration without exposing host secrets.
- Least-privilege by construction; each exposed variable is explicit and auditable.
- Uniform with `fs:`/`http` — one capability model across the sandbox.

### Negative
- Authors must enumerate every variable a connector reads (no wildcard).

### Neutral
- The guest uses stock `std::env::var`; no fidius guest API or WIT surface for env.
- Unset variables are indistinguishable to the guest from "not granted" (both `Err`).

## Review Schedule

### Review Triggers
- Demand for grouped grants (prefix/glob) that per-variable listing makes unwieldy.
- A WASI environment-access semantics change.