// Copyright 2026 Colliery, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Configured cdylib plugin instances (FIDIUS-I-0029 / CI.2, ADR-0006): a
//! `#[plugin_impl(Trait, config = C)]` whose `configure` constructor binds the
//! config once; methods close over it without re-passing it across the boundary.
//! Proves config is bound at construct, used in methods, and that N
//! differently-configured instances coexist in one host.

use fidius_host::PluginHandle;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GreetConfig {
    pub greeting: String,
}

#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;
}

pub struct ConfiguredGreeter {
    cfg: GreetConfig,
}

#[fidius_macro::plugin_impl(Greeter, crate = "fidius_core", config = GreetConfig)]
impl Greeter for ConfiguredGreeter {
    fn greet(&self, name: String) -> String {
        // Uses the bound config — never re-passed by the caller.
        format!("{}, {}!", self.cfg.greeting, name)
    }
}

impl ConfiguredGreeter {
    fn configure(cfg: GreetConfig) -> Self {
        Self { cfg }
    }
}

fidius_core::fidius_plugin_registry!();

#[test]
fn config_bound_once_and_used_in_methods() {
    let desc = PluginHandle::find_in_process_descriptor("ConfiguredGreeter").unwrap();
    let handle = PluginHandle::configure_in_process(
        desc,
        &GreetConfig {
            greeting: "Hej".into(),
        },
    )
    .expect("configure");
    // greet is method 0; the caller passes only `name`, not the config.
    let out: String = handle.call_method(0, &("Ada".to_string(),)).unwrap();
    assert_eq!(out, "Hej, Ada!");
}

#[test]
fn n_differently_configured_instances_coexist() {
    let desc = PluginHandle::find_in_process_descriptor("ConfiguredGreeter").unwrap();
    let a = PluginHandle::configure_in_process(
        desc,
        &GreetConfig {
            greeting: "Hej".into(),
        },
    )
    .unwrap();
    let b = PluginHandle::configure_in_process(
        desc,
        &GreetConfig {
            greeting: "Hi".into(),
        },
    )
    .unwrap();
    let oa: String = a.call_method(0, &("Ada".to_string(),)).unwrap();
    let ob: String = b.call_method(0, &("Bob".to_string(),)).unwrap();
    assert_eq!(oa, "Hej, Ada!");
    assert_eq!(ob, "Hi, Bob!");
}
