// Copyright 2026 Colliery, Inc. Licensed under Apache-2.0.
extern crate fidius_core as fidius;

use fidius_macro::plugin_interface;

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait BadPlugin: Send + Sync {
    #[method_meta("fidius.reserved", "nope")]
    fn do_thing(&self, input: String) -> String;
}

fn main() {}
