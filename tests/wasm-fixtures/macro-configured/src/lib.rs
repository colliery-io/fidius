// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// WASM author fixture (FIDIUS-I-0029 / CI.3): a CONFIGURED fidius plugin. The
// host binds `Cfg` once via the macro-emitted `fidius-configure` export; `greet`
// then uses `self.cfg.greeting` without the caller re-passing it. N differently-
// configured instances coexist (each its own persistent store).

use fidius_macro::{plugin_impl, plugin_interface};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Cfg {
    pub greeting: String,
}

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;
}

pub struct ConfGreeter {
    cfg: Cfg,
}

#[plugin_impl(Greeter, crate = "fidius_guest", config = Cfg)]
impl Greeter for ConfGreeter {
    fn greet(&self, name: String) -> String {
        format!("{}, {}!", self.cfg.greeting, name)
    }
}

impl ConfGreeter {
    fn configure(cfg: Cfg) -> Self {
        Self { cfg }
    }
}
