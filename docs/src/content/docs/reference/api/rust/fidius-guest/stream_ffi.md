---
title: "fidius-guest::stream_ffi"
description: "cdylib server-streaming FFI handle (FIDIUS-I-0026 Phase 3 / CS.1, T-0138)."
sidebar:
  order: 22
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


cdylib server-streaming FFI handle (FIDIUS-I-0026 Phase 3 / CS.1, T-0138).

A `-> fidius::Stream<T>` method on the **cdylib** backend can't be a single
unary call. Instead its vtable slot holds an *init* shim (with the ordinary
`(in_ptr, in_len, *mut *mut u8, *mut u32) -> i32` signature, so the vtable
layout is unchanged) that returns — via the standard `out_ptr` slot — a
pointer to a [`FidiusStreamHandle`]. The host then pulls the stream with an
**arena-style `next`** (FIDIUS-T-0138): the host passes a *reusable* buffer it
owns, and the guest writes the bincode-encoded item into it — no per-item
heap alloc, and no `free_buffer` FFI crossing.
```text
status = (handle.next)(handle, buf_ptr, buf_cap, &mut out_len);
STATUS_OK                → one item: buf[..out_len] is the bincode-encoded item.
STATUS_STREAM_END        → clean end of stream (no item).
STATUS_BUFFER_TOO_SMALL  → out_len = required size; host grows + retries once.
(The guest holds the serialized item across the
retry, so no item is lost.)
STATUS_PLUGIN_ERROR      → buf[..out_len] is a bincode `PluginError`.
STATUS_SERIALIZATION_ERROR / STATUS_PANIC → as usual.
(handle.drop_fn)(handle);  // run once: drops the producer + frees the handle
```
Items cross as **concrete bincode of the item type** — byte-identical to the
unary cdylib wire (FIDIUS-T-0137). The host decodes each item with a
caller-supplied `bincode::<O>` decoder (the typed Client knows `O`).

## Structs

### <span id="struct.FidiusStreamHandle"></span>`struct FidiusStreamHandle`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


Per-stream handle returned by a cdylib streaming method's init shim. See the module docs for the pull protocol. `#[repr(C)]` so the guest (which builds it) and the host (which drives it) agree on the layout across the FFI boundary.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `next` | `unsafe extern "C" fn (* mut FidiusStreamHandle , * mut u8 , u32 , * mut u32) -> i32` | Advance one item into a host-provided buffer.
`(handle, buf_ptr, buf_cap, &mut out_len) -> status`. |
| `drop_fn` | `unsafe extern "C" fn (* mut FidiusStreamHandle)` | Finish/cancel: drops the producer and frees the handle box. Call exactly once. |
| `state` | `* mut c_void` | Opaque pointer to the boxed [`StreamState<T>`], owned by the guest; freed
by `drop_fn`. |



### <span id="struct.StreamState"></span>`struct StreamState<T>`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


Guest-side driver for an arena-style cdylib stream (FIDIUS-T-0138). Wraps the producer [`Stream<T>`](crate::stream_marker::Stream) and holds the *current item value* (not its bytes), so a `BUFFER_TOO_SMALL` retry re-serializes the same item rather than advancing the iterator (which would drop it) — and so the happy path serializes **directly into the host buffer with no `Vec` allocation**.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `stream` | `crate :: stream_marker :: Stream < T >` |  |
| `pending` | `Option < T >` | The current item awaiting delivery; `None` between items. |



### <span id="struct.HostStream"></span>`struct HostStream<T>`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


Guest-side **consumer** of a host-produced stream — the client-streaming counterpart of [`StreamState`] (FIDIUS-I-0030 / ADR-0007). For a method that takes a `Stream<T>` argument, the host fills a [`FidiusStreamHandle`] from its producer and the guest pulls items by calling `next` and bincode-deserializing each into `T`. Yields items via [`Iterator`]; dropping it runs the host producer's `drop_fn` (cancel).

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `handle` | `* mut FidiusStreamHandle` |  |
| `cap` | `usize` | Grow hint carried across calls so a `TooSmall` retry uses a big-enough buffer. |
| `_marker` | `PhantomData < T >` |  |



## Enums

### <span id="enum.NextStatus"></span>`enum NextStatus`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


Outcome of [`StreamState::next_into`] — mapped to FFI status codes by the macro-generated `next` shim.

#### Variants

- **`Item`** - An item was written; payload is `buf[..n]`.
- **`End`** - No more items.
- **`TooSmall`** - The buffer was too small; `usize` is the size required. The item is
retained for the next call (no item lost on grow-and-retry).
- **`SerErr`** - Serializing the item failed.



