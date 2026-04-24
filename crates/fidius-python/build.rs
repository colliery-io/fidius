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

//! Build script: configure PyO3 cfg flags and emit a runtime rpath so the
//! embedded Python shared library is found at process launch.
//!
//! Without this, macOS framework Python builds crash on first call with:
//!
//! ```text
//! dyld: Library not loaded: @rpath/Python3.framework/Versions/3.x/Python3
//! ```
//!
//! Mirrors the cloacina-build pattern (which exists for the same reason).

fn main() {
    pyo3_build_config::use_pyo3_cfgs();

    let config = pyo3_build_config::get();
    if let Some(lib_dir) = &config.lib_dir {
        // macOS framework Python: lib_dir looks like
        // /opt/homebrew/Frameworks/Python.framework/Versions/3.X/lib
        // and the dylib loads as @rpath/Python3.framework/...
        // We need to add the parent of the .framework directory to rpath.
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
