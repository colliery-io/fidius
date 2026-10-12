---
title: "fidius-guest::frame"
description: "Streaming frame format for the server-streaming plugin boundary (FIDIUS-I-0026, design decision D2)."
sidebar:
  order: 14
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


Streaming frame format for the server-streaming plugin boundary (FIDIUS-I-0026, design decision D2).

A streaming method call yields a *sequence of frames* rather than one buffer.
Each frame is length-delimited and self-identifying:
```text
[ tag: u8 ][ len: u32 little-endian ][ payload: len bytes ]
```
Three tags exist:
- [`FRAME_ITEM`] — one streamed item; `payload` is the existing bincode/`Value`
wire bytes, identical to a unary return value. Streaming therefore inherits
the unary wire format verbatim (one item == one unary payload).
- [`FRAME_END`] — clean end of stream; `len == 0`, no payload.
- [`FRAME_ERROR`] — the producer failed mid-stream; `payload` is a bincode
[`PluginError`]. An error frame *terminates* the stream: no frames may
follow it (enforced by the host-side decoder, [`crate::frame`] consumers).
This framing is identical across every backend — a WIT `list<u8>` returned
per `next()` (WASM), a `bytes` object yielded by a generator (Python), or a
host-allocated buffer filled by an FFI `next()` export (cdylib). The scheme
is deliberately batch-compatible: a future `next_batch(n)` (design decision
D5) is simply *n* `ITEM` frames concatenated, needing no wire change.

## Enums

### <span id="enum.Frame"></span>`enum Frame`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


One frame crossing the streaming boundary.

`Item` carries opaque payload bytes (the caller decides how to decode them —
typically `wire::deserialize::<T>()` or a `Value`), so this type stays
backend- and schema-neutral.

#### Variants

- **`Item`** - One streamed item; bytes are the bincode/`Value` payload.
- **`End`** - Clean end of stream.
- **`Error`** - Producer failed mid-stream; the stream terminates here.



### <span id="enum.FrameError"></span>`enum FrameError`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


Errors decoding a [`Frame`] from bytes.

#### Variants

- **`Truncated`** - Buffer ended before a full header or the declared payload was read.
- **`UnknownTag`** - The tag byte was not one of the known frame tags.
- **`Payload`** - The `ERROR` frame payload failed to decode as a `PluginError`.
- **`Malformed`** - An `END` or `ERROR` frame declared a payload length it must not have, or
trailing bytes followed a terminal frame in a single-frame decode.



