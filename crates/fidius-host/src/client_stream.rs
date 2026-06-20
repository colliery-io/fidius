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

//! Host-side **producer** for client-streaming (FIDIUS-I-0030 / ADR-0007).
//!
//! The inverse of the server-streaming `StreamState`: here the **host** produces
//! and the **guest** consumes. [`host_producer_handle`] builds a
//! [`FidiusStreamHandle`] from an iterator of bincode-encoded items; the guest's
//! `HostStream<T>` pulls them by calling `next`. Reusing the same handle struct
//! keeps both stream directions on one ABI.

use std::ffi::c_void;

use fidius_core::status::{STATUS_BUFFER_TOO_SMALL, STATUS_OK, STATUS_STREAM_END};
use fidius_core::stream_ffi::FidiusStreamHandle;

/// Boxed producer state: an iterator of pre-encoded items plus a held-back
/// `pending` item, so a `BUFFER_TOO_SMALL` retry re-delivers the same item
/// instead of dropping it (mirrors `StreamState`).
struct ProducerState {
    items: Box<dyn Iterator<Item = Vec<u8>> + Send>,
    pending: Option<Vec<u8>>,
}

/// The `next` callback the guest invokes: deliver one item into the guest buffer.
unsafe extern "C" fn producer_next(
    h: *mut FidiusStreamHandle,
    buf: *mut u8,
    cap: u32,
    out_len: *mut u32,
) -> i32 {
    let st = &mut *((*h).state as *mut ProducerState);
    if st.pending.is_none() {
        match st.items.next() {
            Some(bytes) => st.pending = Some(bytes),
            None => return STATUS_STREAM_END,
        }
    }
    let bytes = st.pending.as_ref().unwrap();
    if bytes.len() > cap as usize {
        *out_len = bytes.len() as u32;
        return STATUS_BUFFER_TOO_SMALL;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, bytes.len());
    *out_len = bytes.len() as u32;
    st.pending = None;
    STATUS_OK
}

/// Finish/cancel: free the producer state + the handle box. Called once by the
/// guest consumer's `Drop`.
unsafe extern "C" fn producer_drop(h: *mut FidiusStreamHandle) {
    drop(Box::from_raw((*h).state as *mut ProducerState));
    drop(Box::from_raw(h));
}

/// Build a `FidiusStreamHandle` the guest can pull, from an iterator of
/// bincode-encoded items. The returned handle is owned by the guest consumer,
/// which frees it via `drop_fn`. The host hands the raw pointer to a
/// client-streaming method call.
pub fn host_producer_handle(
    items: impl Iterator<Item = Vec<u8>> + Send + 'static,
) -> *mut FidiusStreamHandle {
    let st = Box::into_raw(Box::new(ProducerState {
        items: Box::new(items),
        pending: None,
    }));
    Box::into_raw(Box::new(FidiusStreamHandle {
        next: producer_next,
        drop_fn: producer_drop,
        state: st as *mut c_void,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fidius_core::stream_ffi::HostStream;

    #[test]
    fn host_producer_feeds_guest_consumer() {
        // The host producer + the guest consumer round-trip the items end to end —
        // the core client-streaming pull mechanism, in-process (no dylib/macro yet).
        let items: Vec<u64> = vec![1, 2, 3, 4];
        let encoded: Vec<Vec<u8>> = items
            .iter()
            .map(|i| fidius_core::wire::serialize(i).unwrap())
            .collect();
        let handle = host_producer_handle(encoded.into_iter());
        // SAFETY: freshly-built, exclusively-owned handle.
        let consumer = unsafe { HostStream::<u64>::from_handle(handle) };
        let got: Vec<u64> = consumer.collect();
        assert_eq!(got, items);
    }
}
