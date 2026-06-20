// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// WASM filesystem fixture (FIDIUS-A-0008): a sandboxed connector that does real
// `std::fs` I/O. On wasm32-wasip2 these calls go through wasi:filesystem, which the
// host links; the guest can only touch directories the host preopened via a
// `fs:ro:<path>` / `fs:rw:<path>` capability grant.

use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Fs: Send + Sync {
    /// Read a file; empty string if it can't be read (e.g. no grant).
    fn read_file(&self, path: String) -> String;
    /// Write a file; `true` on success, `false` if denied (e.g. read-only grant).
    fn write_file(&self, path: String, contents: String) -> bool;
}

pub struct FsPlugin;

#[plugin_impl(Fs, crate = "fidius_guest")]
impl Fs for FsPlugin {
    fn read_file(&self, path: String) -> String {
        std::fs::read_to_string(&path).unwrap_or_default()
    }

    fn write_file(&self, path: String, contents: String) -> bool {
        std::fs::write(&path, contents).is_ok()
    }
}
