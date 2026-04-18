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

//! End-to-end package tests: validate, build, load, call.

use std::path::PathBuf;

use fidius_core::package::PackageError;
use fidius_host::package;
use serde::Deserialize;

fn test_package_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/test-plugin-smoke")
}

#[derive(Debug, Deserialize)]
struct TestSchema {
    category: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct StrictSchema {
    category: String,
    description: String,
    required_field: String, // This doesn't exist in the fixture
}

#[test]
fn load_manifest_with_schema() {
    let dir = test_package_dir();
    let manifest = package::load_package_manifest::<TestSchema>(&dir).unwrap();

    assert_eq!(manifest.package.name, "test-calculator");
    assert_eq!(manifest.package.version, "0.1.0");
    assert_eq!(manifest.package.interface, "calculator-interface");
    assert_eq!(manifest.package.interface_version, 1);
    assert_eq!(manifest.metadata.category, "math");
    assert!(manifest.metadata.description.contains("calculator"));
}

#[test]
fn schema_mismatch_fails() {
    let dir = test_package_dir();
    let result = package::load_package_manifest::<StrictSchema>(&dir);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("required_field"),
        "error should mention missing field: {err}"
    );
}

#[test]
fn build_and_load_package() {
    let dir = test_package_dir();

    // Build in the same profile as the test binary so wire formats match
    let release = !cfg!(debug_assertions);
    let dylib_path = package::build_package(&dir, release).unwrap();
    assert!(
        dylib_path.exists(),
        "built dylib should exist at {:?}",
        dylib_path
    );

    // Load via PluginHost
    let dylib_dir = dylib_path.parent().unwrap();
    let host = fidius_host::PluginHost::builder()
        .search_path(dylib_dir)
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    assert_eq!(loaded.info.name, "BasicCalculator");
    assert_eq!(loaded.info.interface_name, "Calculator");

    // Call a method
    let handle = fidius_host::PluginHandle::from_loaded(loaded);

    #[derive(serde::Serialize)]
    struct AddInput {
        a: i64,
        b: i64,
    }
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct AddOutput {
        result: i64,
    }

    let output: AddOutput = handle.call_method(0, &(AddInput { a: 5, b: 3 },)).unwrap();
    assert_eq!(output, AddOutput { result: 8 });
}

#[test]
fn discover_packages_finds_fixture() {
    let packages_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests");
    let found = package::discover_packages(&packages_dir).unwrap();

    let names: Vec<&str> = found
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
        .collect();

    assert!(
        names.contains(&"test-plugin-smoke"),
        "should find test-plugin-smoke in {:?}",
        names
    );
}

#[test]
fn missing_manifest_returns_error() {
    let tmp = tempfile::TempDir::new().unwrap();
    let result = package::load_package_manifest::<TestSchema>(tmp.path());
    assert!(matches!(result, Err(PackageError::ManifestNotFound { .. })));
}
