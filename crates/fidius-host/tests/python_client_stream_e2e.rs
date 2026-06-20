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

//! Python **client-streaming** end to end (FIDIUS-I-0030 CS2.4): a plugin method
//! whose `Stream<T>` argument is fed by the host. The host produces items; the
//! Python method receives them as a host-backed iterator (`HostFedStream`) and
//! folds them with plain `sum(rows)`. Completes client-streaming across all three
//! backends.

#![cfg(all(feature = "python", feature = "streaming"))]

use std::path::{Path, PathBuf};

use fidius_core::python_descriptor::PythonInterfaceDescriptor;
use fidius_host::PluginHost;

#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Sink: Send + Sync {
    fn load(&self, rows: fidius_core::Stream<u64>) -> u64;
}

fn sink_descriptor() -> &'static PythonInterfaceDescriptor {
    &__fidius_Sink::Sink_PYTHON_DESCRIPTOR
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
    let dest = plugins_root.join("py-client-stream");
    copy_dir(
        &repo_root().join("tests/test-plugin-py-client-stream"),
        &dest,
    );
    copy_dir(
        &repo_root().join("python/fidius"),
        &dest.join("vendor").join("fidius"),
    );
    let py = dest.join("sink.py");
    let src = std::fs::read_to_string(&py).unwrap();
    let injected = src.replace(
        "__HASH_PLACEHOLDER__",
        &format!("0x{:016X}", sink_descriptor().interface_hash),
    );
    std::fs::write(&py, injected).unwrap();
    plugins_root
}

#[test]
fn python_consumes_a_host_produced_stream() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plugins = stage(&tmp);
    let host = PluginHost::builder().search_path(&plugins).build().unwrap();
    let handle = host
        .load_python("py-client-stream", sink_descriptor())
        .expect("load_python");

    // The host produces [1..=5]; the Python `load` pulls them via the host-fed
    // iterator and sums → 15.
    let items: Vec<u64> = vec![1, 2, 3, 4, 5];
    let sum: u64 = handle
        .call_client_streaming(0, items, &())
        .expect("client-streaming call");
    assert_eq!(sum, 15);
}
