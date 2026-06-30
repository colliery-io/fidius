// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// WASM author fixture: a CONFIGURED fidius plugin that also does real `std::fs`
// I/O. Config (`Cfg.suffix`) is bound once via the macro-emitted
// `fidius-configure` export; `read_file` then appends the bound suffix to what it
// read. On wasm32-wasip2 the `std::fs` call goes through wasi:filesystem, reachable
// only for directories the host preopened via an `fs:ro:`/`fs:rw:` grant. Used to
// prove `load_wasm_configured_with_grants` binds config AND applies a load-time
// capability allow-list that overrides the package manifest.

use fidius_macro::{plugin_impl, plugin_interface};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Cfg {
    pub suffix: String,
}

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait ConfiguredFs: Send + Sync {
    /// Read a file and append the configured suffix; empty (then suffix) if it
    /// can't be read (e.g. no grant).
    fn read_file(&self, path: String) -> String;
}

pub struct ConfFsPlugin {
    cfg: Cfg,
}

#[plugin_impl(ConfiguredFs, crate = "fidius_guest", config = Cfg)]
impl ConfiguredFs for ConfFsPlugin {
    fn read_file(&self, path: String) -> String {
        let body = std::fs::read_to_string(&path).unwrap_or_default();
        format!("{body}{}", self.cfg.suffix)
    }
}

impl ConfFsPlugin {
    fn configure(cfg: Cfg) -> Self {
        Self { cfg }
    }
}
