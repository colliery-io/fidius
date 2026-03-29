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

//! Host-side package integration: manifest loading, discovery, and building.

use std::path::{Path, PathBuf};

use fidius_core::package::{PackageError, PackageManifest};
use serde::de::DeserializeOwned;

/// Load and validate a package manifest against a host-defined schema.
///
/// This is the primary entry point for host applications working with packages.
/// The type parameter `M` is the host's metadata schema — if the manifest's
/// `[metadata]` section doesn't deserialize into `M`, this returns an error.
///
/// # Example
///
/// ```ignore
/// #[derive(Deserialize)]
/// struct MySchema {
///     category: String,
///     min_host_version: String,
/// }
///
/// let manifest = load_package_manifest::<MySchema>(Path::new("./packages/blur/"))?;
/// assert_eq!(manifest.metadata.category, "image-processing");
/// ```
pub fn load_package_manifest<M: DeserializeOwned>(
    dir: &Path,
) -> Result<PackageManifest<M>, PackageError> {
    fidius_core::package::load_manifest(dir)
}

/// Discover packages in a directory.
///
/// Scans `dir` for subdirectories containing a `package.toml` file.
/// Returns the paths to each package directory found.
pub fn discover_packages(dir: &Path) -> Result<Vec<PathBuf>, PackageError> {
    let mut packages = Vec::new();

    if !dir.is_dir() {
        return Ok(packages);
    }

    let entries =
        std::fs::read_dir(dir).map_err(PackageError::Io)?;

    for entry in entries {
        let entry = entry.map_err(PackageError::Io)?;
        let path = entry.path();
        if path.is_dir() && path.join("package.toml").exists() {
            packages.push(path);
        }
    }

    packages.sort();
    Ok(packages)
}

/// Build a package by running `cargo build` inside the package directory.
///
/// Returns the path to the compiled cdylib on success.
pub fn build_package(dir: &Path, release: bool) -> Result<PathBuf, PackageError> {
    let cargo_toml = dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(PackageError::BuildFailed(format!(
            "Cargo.toml not found in {}",
            dir.display()
        )));
    }

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build").arg("--manifest-path").arg(&cargo_toml);
    if release {
        cmd.arg("--release");
    }

    let output = cmd.output().map_err(PackageError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PackageError::BuildFailed(stderr.to_string()));
    }

    let profile = if release { "release" } else { "debug" };
    let target_dir = dir.join("target").join(profile);

    // Find the cdylib in the target directory
    let dylib_ext = if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(target_os = "windows") {
        "dll"
    } else {
        "so"
    };

    // Look for any file with the right extension
    if let Ok(entries) = std::fs::read_dir(&target_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some(dylib_ext) {
                return Ok(path);
            }
        }
    }

    // Return the target dir even if we can't find the specific dylib
    Ok(target_dir)
}
