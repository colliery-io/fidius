// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//! Host composition: bind config once, then call — N differently-configured
//! instances of one plugin coexist (partial application).
//!
//! Run: `cargo run -p fidius-examples --example 02_configure`
#![allow(unexpected_cfgs)]

use fidius::{plugin_impl, plugin_interface, PluginHandle};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub greeting: String,
}

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;
}

pub struct ConfGreeter {
    cfg: Config,
}

#[plugin_impl(Greeter, config = Config)]
impl Greeter for ConfGreeter {
    fn greet(&self, name: String) -> String {
        // Uses the bound config — the caller never re-passes the greeting.
        format!("{}, {}!", self.cfg.greeting, name)
    }
}

impl ConfGreeter {
    fn configure(cfg: Config) -> Self {
        Self { cfg }
    }
}

fidius::fidius_plugin_registry!();

fn main() {
    let desc = PluginHandle::find_in_process_descriptor("ConfGreeter").expect("registered");

    // Two differently-configured instances of the same plugin, in one host.
    let en = PluginHandle::configure_in_process(
        desc,
        &Config {
            greeting: "Hello".into(),
        },
    )
    .unwrap();
    let sv = PluginHandle::configure_in_process(
        desc,
        &Config {
            greeting: "Hej".into(),
        },
    )
    .unwrap();

    let a: String = en.call_method(0, &("Ada".to_string(),)).unwrap();
    let b: String = sv.call_method(0, &("Bo".to_string(),)).unwrap();
    println!("{a}\n{b}");
    assert_eq!(a, "Hello, Ada!");
    assert_eq!(b, "Hej, Bo!");
}
