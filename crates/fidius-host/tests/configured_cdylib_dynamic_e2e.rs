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

//! Configured cdylib via DYNAMIC load (FIDIUS-A-0006 / CI.2). The sibling
//! `configured_cdylib_e2e.rs` proves the in-process path
//! (`configure_in_process` over a statically-linked descriptor). This proves the
//! dynamic-load path: a `#[plugin_impl(Trait, config = C)]` `ConfiguredGreeter`
//! is loaded from a REAL dylib (`tests/test-plugin-smoke`) and configured via
//! `PluginHandle::configure_from_loaded` — the combination a host needs to load a
//! configured provider cdylib at runtime and bind its config once.

use std::path::PathBuf;

use fidius_host::{PluginHandle, PluginHost};
use fidius_test::dylib_fixture;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct GreetConfig {
    greeting: String,
}

/// Build the smoke dylib and return a host whose search path finds it.
fn smoke_host() -> PluginHost {
    let fixture = dylib_fixture(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/test-plugin-smoke"),
    )
    .build();
    PluginHost::builder()
        .search_path(fixture.dir())
        .build()
        .unwrap()
}

#[test]
fn dynamic_cdylib_binds_config_at_load() {
    let host = smoke_host();
    let loaded = host
        .load("ConfiguredGreeter")
        .expect("load ConfiguredGreeter");
    let handle = PluginHandle::configure_from_loaded(
        loaded,
        &GreetConfig {
            greeting: "Hej".into(),
        },
    )
    .expect("configure_from_loaded");

    // greet is method 0; the caller passes only `name`, not the config.
    let out: String = handle.call_method(0, &("Ada".to_string(),)).unwrap();
    assert_eq!(out, "Hej, Ada!");
}

#[test]
fn n_dynamically_configured_instances_coexist() {
    let host = smoke_host();
    let a = PluginHandle::configure_from_loaded(
        host.load("ConfiguredGreeter").unwrap(),
        &GreetConfig {
            greeting: "Hej".into(),
        },
    )
    .unwrap();
    let b = PluginHandle::configure_from_loaded(
        host.load("ConfiguredGreeter").unwrap(),
        &GreetConfig {
            greeting: "Hello".into(),
        },
    )
    .unwrap();

    let a_out: String = a.call_method(0, &("Ada".to_string(),)).unwrap();
    let b_out: String = b.call_method(0, &("Ada".to_string(),)).unwrap();
    assert_eq!(a_out, "Hej, Ada!");
    assert_eq!(b_out, "Hello, Ada!");
}
