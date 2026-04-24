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

//! Verify `PluginHost` routes correctly between cdylib and python packages
//! when the `python` feature is enabled. Runs only with `--features python`.

#![cfg(feature = "python")]

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use fidius_core::python_descriptor::{PythonInterfaceDescriptor, PythonMethodDesc};
use fidius_host::{PluginHost, PluginRuntimeKind};

const HASH: u64 = 0xCAFEBABE_DEADBEEF;
const METHODS: [PythonMethodDesc; 1] = [PythonMethodDesc {
    name: "shout",
    wire_raw: false,
}];

fn fresh_descriptor() -> (&'static PythonInterfaceDescriptor, String) {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let name = format!("shouter_t{id}");
    let leaked: &'static str = Box::leak(name.clone().into_boxed_str());
    let desc = Box::leak(Box::new(PythonInterfaceDescriptor {
        interface_name: "Shouter",
        interface_hash: HASH,
        methods: &METHODS,
    }));
    let _ = leaked; // suppress lint; descriptor.interface_name is fixed for the test trait
    (desc, name)
}

fn copy_dir(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

fn make_python_package(
    plugins_root: &std::path::Path,
    pkg_name: &str,
    entry_module: &str,
) -> PathBuf {
    let dir = plugins_root.join(pkg_name);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        format!(
            r#"
[package]
name = "{pkg_name}"
version = "0.1.0"
interface = "shouter"
interface_version = 1
runtime = "python"

[metadata]
category = "test"

[python]
entry_module = "{entry_module}"
"#
        ),
    )
    .unwrap();

    let sdk_src = repo_root().join("python/fidius");
    let vendor = dir.join("vendor");
    std::fs::create_dir_all(&vendor).unwrap();
    copy_dir(&sdk_src, &vendor.join("fidius"));

    std::fs::write(
        dir.join(format!("{entry_module}.py")),
        format!(
            r#"
from fidius import method

__interface_hash__ = {HASH}

@method
def shout(text):
    return text.upper()
"#,
        ),
    )
    .unwrap();
    dir
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn discover_surfaces_python_package() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_desc, mod_name) = fresh_descriptor();
    make_python_package(tmp.path(), "py-shouter-discover", &mod_name);

    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let infos = host.discover().unwrap();

    let py = infos
        .iter()
        .find(|i| i.name == "py-shouter-discover")
        .expect("python package should be in discovery results");
    assert!(matches!(py.runtime, PluginRuntimeKind::Python));
    assert_eq!(py.interface_name, "shouter");
}

#[test]
fn load_python_dispatches_through_host() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (desc, mod_name) = fresh_descriptor();
    make_python_package(tmp.path(), "py-shouter-load", &mod_name);

    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_python("py-shouter-load", desc)
        .expect("load_python");

    let input = serde_json::to_vec(&("loud".to_string(),)).unwrap();
    let out = handle.call_typed_json(0, &input).expect("shout");
    let result: String = serde_json::from_slice(&out).unwrap();
    assert_eq!(result, "LOUD");
}

#[test]
fn load_python_unknown_name_returns_not_found() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (desc, _) = fresh_descriptor();

    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let err = host.load_python("does-not-exist", desc).unwrap_err();
    assert!(
        matches!(err, fidius_host::LoadError::PluginNotFound { .. }),
        "expected PluginNotFound, got: {err:?}"
    );
}

#[test]
fn cdylib_load_path_unaffected() {
    // Sanity: discovering a directory that contains both a .so/.dylib and a
    // python package returns both kinds of PluginInfo. We don't have a
    // pre-built cdylib in this temp dir, so just assert that the cdylib
    // path doesn't error when there's only a python package present.
    let tmp = tempfile::TempDir::new().unwrap();
    let (_desc, mod_name) = fresh_descriptor();
    make_python_package(tmp.path(), "py-shouter-coexist", &mod_name);

    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    // load() (cdylib path) on a name that's a python package should be
    // PluginNotFound, not a confusing parse error — the cdylib path simply
    // doesn't see it.
    let err = host.load("py-shouter-coexist").unwrap_err();
    assert!(matches!(err, fidius_host::LoadError::PluginNotFound { .. }));
}
