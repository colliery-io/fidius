// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// WASM author fixture (FIDIUS-I-0028): a fidius connector written entirely with
// the macros. `fetch` issues a brokered outbound GET via `fidius_guest::http`;
// the host's EgressPolicy decides allow/deny before the request leaves the
// sandbox. `#[plugin_impl]` auto-exports the component; `fidius_guest::http`
// contributes the wasi:http import (the two generate! compose). No hand-written
// WIT, no raw bindings — exactly what an adopter's codegen emits.

use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Fetcher: Send + Sync {
    /// GET `url`, returning the body — or `"ERROR: …"` (incl. a denied egress).
    fn fetch(&self, url: String) -> String;
}

pub struct MyFetcher;

#[plugin_impl(Fetcher, crate = "fidius_guest")]
impl Fetcher for MyFetcher {
    fn fetch(&self, url: String) -> String {
        match fidius_guest::http::get(&url) {
            Ok(resp) => resp.text(),
            Err(e) => format!("ERROR: {e}"),
        }
    }
}
