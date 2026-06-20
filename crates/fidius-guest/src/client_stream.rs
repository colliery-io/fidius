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

//! Guest-side consumer for **client-streaming over WASM** (FIDIUS-I-0030 CS2.3).
//!
//! WASM can't share a raw `FidiusStreamHandle` across the sandbox (no shared
//! memory like cdylib), so the guest **imports** a host function and pulls through
//! it. The component imports `fidius:stream-pull/pull.next() -> option<list<u8>>`;
//! [`WasmHostStream`] presents the bincode items the host produces as an
//! `Iterator`, which the macro wraps in the `Stream<T>` the method consumes. This
//! `generate!` composes with the macro's interface-export `generate!` exactly like
//! `fidius_guest::http`'s wasi:http import (I-0028).

#![cfg(target_family = "wasm")]

use core::marker::PhantomData;

use serde::de::DeserializeOwned;

#[allow(warnings)]
mod bindings {
    wit_bindgen::generate!({
        inline: "package fidius:stream-pull@0.1.0;\n\
                 interface pull {\n\
                 \x20 /// Pull the next item (bincode); `none` = clean end of stream.\n\
                 \x20 next: func() -> option<list<u8>>;\n\
                 }\n\
                 world stream-pull-client {\n\
                 \x20 import pull;\n\
                 }",
        world: "stream-pull-client",
    });
}

/// Guest consumer of a host-produced stream — the client-streaming `Stream<T>`
/// argument. Each `next()` pulls one bincode item via the imported host func and
/// deserializes it into `T`; `None` from the import = end of stream.
pub struct WasmHostStream<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> WasmHostStream<T> {
    /// Construct a consumer over the host's `fidius:stream-pull` import.
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for WasmHostStream<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: DeserializeOwned> Iterator for WasmHostStream<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match bindings::fidius::stream_pull::pull::next() {
            Some(bytes) => crate::wire::deserialize::<T>(&bytes).ok(),
            None => None,
        }
    }
}

// SAFETY: `WasmHostStream` holds no shared state (the import is global to the
// single-threaded component instance); the `Send` bound lets the macro wrap it in
// a `Stream<T>` for the user method to consume.
unsafe impl<T> Send for WasmHostStream<T> {}
