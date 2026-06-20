// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// FIDIUS-I-0029 / CI.3: a CONFIGURED server-streaming plugin. `base` is bound once
// via fidius-configure; `tick` streams base, base+1, ... — proving config + streaming
// compose (the stream's store is configured before it starts).
use fidius_macro::{plugin_impl, plugin_interface};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Cfg {
    pub base: u64,
}

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Ticker: Send + Sync {
    fn tick(&self, count: u32) -> fidius_guest::Stream<u64>;
}

pub struct ConfTicker {
    cfg: Cfg,
}

#[plugin_impl(Ticker, crate = "fidius_guest", config = Cfg)]
impl Ticker for ConfTicker {
    fn tick(&self, count: u32) -> fidius_guest::Stream<u64> {
        let base = self.cfg.base;
        fidius_guest::Stream::from_iter((0..count as u64).map(move |i| base + i))
    }
}

impl ConfTicker {
    fn configure(cfg: Cfg) -> Self {
        Self { cfg }
    }
}
