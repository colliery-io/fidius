---
title: "fidius-host::client_stream"
description: "Host-side **producer** for client-streaming (FIDIUS-I-0030 / ADR-0007)."
editUrl: false
sidebar:
  order: 29
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Host-side **producer** for client-streaming (FIDIUS-I-0030 / ADR-0007).

The inverse of the server-streaming `StreamState`: here the **host** produces
and the **guest** consumes. [`host_producer_handle`](../../../rust/fidius-host/client_stream/#fn.host_producer_handle) builds a
`FidiusStreamHandle` from an iterator of bincode-encoded items; the guest's
`HostStream<T>` pulls them by calling `next`. Reusing the same handle struct
keeps both stream directions on one ABI.

## Functions

### <span id="fn.host_producer_handle"></span><code style="font-size: 0.85em">fn host_​producer_​handle</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn host_producer_handle(
    items: impl Iterator<Item = Vec<u8>> + Send + 'static,
) -> *mut FidiusStreamHandle
```

Build a `FidiusStreamHandle` the guest can pull, from an iterator of **pre-encoded** items (raw path). The returned handle is owned by the guest consumer, which frees it via `drop_fn`.

<details>
<summary>Source</summary>

```rust
pub fn host_producer_handle(
    items: impl Iterator<Item = Vec<u8>> + Send + 'static,
) -> *mut FidiusStreamHandle {
    let mut items = items;
    build_handle(Box::new(move || items.next().map(Ok)))
}
```

</details>

### <span id="fn.host_producer_handle_typed"></span><code style="font-size: 0.85em">fn host_​producer_​handle_​typed</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn host_producer_handle_typed<I: serde::Serialize + 'static>(
    items: impl Iterator<Item = I> + Send + 'static,
) -> *mut FidiusStreamHandle
```

Like [`host_producer_handle`](../../../rust/fidius-host/client_stream/#fn.host_producer_handle) but takes a **typed** item iterator and bincode-encodes each item **lazily** — only when the guest pulls it (FIDIUS-T-0172). This is the path the typed `PluginHandle::call_client_streaming` / `call_bidi_streaming` use, so an unbounded input iterator flows with bounded host memory. An item that fails to encode surfaces as a serialization error to the guest call (no panic across the FFI).

<details>
<summary>Source</summary>

```rust
pub fn host_producer_handle_typed<I: serde::Serialize + 'static>(
    items: impl Iterator<Item = I> + Send + 'static,
) -> *mut FidiusStreamHandle {
    let mut items = items;
    build_handle(Box::new(move || {
        items
            .next()
            .map(|i| fidius_core::wire::serialize(&i).map_err(|_| ()))
    }))
}
```

</details>
