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

//! Thread-local arena pool for Arena-strategy plugin invocations.
//!
//! Plugin methods built with `buffer = Arena` write their serialized output
//! into a host-provided buffer rather than allocating on each call. This
//! module supplies those buffers from a per-thread pool so repeated calls
//! reuse memory instead of reallocating.

use std::cell::RefCell;

/// Default initial arena capacity (4 KB) when the pool is empty and a
/// caller hasn't indicated a larger minimum. Most serialized outputs fit
/// comfortably within this; larger ones grow via the retry path.
pub const DEFAULT_ARENA_CAPACITY: usize = 4096;

thread_local! {
    /// Pool of reusable arena buffers for the current thread. Acquired on
    /// entry to an Arena-strategy call, released on exit.
    static ARENA_POOL: RefCell<Vec<Vec<u8>>> = const { RefCell::new(Vec::new()) };
}

/// Acquire an arena buffer with at least `min_capacity` bytes. Prefers to
/// reuse a pooled buffer if one is available; otherwise allocates a fresh
/// `Vec<u8>`. The returned buffer is filled with zero bytes up to its
/// length (which equals its capacity after this call) — callers write into
/// it via raw pointers and track their own output length.
pub fn acquire_arena(min_capacity: usize) -> Vec<u8> {
    let target = min_capacity.max(DEFAULT_ARENA_CAPACITY);
    ARENA_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        if let Some(mut buf) = pool.pop() {
            if buf.capacity() < target {
                let extra = target - buf.capacity();
                buf.reserve_exact(extra);
            }
            // Fill the available capacity with zeros so the raw mut-slice
            // view aligns with the Vec's len invariant.
            let cap = buf.capacity();
            buf.clear();
            buf.resize(cap, 0);
            buf
        } else {
            vec![0u8; target]
        }
    })
}

/// Return an arena buffer to the pool for future reuse. The buffer's
/// capacity is retained; length is irrelevant (callers set length via
/// direct writes, not Vec::extend).
pub fn release_arena(buf: Vec<u8>) {
    ARENA_POOL.with(|pool| pool.borrow_mut().push(buf));
}

/// Grow an in-flight arena buffer to hold at least `needed_capacity` bytes.
/// Used by the retry path when a plugin returns `STATUS_BUFFER_TOO_SMALL`.
pub fn grow_arena(buf: &mut Vec<u8>, needed_capacity: usize) {
    if buf.capacity() < needed_capacity {
        let extra = needed_capacity - buf.capacity();
        buf.reserve_exact(extra);
    }
    let cap = buf.capacity();
    buf.clear();
    buf.resize(cap, 0);
}
