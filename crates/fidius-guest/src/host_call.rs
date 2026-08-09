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

//! Guest-side consumer for **host functions over WASM** (the plugin → host
//! callback channel, wasm variant).
//!
//! WASM can't share a raw [`crate::host_ffi::HostFunctionTable`] across the
//! sandbox, so the guest **imports** a host function and dispatches through
//! it: `fidius:host-call/host.call(interface-name, expected-version,
//! expected-hash, index, args) -> (status, payload)`. The identity triple
//! travels with every call, so the host enforces the version/hash gate on
//! each call — the wasm counterpart of the dylib channel's bind-time gate,
//! with the same guarantee (typed loud failure, never a mis-dispatch of
//! positional bincode). This `generate!` composes with the macro's
//! interface-export `generate!` exactly like [`crate::client_stream`]'s
//! `fidius:stream-pull` import.

#![cfg(target_family = "wasm")]

use crate::host_ffi::{decode_host_call_status, HostCallError, HOST_CALL_PROBE_INDEX};

#[allow(warnings)]
mod bindings {
    wit_bindgen::generate!({
        inline: "package fidius:host-call@0.1.0;\n\
                 interface host {\n\
                 \x20 /// Invoke host function `index` of `interface-name`, gated on the\n\
                 \x20 /// version + signature hash the plugin was built against. Returns\n\
                 \x20 /// (status, payload) — see fidius host_ffi status codes.\n\
                 \x20 call: func(interface-name: string, expected-version: u32,\n\
                 \x20            expected-hash: u64, index: u32, args: list<u8>)\n\
                 \x20     -> tuple<s32, list<u8>>;\n\
                 }\n\
                 world host-call-client {\n\
                 \x20 import host;\n\
                 }",
        world: "host-call-client",
    });
}

/// Invoke a host function through the `fidius:host-call` import: bincode of
/// the argument tuple in, bincode of the result out, with the status protocol
/// mapped to [`HostCallError`]. Used by the wasm side of every generated
/// `<Trait>Client` method.
pub fn call(
    interface: &'static str,
    expected_version: u32,
    expected_hash: u64,
    index: u32,
    args: &[u8],
) -> Result<Vec<u8>, HostCallError> {
    let (status, payload) = bindings::fidius::host_call::host::call(
        interface,
        expected_version,
        expected_hash,
        index,
        args,
    );
    decode_host_call_status(interface, index, status, payload)
}

/// Run the name/version/hash gate without dispatching a function (the
/// reserved probe index). `Ok(())` means the host bound a matching table;
/// the error is the same typed error a real call would surface.
pub fn probe(
    interface: &'static str,
    expected_version: u32,
    expected_hash: u64,
) -> Result<(), HostCallError> {
    call(
        interface,
        expected_version,
        expected_hash,
        HOST_CALL_PROBE_INDEX,
        &[],
    )
    .map(|_| ())
}
