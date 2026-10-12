---
title: "fidius-guest::frame"
description: "Streaming frame format for the server-streaming plugin boundary (FIDIUS-I-0026, design decision D2)."
editUrl: false
sidebar:
  order: 13
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

- [`FRAME_ITEM`](../../../rust/fidius-guest/frame/#const.FRAME_ITEM) — one streamed item; `payload` is the existing bincode/`Value`
  wire bytes, identical to a unary return value. Streaming therefore inherits
  the unary wire format verbatim (one item == one unary payload).
- [`FRAME_END`](../../../rust/fidius-guest/frame/#const.FRAME_END) — clean end of stream; `len == 0`, no payload.
- [`FRAME_ERROR`](../../../rust/fidius-guest/frame/#const.FRAME_ERROR) — the producer failed mid-stream; `payload` is a bincode
  [`PluginError`](../../../rust/fidius-guest/error/#struct.PluginError). An error frame *terminates* the stream: no frames may
  follow it (enforced by the host-side decoder, [`crate::frame`](../../../rust/fidius-guest/frame/) consumers).

This framing is identical across every backend — a WIT `list<u8>` returned
per `next()` (WASM), a `bytes` object yielded by a generator (Python), or a
host-allocated buffer filled by an FFI `next()` export (cdylib). The scheme
is deliberately batch-compatible: a future `next_batch(n)` (design decision
D5) is simply *n* `ITEM` frames concatenated, needing no wire change.

## Enums

### <span id="enum.Frame"></span><code style="font-size: 0.85em">enum Frame</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

One frame crossing the streaming boundary.

`Item` carries opaque payload bytes (the caller decides how to decode them —
typically `wire::deserialize::<T>()` or a `Value`), so this type stays
backend- and schema-neutral.

#### Variants

- **`Item`** - One streamed item; bytes are the bincode/`Value` payload.
- **`End`** - Clean end of stream.
- **`Error`** - Producer failed mid-stream; the stream terminates here.

#### Methods

##### <span id="Frame.encode"></span><code>encode</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn encode(&self) -> Result<Vec<u8>, WireError>
```

Encode this frame as `[tag][len][payload]`.

<details>
<summary>Source</summary>

```rust
    pub fn encode(&self) -> Result<Vec<u8>, WireError> {
        let (tag, payload): (u8, Vec<u8>) = match self {
            Frame::Item(bytes) => (FRAME_ITEM, bytes.clone()),
            Frame::End => (FRAME_END, Vec::new()),
            Frame::Error(err) => (FRAME_ERROR, wire::serialize(err)?),
        };
        let mut out = Vec::with_capacity(FRAME_HEADER_LEN + payload.len());
        out.push(tag);
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&payload);
        Ok(out)
    }
```

</details>

##### <span id="Frame.decode"></span><code>decode</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn decode(bytes: &[u8]) -> Result<Frame, FrameError>
```

Decode exactly one frame from `bytes`, which must contain a single frame and nothing more. Use [`Frame::read`](../../../rust/fidius-guest/frame/#Frame.read) to pull one frame from a larger buffer and learn how many bytes were consumed.

<details>
<summary>Source</summary>

```rust
    pub fn decode(bytes: &[u8]) -> Result<Frame, FrameError> {
        let (frame, consumed) = Frame::read(bytes)?;
        if consumed != bytes.len() {
            return Err(FrameError::Malformed(format!(
                "{} trailing byte(s) after frame",
                bytes.len() - consumed
            )));
        }
        Ok(frame)
    }
```

</details>

##### <span id="Frame.read"></span><code>read</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn read(bytes: &[u8]) -> Result<(Frame, usize), FrameError>
```

Read one frame from the front of `bytes`, returning the frame and the number of bytes consumed. Lets a decoder walk a buffer holding several concatenated frames (the cdylib `next_batch` / `Value`-rail path).

<details>
<summary>Source</summary>

```rust
    pub fn read(bytes: &[u8]) -> Result<(Frame, usize), FrameError> {
        if bytes.len() < FRAME_HEADER_LEN {
            return Err(FrameError::Truncated {
                needed: FRAME_HEADER_LEN,
                had: bytes.len(),
            });
        }
        let tag = bytes[0];
        let len = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
        let end = FRAME_HEADER_LEN + len;
        if bytes.len() < end {
            return Err(FrameError::Truncated {
                needed: end,
                had: bytes.len(),
            });
        }
        let payload = &bytes[FRAME_HEADER_LEN..end];
        let frame = match tag {
            FRAME_ITEM => Frame::Item(payload.to_vec()),
            FRAME_END => {
                if len != 0 {
                    return Err(FrameError::Malformed(format!(
                        "END frame carries {len} payload byte(s)"
                    )));
                }
                Frame::End
            }
            FRAME_ERROR => Frame::Error(wire::deserialize::<PluginError>(payload)?),
            other => return Err(FrameError::UnknownTag(other)),
        };
        Ok((frame, end))
    }
```

</details>

### <span id="enum.FrameError"></span><code style="font-size: 0.85em">enum FrameError</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

Errors decoding a [`Frame`](../../../rust/fidius-guest/frame/#enum.Frame) from bytes.

#### Variants

- **`Truncated`** - Buffer ended before a full header or the declared payload was read.
- **`UnknownTag`** - The tag byte was not one of the known frame tags.
- **`Payload`** - The `ERROR` frame payload failed to decode as a `PluginError`.
- **`Malformed`** - An `END` or `ERROR` frame declared a payload length it must not have, or
  trailing bytes followed a terminal frame in a single-frame decode.

## Constants

| Name | Type | Value | Description |
|------|------|-------|-------------|
| <span id="const.FRAME_ITEM"></span>`FRAME_ITEM` | `u8` | `0` | Frame tag: one streamed item. Payload is bincode/`Value` wire bytes. |
| <span id="const.FRAME_END"></span>`FRAME_END` | `u8` | `1` | Frame tag: clean end of stream. No payload. |
| <span id="const.FRAME_ERROR"></span>`FRAME_ERROR` | `u8` | `2` | Frame tag: producer error. Payload is a bincode [`PluginError`](../../../rust/fidius-guest/error/#struct.PluginError). |
| <span id="const.FRAME_HEADER_LEN"></span>`FRAME_HEADER_LEN` | `usize` | `1 + 4` | Fixed size of a frame header: one tag byte plus a `u32` length. |
