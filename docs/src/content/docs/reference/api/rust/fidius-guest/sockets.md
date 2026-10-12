---
title: "fidius-guest::sockets"
description: "`fidius::sockets` — capability-gated outbound **TCP** for sandboxed WASM connectors (FIDIUS-I-0033)."
editUrl: false
sidebar:
  order: 19
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

`fidius::sockets` — capability-gated outbound **TCP** for sandboxed WASM connectors (FIDIUS-I-0033).

This is the missing seam for connectors that speak a raw binary wire protocol
over TCP rather than HTTP — most importantly database/warehouse drivers (e.g.
Postgres on `:5432`), which a host can't broker as HTTP. A pure-Rust **sync**
driver built on `tcp::TcpStream` (with `rustls` layered on top for TLS) runs
fully sandboxed, closing the last gap that kept DB connectors on the retired
native path.

**Backed by `std::net` → `wasi:sockets`**

Unlike [`crate::http`](../../../rust/fidius-guest/http/) (which wraps `wasi:http` because std has no HTTP), TCP
needs no hand-written WIT: on `wasm32-wasip2` Rust's `std::net::TcpStream`
**is** `wasi:sockets` (since Rust 1.83, std's net layer calls wasi-libc's
socket functions). So `tcp::TcpStream` is a thin, blocking wrapper over
`std::net::TcpStream` and is **portable**: identical source compiles for the
host (a normal socket) and for `wasm32-wasip2` (a sandboxed one). Sync/blocking
semantics suit the off-tokio `block_on` execution model.

**Two-key, fail-closed — the same gate as `http`**

A connect only succeeds when **both** keys are turned:
1. the package declares the `tcp` capability in `[wasm].capabilities`, **and**
2. the host supplied an `EgressPolicy` whose `authorize_tcp(host:port)` allows
   the (resolved) peer.

Miss either and the connect is refused (the deny-all `WasiCtx` default) — there
is no ambient network. The guest sees a normal [`std::io::Error`](../../../rust/fidius-guest/frame/#enum.Frame); it never
learns the policy's reasoning, by design.

```rust
use std::io::{Read, Write};
use fidius_guest::sockets::tcp;

// Allowed only if the host's EgressPolicy::authorize_tcp permits db:5432.
let mut conn = tcp::connect("db.internal", 5432)?;
conn.write_all(startup_packet)?;
let mut buf = [0u8; 1024];
let n = conn.read(&mut buf)?;
```

`TcpStream` implements `Read` + `Write`, so a TLS layer composes directly:

```rust
let mut tls = rustls::Stream::new(&mut client_conn, &mut conn);
tls.write_all(startup_packet)?; // TLS over the sandboxed TCP stream
```
