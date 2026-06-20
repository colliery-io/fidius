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
use core::marker::PhantomData;

use serde::de::DeserializeOwned;
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

/// Guest-side **consumer** of a host-produced stream — the client-streaming
/// counterpart of [`StreamState`] (FIDIUS-I-0030 / ADR-0007). For a method that
/// takes a `Stream<T>` argument, the host fills a [`FidiusStreamHandle`] from its
/// producer and the guest pulls items by calling `next` and bincode-deserializing
/// each into `T`. Yields items via [`Iterator`]; dropping it runs the host
/// producer's `drop_fn` (cancel).
pub struct HostStream<T> {
    handle: *mut FidiusStreamHandle,
    /// Grow hint carried across calls so a `TooSmall` retry uses a big-enough buffer.
    cap: usize,
    _marker: PhantomData<T>,
}

impl<T: DeserializeOwned> HostStream<T> {
    /// Wrap a host-provided handle. The handle is owned by this consumer and freed
    /// (via `drop_fn`) on drop.
    ///
    /// # Safety
    /// `handle` must be a valid, exclusively-owned `FidiusStreamHandle` supplied by
    /// the host for the duration of this consumer.
    pub unsafe fn from_handle(handle: *mut FidiusStreamHandle) -> Self {
        Self {
            handle,
            cap: 256,
            _marker: PhantomData,
        }
    }

    fn pull(&mut self) -> Option<T> {
        let mut buf = vec![0u8; self.cap];
        loop {
            let mut out_len: u32 = 0;
            // SAFETY: `handle` is valid; `buf` has `self.cap` writable bytes; `next`
            // writes at most `cap` and reports the count in `out_len`.
            let status = unsafe {
                ((*self.handle).next)(self.handle, buf.as_mut_ptr(), self.cap as u32, &mut out_len)
            };
            match status {
                crate::status::STATUS_OK => {
                    return crate::wire::deserialize::<T>(&buf[..out_len as usize]).ok();
                }
                crate::status::STATUS_BUFFER_TOO_SMALL => {
                    self.cap = (out_len as usize).max(self.cap * 2);
                    buf = vec![0u8; self.cap];
                }
                // STREAM_END / PLUGIN_ERROR → end of stream for the consumer.
                _ => return None,
            }
        }
    }
}

impl<T: DeserializeOwned> Iterator for HostStream<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.pull()
    }
}

// SAFETY: a `HostStream` owns its handle and is consumed on a single thread within
// one client-streaming method call (the host never shares the handle concurrently).
// The `Send` bound lets the macro wrap it in a `Stream<T>` (whose iterator is
// `Send`) for the user method to consume.
unsafe impl<T> Send for HostStream<T> {}

impl<T> Drop for HostStream<T> {
    fn drop(&mut self) {
        // SAFETY: the handle is valid + owned; `drop_fn` is called exactly once.
        unsafe { ((*self.handle).drop_fn)(self.handle) };
    }
}

#[cfg(test)]
mod host_stream_tests {
    use super::*;

    struct MockProducer {
        items: Vec<u64>,
        idx: usize,
    }

    unsafe extern "C" fn mock_next(
        h: *mut FidiusStreamHandle,
        buf: *mut u8,
        cap: u32,
        out_len: *mut u32,
    ) -> i32 {
        let p = &mut *((*h).state as *mut MockProducer);
        if p.idx >= p.items.len() {
            return crate::status::STATUS_STREAM_END;
        }
        let bytes = crate::wire::serialize(&p.items[p.idx]).unwrap();
        if bytes.len() > cap as usize {
            *out_len = bytes.len() as u32;
            return crate::status::STATUS_BUFFER_TOO_SMALL;
        }
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
        *out_len = bytes.len() as u32;
        p.idx += 1;
        crate::status::STATUS_OK
    }

    unsafe extern "C" fn mock_drop(h: *mut FidiusStreamHandle) {
        drop(Box::from_raw((*h).state as *mut MockProducer));
        drop(Box::from_raw(h));
    }

    fn mock_handle(items: Vec<u64>) -> *mut FidiusStreamHandle {
        let producer = Box::into_raw(Box::new(MockProducer { items, idx: 0 }));
        Box::into_raw(Box::new(FidiusStreamHandle {
            next: mock_next,
            drop_fn: mock_drop,
            state: producer as *mut c_void,
        }))
    }

    #[test]
    fn host_stream_consumes_all_items_then_drops_cleanly() {
        let h = mock_handle(vec![10u64, 20, 30]);
        // SAFETY: `h` is a freshly-built, exclusively-owned handle.
        let consumer = unsafe { HostStream::<u64>::from_handle(h) };
        let got: Vec<u64> = consumer.collect();
        assert_eq!(got, vec![10, 20, 30]);
        // Dropping `consumer` ran `mock_drop` (freed producer + handle) — under
        // Miri/ASAN this would catch a leak or double-free.
    }
}
