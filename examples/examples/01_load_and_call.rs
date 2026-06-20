// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//! Host composition: load an in-process cdylib plugin and call it.
//!
//! Run: `cargo run -p fidius-examples --example 01_load_and_call`
#![allow(unexpected_cfgs)]

use fidius::{plugin_impl, plugin_interface, PluginHandle};

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;
}

pub struct Hello;

#[plugin_impl(Greeter)]
impl Greeter for Hello {
    fn greet(&self, name: String) -> String {
        format!("Hello, {name}!")
    }
}

fidius::fidius_plugin_registry!();

fn main() {
    // The plugin is linked into this binary; the host finds its descriptor in the
    // in-process registry and calls method 0 (greet) through the unified handle API.
    let desc = PluginHandle::find_in_process_descriptor("Hello").expect("plugin registered");
    let handle = PluginHandle::from_descriptor(desc).expect("load");
    let greeting: String = handle.call_method(0, &("Ada".to_string(),)).expect("greet");
    println!("{greeting}");
    assert_eq!(greeting, "Hello, Ada!");
}
