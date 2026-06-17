// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// WASM author fixture (FIDIUS-T-0112): a fidius plugin defined entirely with the
// fidius macros. `#[plugin_impl]` auto-exports a WIT component (FIDIUS-T-0106) on
// the wasm target; the cdylib machinery is gated off wasm (FIDIUS-T-0111). Built
// for wasm32-wasip2; loaded by the host via the macro-emitted `Greeter_WASM_DESCRIPTOR`.

use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;

    #[wire(raw)]
    fn echo(&self, data: Vec<u8>) -> Vec<u8>;
}

pub struct MyGreeter;

#[plugin_impl(Greeter, crate = "fidius_guest")]
impl Greeter for MyGreeter {
    fn greet(&self, name: String) -> String {
        format!("Hello, {name}!")
    }

    #[wire(raw)]
    fn echo(&self, data: Vec<u8>) -> Vec<u8> {
        let mut d = data;
        d.reverse();
        d
    }
}
