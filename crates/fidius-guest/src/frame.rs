// Copyright 2026 Colliery, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Streaming frame format for the server-streaming plugin boundary
//! (FIDIUS-I-0026, design decision D2).
//!
//! A streaming method call yields a *sequence of frames* rather than one buffer.
//! Each frame is length-delimited and self-identifying:
//!
//! ```text
//! [ tag: u8 ][ len: u32 little-endian ][ payload: len bytes ]
//! ```
//!
//! Three tags exist:
//!
//! - [`FRAME_ITEM`] — one streamed item; `payload` is the existing bincode/`Value`
//!   wire bytes, identical to a unary return value. Streaming therefore inherits
//!   the unary wire format verbatim (one item == one unary payload).
//! - [`FRAME_END`] — clean end of stream; `len == 0`, no payload.
//! - [`FRAME_ERROR`] — the producer failed mid-stream; `payload` is a bincode
//!   [`PluginError`]. An error frame *terminates* the stream: no frames may
//!   follow it (enforced by the host-side decoder, [`crate::frame`] consumers).
//!
//! This framing is identical across every backend — a WIT `list<u8>` returned
//! per `next()` (WASM), a `bytes` object yielded by a generator (Python), or a
//! host-allocated buffer filled by an FFI `next()` export (cdylib). The scheme
//! is deliberately batch-compatible: a future `next_batch(n)` (design decision
//! D5) is simply *n* `ITEM` frames concatenated, needing no wire change.

use crate::error::PluginError;
use crate::wire::{self, WireError};

/// Frame tag: one streamed item. Payload is bincode/`Value` wire bytes.
pub const FRAME_ITEM: u8 = 0;
/// Frame tag: clean end of stream. No payload.
pub const FRAME_END: u8 = 1;
/// Frame tag: producer error. Payload is a bincode [`PluginError`].
pub const FRAME_ERROR: u8 = 2;

/// Fixed size of a frame header: one tag byte plus a `u32` length.
pub const FRAME_HEADER_LEN: usize = 1 + 4;

/// One frame crossing the streaming boundary.
///
/// `Item` carries opaque payload bytes (the caller decides how to decode them —
/// typically `wire::deserialize::<T>()` or a `Value`), so this type stays
/// backend- and schema-neutral.
#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    /// One streamed item; bytes are the bincode/`Value` payload.
    Item(Vec<u8>),
    /// Clean end of stream.
    End,
    /// Producer failed mid-stream; the stream terminates here.
    Error(PluginError),
}

/// Errors decoding a [`Frame`] from bytes.
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    /// Buffer ended before a full header or the declared payload was read.
    #[error("truncated frame: needed {needed} bytes, had {had}")]
    Truncated { needed: usize, had: usize },

    /// The tag byte was not one of the known frame tags.
    #[error("unknown frame tag: {0}")]
    UnknownTag(u8),

    /// The `ERROR` frame payload failed to decode as a `PluginError`.
    #[error("malformed error-frame payload: {0}")]
    Payload(#[from] WireError),

    /// An `END` or `ERROR` frame declared a payload length it must not have, or
    /// trailing bytes followed a terminal frame in a single-frame decode.
    #[error("malformed frame: {0}")]
    Malformed(String),
}

impl Frame {
    /// Encode this frame as `[tag][len][payload]`.
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

    /// Decode exactly one frame from `bytes`, which must contain a single frame
    /// and nothing more. Use [`Frame::read`] to pull one frame from a larger
    /// buffer and learn how many bytes were consumed.
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

    /// Read one frame from the front of `bytes`, returning the frame and the
    /// number of bytes consumed. Lets a decoder walk a buffer holding several
    /// concatenated frames (the cdylib `next_batch` / `Value`-rail path).
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(payload: &[u8]) -> Frame {
        Frame::Item(payload.to_vec())
    }

    #[test]
    fn item_round_trip() {
        let f = item(&[1, 2, 3, 4]);
        let bytes = f.encode().unwrap();
        assert_eq!(bytes[0], FRAME_ITEM);
        assert_eq!(Frame::decode(&bytes).unwrap(), f);
    }

    #[test]
    fn end_round_trip() {
        let bytes = Frame::End.encode().unwrap();
        assert_eq!(bytes[0], FRAME_END);
        assert_eq!(bytes.len(), FRAME_HEADER_LEN);
        assert_eq!(Frame::decode(&bytes).unwrap(), Frame::End);
    }

    #[test]
    fn error_round_trip() {
        let err = PluginError::new("BOOM", "it broke");
        let f = Frame::Error(err.clone());
        let bytes = f.encode().unwrap();
        assert_eq!(bytes[0], FRAME_ERROR);
        assert_eq!(Frame::decode(&bytes).unwrap(), Frame::Error(err));
    }

    #[test]
    fn empty_item_is_valid() {
        let f = item(&[]);
        let bytes = f.encode().unwrap();
        assert_eq!(Frame::decode(&bytes).unwrap(), f);
    }

    #[test]
    fn read_walks_concatenated_frames() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&item(&[9]).encode().unwrap());
        buf.extend_from_slice(&item(&[8, 7]).encode().unwrap());
        buf.extend_from_slice(&Frame::End.encode().unwrap());

        let (f1, n1) = Frame::read(&buf).unwrap();
        assert_eq!(f1, item(&[9]));
        let (f2, n2) = Frame::read(&buf[n1..]).unwrap();
        assert_eq!(f2, item(&[8, 7]));
        let (f3, n3) = Frame::read(&buf[n1 + n2..]).unwrap();
        assert_eq!(f3, Frame::End);
        assert_eq!(n1 + n2 + n3, buf.len());
    }

    #[test]
    fn truncated_header_is_rejected() {
        let err = Frame::read(&[FRAME_ITEM, 0, 0]).unwrap_err();
        assert!(matches!(err, FrameError::Truncated { .. }));
    }

    #[test]
    fn truncated_payload_is_rejected() {
        // Declares 8 payload bytes but only supplies 2.
        let mut bytes = vec![FRAME_ITEM];
        bytes.extend_from_slice(&8u32.to_le_bytes());
        bytes.extend_from_slice(&[1, 2]);
        let err = Frame::decode(&bytes).unwrap_err();
        assert!(matches!(err, FrameError::Truncated { needed: 13, had: 7 }));
    }

    #[test]
    fn unknown_tag_is_rejected() {
        let mut bytes = vec![99u8];
        bytes.extend_from_slice(&0u32.to_le_bytes());
        assert!(matches!(
            Frame::decode(&bytes).unwrap_err(),
            FrameError::UnknownTag(99)
        ));
    }

    #[test]
    fn end_with_payload_is_rejected() {
        let mut bytes = vec![FRAME_END];
        bytes.extend_from_slice(&3u32.to_le_bytes());
        bytes.extend_from_slice(&[1, 2, 3]);
        assert!(matches!(
            Frame::decode(&bytes).unwrap_err(),
            FrameError::Malformed(_)
        ));
    }

    #[test]
    fn trailing_bytes_after_single_decode_rejected() {
        let mut bytes = item(&[1]).encode().unwrap();
        bytes.push(0xFF);
        assert!(matches!(
            Frame::decode(&bytes).unwrap_err(),
            FrameError::Malformed(_)
        ));
    }

    #[test]
    fn garbage_is_rejected_not_panicking() {
        for g in [vec![], vec![0u8], vec![2u8, 0, 0, 0, 1]] {
            let _ = Frame::read(&g); // must return Err/Ok, never panic
        }
    }
}
