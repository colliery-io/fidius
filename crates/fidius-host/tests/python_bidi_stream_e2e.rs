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

//! Python **bidirectional** streaming end to end (FIDIUS-I-0032 / ADR-0010): a method
//! that receives a host-fed iterator (input, CS2.4) AND returns a generator (output,
//! ST). The host produces the input and pumps the output generator; each `yield` pulls
//! one input item. Completes bidirectional across all three backends.

#![cfg(all(feature = "python", feature = "streaming"))]

use std::path::{Path, PathBuf};

use fidius_core::from_value;
use fidius_core::python_descriptor::PythonInterfaceDescriptor;
use fidius_host::PluginHost;
use futures::StreamExt;

#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Transformer: Send + Sync {
    fn transform(&self, input: fidius_core::Stream<u64>) -> fidius_core::Stream<u64>;
}

fn transformer_descriptor() -> &'static PythonInterfaceDescriptor {
    &__fidius_Transformer::Transformer_PYTHON_DESCRIPTOR
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
    let dest = plugins_root.join("py-bidi-stream");
    copy_dir(&repo_root().join("tests/test-plugin-py-bidi-stream"), &dest);
    copy_dir(
        &repo_root().join("python/fidius"),
        &dest.join("vendor").join("fidius"),
    );
    let py = dest.join("transformer.py");
    let src = std::fs::read_to_string(&py).unwrap();
    let injected = src.replace(
        "__HASH_PLACEHOLDER__",
        &format!("0x{:016X}", transformer_descriptor().interface_hash),
    );
    std::fs::write(&py, injected).unwrap();
    plugins_root
}

#[tokio::test]
async fn python_bidi_doubles_a_host_produced_stream() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plugins = stage(&tmp);
    let host = PluginHost::builder().search_path(&plugins).build().unwrap();
    let handle = host
        .load_python("py-bidi-stream", transformer_descriptor())
        .expect("load_python");

    // Host produces [1..=5]; the generator yields each doubled, pulling one input per yield.
    let items: Vec<u64> = vec![1, 2, 3, 4, 5];
    let mut stream = handle
        .call_bidi_streaming::<u64, (), u64>(0, items, &())
        .await
        .expect("call_bidi_streaming");

    let mut got = Vec::new();
    while let Some(item) = stream.next().await {
        got.push(from_value::<u64>(item.expect("item ok")).expect("u64"));
    }
    assert_eq!(got, vec![2, 4, 6, 8, 10]);
}
