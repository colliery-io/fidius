// Copyright 2026 Colliery, Inc. Licensed under Apache-2.0.
//
// FIDIUS-I-0032 / ADR-0010: bidirectional streaming (`Stream<T>` in BOTH argument
// and return position) is modelled by the IR + interface hash, but the per-backend
// codegen lands in BD.2 (cdylib) / BD.3 (WASM). Until then a bidi method must fail
// to compile with a clear "not yet wired" error rather than silently mis-generating.
extern crate fidius_core as fidius;

use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Transformer: Send + Sync {
    fn transform(&self, input: fidius::Stream<u32>) -> fidius::Stream<u32>;
}

pub struct MyTransformer;

#[plugin_impl(Transformer)]
impl Transformer for MyTransformer {
    fn transform(&self, input: fidius::Stream<u32>) -> fidius::Stream<u32> {
        let _ = input;
        unimplemented!()
    }
}

fn main() {}
