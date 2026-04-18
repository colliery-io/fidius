// Copyright 2026 Colliery, Inc. Licensed under Apache-2.0.
extern crate fidius_core as fidius;

use fidius_macro::plugin_interface;

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait BadPlugin: Send + Sync {
    #[method_meta("effect", "write")]
    #[method_meta("effect", "read")]
    fn do_thing(&self, input: String) -> String;
}

fn main() {}
