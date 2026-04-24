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

//! Build script: when the `python` feature is enabled, embed a runtime
//! rpath so this crate's test binaries (and downstream binaries that
//! depend on `fidius-host` with `python`) can find libpython at process
//! launch.
//!
//! Mirrors `crates/fidius-python/build.rs`. Both build scripts must run
//! because each one's outputs only apply to the linker invocation that
//! consumes its compilation unit — `fidius-python`'s build.rs sets rpath
//! for fidius-python's own test binaries, and fidius-host's needs to do
//! the same for fidius-host's test binaries.

fn main() {
    if std::env::var_os("CARGO_FEATURE_PYTHON").is_none() {
        return;
    }
    pyo3_build_config::use_pyo3_cfgs();

    let config = pyo3_build_config::get();
    if let Some(lib_dir) = &config.lib_dir {
        let rpath = if lib_dir.contains(".framework/") {
            let parts: Vec<&str> = lib_dir.splitn(2, ".framework/").collect();
            std::path::Path::new(parts[0])
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| lib_dir.clone())
        } else {
            lib_dir.clone()
        };
        println!("cargo:rustc-link-arg=-Wl,-rpath,{rpath}");
    }
}
