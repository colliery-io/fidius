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

//! Regression test for the `fidius` crate's `host` feature gating.
//!
//! `test-plugin-smoke` depends on `fidius` without the `host` feature. A
//! cdylib plugin has no reason to link the host loader (libloading, etc.)
//! and doing so bloats the dylib. This test runs `cargo tree` on the plugin
//! and asserts `libloading` is not in its dep graph.

use std::path::PathBuf;
use std::process::Command;

#[test]
fn plugin_without_host_feature_does_not_pull_libloading() {
    let manifest =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/test-plugin-smoke/Cargo.toml");

    let output = Command::new("cargo")
        .arg("tree")
        .arg("--manifest-path")
        .arg(&manifest)
        .args([
            "-p",
            "test-plugin-smoke",
            "--edges",
            "normal",
            "--prefix",
            "none",
        ])
        .output()
        .expect("failed to run cargo tree");

    assert!(
        output.status.success(),
        "cargo tree failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let tree = String::from_utf8_lossy(&output.stdout);
    let has_libloading = tree
        .lines()
        .any(|line| line.trim_start().starts_with("libloading "));

    assert!(
        !has_libloading,
        "test-plugin-smoke (no `host` feature) pulls libloading in its dep tree. \
         Either the `host` feature was accidentally enabled by default, or a \
         non-optional dependency on fidius-host was introduced in the fidius facade crate.\n\
         Full dep tree:\n{}",
        tree
    );
}
