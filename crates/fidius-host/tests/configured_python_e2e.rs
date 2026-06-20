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

//! Configured Python plugin instances (FIDIUS-I-0029 / CI.4, ADR-0006): a Python
//! plugin that exports `__fidius_configure__(config) -> instance` instead of
//! module-level functions. The host binds the config once via
//! `load_python_configured`, methods run on the configured instance, and N
//! differently-configured instances coexist. Implements `BytePipe` (reuses its
//! macro-generated descriptor); `name()` returns the configured display name.

#![cfg(feature = "python")]

use std::path::{Path, PathBuf};

use fidius_core::python_descriptor::PythonInterfaceDescriptor;
use fidius_host::PluginHost;
use serde::Serialize;

#[derive(Serialize)]
struct PipeConfig {
    display_name: String,
}

fn byte_pipe_descriptor() -> &'static PythonInterfaceDescriptor {
    &test_plugin_smoke::__fidius_BytePipe::BytePipe_PYTHON_DESCRIPTOR
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &dest);
        } else {
            std::fs::copy(&path, &dest).unwrap();
        }
    }
}

/// Stage the configured fixture: copy it, vendor the SDK, and bake the real
/// BytePipe interface hash into the `__HASH_PLACEHOLDER__` sentinel.
fn stage(tmp: &tempfile::TempDir) -> PathBuf {
    let plugins_root = tmp.path().to_path_buf();
    let dest = plugins_root.join("py-configured-pipe");
    copy_dir(&repo_root().join("tests/test-plugin-py-configured"), &dest);
    copy_dir(
        &repo_root().join("python/fidius"),
        &dest.join("vendor/fidius"),
    );

    let py = dest.join("configured_pipe.py");
    let src = std::fs::read_to_string(&py).unwrap();
    let patched = src.replace(
        "__HASH_PLACEHOLDER__",
        &format!("0x{:016X}", byte_pipe_descriptor().interface_hash),
    );
    std::fs::write(&py, patched).unwrap();
    plugins_root
}

#[test]
fn config_bound_once_and_used_in_methods() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plugins = stage(&tmp);
    let host = PluginHost::builder().search_path(&plugins).build().unwrap();

    let handle = host
        .load_python_configured(
            "py-configured-pipe",
            byte_pipe_descriptor(),
            &PipeConfig {
                display_name: "configured-pipe".into(),
            },
        )
        .expect("load_python_configured");

    // name() is method index 1; it returns the bound config, not a per-call arg.
    let name: String = handle.call_method(1, &()).expect("name");
    assert_eq!(name, "configured-pipe");

    // reverse() (raw wire, index 0) runs on the same configured instance.
    let out = handle.call_method_raw(0, b"abc").expect("reverse");
    assert_eq!(out, b"cba");
}
