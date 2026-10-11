---
title: "fidius-guest::client_stream"
description: "Guest-side consumer for **client-streaming over WASM** (FIDIUS-I-0030 CS2.3)."
sidebar:
  order: 11
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


Guest-side consumer for **client-streaming over WASM** (FIDIUS-I-0030 CS2.3).

WASM can't share a raw `FidiusStreamHandle` across the sandbox (no shared
memory like cdylib), so the guest **imports** a host function and pulls through
it. The component imports `fidius:stream-pull/pull.next() -> option<list<u8>>`;
[`WasmHostStream`] presents the bincode items the host produces as an
`Iterator`, which the macro wraps in the `Stream<T>` the method consumes. This
`generate!` composes with the macro's interface-export `generate!` exactly like
`fidius_guest::http`'s wasi:http import (I-0028).

## Structs

### <span id="struct.WasmHostStream"></span>`struct WasmHostStream<T>`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


Guest consumer of a host-produced stream — the client-streaming `Stream<T>` argument. Each `next()` pulls one bincode item via the imported host func and deserializes it into `T`; `None` from the import = end of stream.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `_marker` | `PhantomData < fn () -> T >` |  |



