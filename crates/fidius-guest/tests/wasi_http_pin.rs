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

//! Drift tripwire (FIDIUS-A-0005). `fidius_guest::http` vendors ONE `wasi:http`
//! version — the single pinned contract for the whole ecosystem, matched to the
//! host's `wasmtime-wasi-http`. This asserts the pin is what we think it is, so an
//! accidental re-vendor is caught with a clear message. The macro-authored
//! connector E2E (`fidius-host/tests/macro_egress_e2e.rs`) is the *runtime* guard:
//! if the guest's version ever diverges from the host's, that test fails to
//! instantiate. When you bump `wasmtime-wasi-http` in the host, re-vendor
//! `crates/fidius-guest/wit/` and update `PINNED` here in the same change.

const PINNED: &str = "wasi:http/outgoing-handler@0.2.6";

#[test]
fn vendored_wasi_http_version_is_pinned() {
    let world = include_str!("../wit/world.wit");
    assert!(
        world.contains(PINNED),
        "fidius-guest's vendored wasi:http pin changed (expected `{PINNED}` in \
         wit/world.wit). Bumping wasmtime? Re-vendor wit/ AND update PINNED — the \
         pin is a published ABI (ADR-0005), moved only deliberately."
    );
}
