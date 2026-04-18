// Copyright 2026 Colliery, Inc. Licensed under Apache-2.0.
//
// `CallerAllocated` buffer strategy was removed in fidius 0.1.0 per
// FIDIUS-I-0014. Interfaces that declare it should fail to compile
// with a clear error.
extern crate fidius_core as fidius;

use fidius_macro::plugin_interface;

#[plugin_interface(version = 1, buffer = CallerAllocated)]
pub trait BadPlugin: Send + Sync {
    fn do_thing(&self, input: String) -> String;
}

fn main() {}
