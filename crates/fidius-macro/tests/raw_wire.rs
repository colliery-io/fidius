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

//! Macro-side tests for `#[wire(raw)]` byte-passthrough mode.
//!
//! These tests live in fidius-macro (rather than fidius-host) so they can
//! probe the generated companion module's interface-hash constants directly,
//! without needing to load a dylib.

extern crate fidius_core as fidius;

use fidius_macro::plugin_interface;

// Two interfaces with the *same* method name and Rust signature, differing
// only in `#[wire(raw)]`. The `!raw` marker baked into the signature string
// must make their interface hashes diverge — that's the guarantee that a
// host built against one and a plugin built against the other refuses to
// load instead of silently corrupting data.

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait TypedPipe: Send + Sync {
    fn process(&self, data: Vec<u8>) -> Vec<u8>;
}

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait RawPipe: Send + Sync {
    #[wire(raw)]
    fn process(&self, data: Vec<u8>) -> Vec<u8>;
}

#[test]
fn raw_marker_changes_interface_hash() {
    let typed = __fidius_TypedPipe::TypedPipe_INTERFACE_HASH;
    let raw = __fidius_RawPipe::RawPipe_INTERFACE_HASH;
    assert_ne!(
        typed, raw,
        "raw and typed methods with identical Rust signatures must hash differently",
    );
}

// Mixed interface: one raw method, one typed method, plus an optional method.
// Confirms codegen handles the interleave and that the companion module is
// well-formed.
#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Mixed: Send + Sync {
    #[wire(raw)]
    fn bulk(&self, payload: Vec<u8>) -> Vec<u8>;

    fn ping(&self) -> String;

    #[optional(since = 2)]
    #[wire(raw)]
    fn bulk_v2(&self, payload: Vec<u8>) -> Vec<u8>;
}

#[test]
fn mixed_interface_companion_module_compiles() {
    // The mere fact that the constants resolve at all means codegen accepted
    // the interleave of raw, typed, and optional+raw methods.
    let _ = __fidius_Mixed::Mixed_INTERFACE_HASH;
    let _ = __fidius_Mixed::Mixed_INTERFACE_VERSION;
    let _ = __fidius_Mixed::METHOD_BULK;
    let _ = __fidius_Mixed::METHOD_PING;
    let _ = __fidius_Mixed::METHOD_BULK_V2;
    let _ = __fidius_Mixed::Mixed_CAP_BULK_V2;
}

// Confirm `Result<Vec<u8>, E>` returns are accepted by the macro and the
// companion module emits cleanly. End-to-end Result-path execution is
// exercised at the integration-test layer (typed errors take the existing
// bincode error path; only the success payload bypasses bincode).
#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait FallibleBytePipe: Send + Sync {
    #[wire(raw)]
    fn maybe(&self, data: Vec<u8>) -> Result<Vec<u8>, fidius::PluginError>;
}

#[test]
fn raw_method_with_result_return_compiles() {
    let _ = __fidius_FallibleBytePipe::FallibleBytePipe_INTERFACE_HASH;
    let _ = __fidius_FallibleBytePipe::METHOD_MAYBE;
}
