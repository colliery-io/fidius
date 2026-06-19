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

//! cdylib server-streaming FFI handle (FIDIUS-I-0026 Phase 3 / CS.1, T-0138).
//!
//! A `-> fidius::Stream<T>` method on the **cdylib** backend can't be a single
//! unary call. Instead its vtable slot holds an *init* shim (with the ordinary
//! `(in_ptr, in_len, *mut *mut u8, *mut u32) -> i32` signature, so the vtable
//! layout is unchanged) that returns — via the standard `out_ptr` slot — a
//! pointer to a [`FidiusStreamHandle`]. The host then pulls the stream with an
//! **arena-style `next`** (FIDIUS-T-0138): the host passes a *reusable* buffer it
//! owns, and the guest writes the bincode-encoded item into it — no per-item
//! heap alloc, and no `free_buffer` FFI crossing.
//!
//! ```text
//! status = (handle.next)(handle, buf_ptr, buf_cap, &mut out_len);
//!   STATUS_OK                → one item: buf[..out_len] is the bincode-encoded item.
//!   STATUS_STREAM_END        → clean end of stream (no item).
//!   STATUS_BUFFER_TOO_SMALL  → out_len = required size; host grows + retries once.
//!                              (The guest holds the serialized item across the
//!                              retry, so no item is lost.)
//!   STATUS_PLUGIN_ERROR      → buf[..out_len] is a bincode `PluginError`.
//!   STATUS_SERIALIZATION_ERROR / STATUS_PANIC → as usual.
//! (handle.drop_fn)(handle);  // run once: drops the producer + frees the handle
//! ```
//!
//! Items cross as **concrete bincode of the item type** — byte-identical to the
//! unary cdylib wire (FIDIUS-T-0137). The host decodes each item with a
//! caller-supplied `bincode::<O>` decoder (the typed Client knows `O`).

use core::ffi::c_void;

use serde::Serialize;

/// Per-stream handle returned by a cdylib streaming method's init shim. See the
/// module docs for the pull protocol. `#[repr(C)]` so the guest (which builds it)
/// and the host (which drives it) agree on the layout across the FFI boundary.
#[repr(C)]
pub struct FidiusStreamHandle {
    /// Advance one item into a host-provided buffer.
    /// `(handle, buf_ptr, buf_cap, &mut out_len) -> status`.
    pub next: unsafe extern "C" fn(*mut FidiusStreamHandle, *mut u8, u32, *mut u32) -> i32,
    /// Finish/cancel: drops the producer and frees the handle box. Call exactly once.
    pub drop_fn: unsafe extern "C" fn(*mut FidiusStreamHandle),
    /// Opaque pointer to the boxed [`StreamState<T>`], owned by the guest; freed
    /// by `drop_fn`.
    pub state: *mut c_void,
}

/// Outcome of [`StreamState::next_into`] — mapped to FFI status codes by the
/// macro-generated `next` shim.
pub enum NextStatus {
    /// An item was written; payload is `buf[..n]`.
    Item(usize),
    /// No more items.
    End,
    /// The buffer was too small; `usize` is the size required. The item is
    /// retained for the next call (no item lost on grow-and-retry).
    TooSmall(usize),
    /// Serializing the item failed.
    SerErr,
}

/// Guest-side driver for an arena-style cdylib stream (FIDIUS-T-0138). Wraps the
/// producer [`Stream<T>`](crate::stream_marker::Stream) and holds the *current
/// item value* (not its bytes), so a `BUFFER_TOO_SMALL` retry re-serializes the
/// same item rather than advancing the iterator (which would drop it) — and so
/// the happy path serializes **directly into the host buffer with no `Vec`
/// allocation**.
pub struct StreamState<T> {
    stream: crate::stream_marker::Stream<T>,
    /// The current item awaiting delivery; `None` between items.
    pending: Option<T>,
}

impl<T: Serialize> StreamState<T> {
    /// Wrap a producer stream.
    pub fn new(stream: crate::stream_marker::Stream<T>) -> Self {
        Self {
            stream,
            pending: None,
        }
    }

    /// Pull the next item (if needed) and serialize it **directly into `buf`** —
    /// no intermediate allocation. On `TooSmall` the item value is retained and
    /// re-serialized on the next (larger-buffer) call, so nothing is lost.
    pub fn next_into(&mut self, buf: &mut [u8]) -> NextStatus {
        if self.pending.is_none() {
            match self.stream.next_item() {
                Some(item) => self.pending = Some(item),
                None => return NextStatus::End,
            }
        }
        // SAFETY of unwrap: pending is Some by the block above.
        let item = self.pending.as_ref().unwrap();
        let size = match crate::wire::serialized_size(item) {
            Ok(s) => s as usize,
            Err(_) => return NextStatus::SerErr,
        };
        if size > buf.len() {
            return NextStatus::TooSmall(size);
        }
        if crate::wire::serialize_into(&mut buf[..size], item).is_err() {
            return NextStatus::SerErr;
        }
        self.pending = None;
        NextStatus::Item(size)
    }
}
