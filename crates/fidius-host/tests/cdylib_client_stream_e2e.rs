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

//! cdylib **client-streaming** end to end (FIDIUS-I-0030 CS2.2): a method whose
//! `Stream<T>` argument is fed by the HOST. The host builds a producer handle, the
//! plugin pulls items from it (host produces, plugin consumes — the inverse of
//! server-streaming), and returns a value. Proves the `ClientStreamFn` vtable
//! shape + the macro shim + `HostStream`/`host_producer_handle` round-trip through
//! a real (in-process) plugin.

#![cfg(feature = "streaming")]
#![allow(unexpected_cfgs)]

use fidius_host::client_stream::host_producer_handle;
use fidius_host::PluginHandle;

#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Sink: Send + Sync {
    /// Client-streaming: the host produces `rows`; the plugin pulls + folds them.
    fn load(&self, rows: fidius_core::Stream<u64>) -> u64;
}

pub struct Summer;

#[fidius_macro::plugin_impl(Sink, crate = "fidius_core")]
impl Sink for Summer {
    fn load(&self, mut rows: fidius_core::Stream<u64>) -> u64 {
        let mut sum = 0u64;
        while let Some(x) = rows.next_item() {
            sum += x;
        }
        sum
    }
}

fidius_core::fidius_plugin_registry!();

#[test]
fn cdylib_consumes_a_host_produced_stream() {
    let desc = PluginHandle::find_in_process_descriptor("Summer").unwrap();
    let handle = PluginHandle::from_descriptor(desc).unwrap();

    // The host produces [1..=5]; the plugin pulls them lazily and sums → 15.
    let items: Vec<u64> = vec![1, 2, 3, 4, 5];
    let encoded: Vec<Vec<u8>> = items
        .iter()
        .map(|i| fidius_core::wire::serialize(i).unwrap())
        .collect();
    let producer = host_producer_handle(encoded.into_iter());
    let args = fidius_core::wire::serialize(&()).unwrap(); // no non-stream args

    // SAFETY: `producer` is a freshly-built, exclusively-owned producer handle.
    let out = unsafe { handle.call_client_streaming_raw(0, producer, &args) }
        .expect("client-streaming call");
    let sum: u64 = fidius_core::wire::deserialize(&out).unwrap();
    assert_eq!(sum, 15);
}
