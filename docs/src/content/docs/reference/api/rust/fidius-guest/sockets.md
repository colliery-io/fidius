---
title: "fidius-guest::sockets"
description: "`fidius::sockets` — capability-gated outbound **TCP** for sandboxed WASM connectors (FIDIUS-I-0033)."
sidebar:
  order: 20
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


`fidius::sockets` — capability-gated outbound **TCP** for sandboxed WASM connectors (FIDIUS-I-0033).

This is the missing seam for connectors that speak a raw binary wire protocol
over TCP rather than HTTP — most importantly database/warehouse drivers (e.g.
Postgres on `:5432`), which a host can't broker as HTTP. A pure-Rust **sync**
driver built on [`tcp::TcpStream`] (with `rustls` layered on top for TLS) runs
fully sandboxed, closing the last gap that kept DB connectors on the retired
native path.

