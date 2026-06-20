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

//! Configured + streaming on the **Python** backend (FIDIUS-I-0029): a configured
//! instance (`__fidius_configure__`) whose streaming generator reads the bound
//! config. Completes configured+streaming across all three backends.

#![cfg(all(feature = "python", feature = "streaming"))]

use std::path::{Path, PathBuf};

use fidius_core::from_value;
use fidius_core::python_descriptor::PythonInterfaceDescriptor;
use fidius_host::PluginHost;
use futures::StreamExt;
use serde::Serialize;

#[derive(Serialize)]
struct Cfg {
    base: u64,
}

fn ticker_descriptor() -> &'static PythonInterfaceDescriptor {
    &test_plugin_smoke::__fidius_Ticker::Ticker_PYTHON_DESCRIPTOR
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
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

fn stage(tmp: &tempfile::TempDir) -> PathBuf {
    let plugins_root = tmp.path().to_path_buf();
    let dest = plugins_root.join("py-configured-ticker");
    copy_dir(
        &repo_root().join("tests/test-plugin-py-configured-stream"),
        &dest,
    );
    copy_dir(
        &repo_root().join("python/fidius"),
        &dest.join("vendor").join("fidius"),
    );
    let py = dest.join("configured_ticker.py");
    let src = std::fs::read_to_string(&py).unwrap();
    let injected = src.replace(
        "__HASH_PLACEHOLDER__",
        &format!("0x{:016X}", ticker_descriptor().interface_hash),
    );
    std::fs::write(&py, injected).unwrap();
    plugins_root
}

#[tokio::test]
async fn configured_python_streaming_reads_bound_config() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plugins = stage(&tmp);
    let host = PluginHost::builder().search_path(&plugins).build().unwrap();

    let handle = host
        .load_python_configured(
            "py-configured-ticker",
            ticker_descriptor(),
            &Cfg { base: 100 },
        )
        .expect("load_python_configured");

    let idx = test_plugin_smoke::__fidius_Ticker::METHOD_TICK;
    let mut stream = handle
        .call_streaming::<_, u64>(idx, &(3u32,))
        .await
        .unwrap();
    let mut got = Vec::new();
    while let Some(item) = stream.next().await {
        got.push(from_value::<u64>(item.unwrap()).unwrap());
    }
    assert_eq!(got, vec![100, 101, 102]);
}
