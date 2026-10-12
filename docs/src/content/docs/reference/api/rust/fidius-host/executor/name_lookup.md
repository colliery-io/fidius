---
title: "fidius-host::executor::name_lookup"
description: "Resolve-and-pin for `wasi:sockets` name lookups (FIDIUS-I-0034)."
editUrl: false
sidebar:
  order: 34
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Resolve-and-pin for `wasi:sockets` name lookups (FIDIUS-I-0034).

fidius shadows `wasi:sockets/ip-name-lookup` in the linker (after
`add_to_linker_sync`, with `allow_shadowing(true)`) with this module's
implementation, which:

1. consults [`EgressPolicy::authorize_dns`](../../../../rust/fidius-host/executor/wasm/#EgressPolicy.authorize_dns) **before** resolving — a denial
   fails the guest's lookup with `permanent-resolver-failure` (the same
   error upstream returns for a lookup denied outright), resolves nothing,
   and pins nothing;
2. resolves host-side (std `ToSocketAddrs`, matching upstream, unless a
   test injects a resolver); and
3. records `name ↔ IPs` in a per-store `PinTable` **inside** the blocking
   resolution task — the pin is written before the future completes, so no
   address ever reaches the guest un-pinned.

`socket_addr_check` then recovers the dialed name for a connect's IP from
the pin table and hands the policy a `TcpTarget { host: Some(name), .. }`.
The shadow is installed only under the same two-key condition that enables
name lookup at all (a `tcp`/`udp` grant AND an embedder policy); otherwise
upstream's implementation stands untouched.
