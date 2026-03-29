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

//! Source package manifest types and parsing.
//!
//! A package is a directory containing plugin source code and a `package.toml`
//! manifest. The manifest has a fixed header (name, version, interface) and
//! an extensible `[metadata]` section validated via serde against a
//! host-defined schema type.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A parsed package manifest, generic over the host-defined metadata schema.
///
/// The `M` type parameter is the host's metadata schema. If the `[metadata]`
/// section of `package.toml` doesn't deserialize into `M`, parsing fails —
/// this is how schema validation works.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest<M> {
    /// Fixed header fields required by fidius.
    pub package: PackageHeader,
    /// Host-defined metadata. Must deserialize from the `[metadata]` section.
    pub metadata: M,
}

/// Fixed header fields that every package manifest must have.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageHeader {
    /// Package name (e.g., `"blur-filter"`).
    pub name: String,
    /// Package version (e.g., `"1.2.0"`).
    pub version: String,
    /// Name of the interface crate this package implements.
    pub interface: String,
    /// Expected interface version.
    pub interface_version: u32,
}

/// Errors that can occur when loading a package manifest.
#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    /// The `package.toml` file was not found in the given directory.
    #[error("package.toml not found in {path}")]
    ManifestNotFound { path: String },

    /// The manifest file could not be parsed as valid TOML or failed
    /// schema validation (the `[metadata]` section didn't match `M`).
    #[error("failed to parse package.toml: {0}")]
    ParseError(#[from] toml::de::Error),

    /// An I/O error occurred reading the manifest file.
    #[error("io error reading package.toml: {0}")]
    Io(#[from] std::io::Error),

    /// Build failed.
    #[error("package build failed: {0}")]
    BuildFailed(String),

    /// Package signature file not found.
    #[error("package.sig not found in {path}")]
    SignatureNotFound { path: String },

    /// Package signature is invalid (no trusted key verified it).
    #[error("package signature invalid for {path}")]
    SignatureInvalid { path: String },
}

/// Load and parse a `package.toml` manifest from a package directory.
///
/// The type parameter `M` is the host's metadata schema. If the `[metadata]`
/// section doesn't deserialize into `M`, this returns `PackageError::ParseError`.
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
/// let manifest = load_manifest::<MySchema>(Path::new("./my-package/"))?;
/// println!("Package: {} v{}", manifest.package.name, manifest.package.version);
/// println!("Category: {}", manifest.metadata.category);
/// ```
pub fn load_manifest<M: DeserializeOwned>(dir: &Path) -> Result<PackageManifest<M>, PackageError> {
    let manifest_path = dir.join("package.toml");

    if !manifest_path.exists() {
        return Err(PackageError::ManifestNotFound {
            path: dir.display().to_string(),
        });
    }

    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest: PackageManifest<M> = toml::from_str(&content)?;
    Ok(manifest)
}

/// Load a manifest validating only the fixed header (accepting any metadata).
///
/// Uses `toml::Value` as the metadata type so any `[metadata]` section is accepted.
/// Useful for CLI tools that validate structure without knowing the host's schema.
pub fn load_manifest_untyped(
    dir: &Path,
) -> Result<PackageManifest<toml::Value>, PackageError> {
    load_manifest::<toml::Value>(dir)
}

/// Compute a deterministic SHA-256 digest over all package source files.
///
/// Walks the package directory, collects all files (excluding `target/`,
/// `.git/`, and `*.sig` files), sorts by relative path, and feeds each
/// file's relative path and contents into a SHA-256 hasher.
///
/// The resulting 32-byte digest covers the entire package contents.
/// Sign this digest to protect against tampering.
pub fn package_digest(dir: &Path) -> Result<[u8; 32], PackageError> {
    use sha2::{Digest, Sha256};

    let mut files = Vec::new();
    collect_files(dir, dir, &mut files)?;
    files.sort();

    let mut hasher = Sha256::new();
    for rel_path in &files {
        let abs_path = dir.join(rel_path);
        let contents = std::fs::read(&abs_path)?;
        // Hash the relative path (as UTF-8 bytes) then the file contents.
        // Length-prefix both to prevent ambiguity.
        let path_bytes = rel_path.as_bytes();
        hasher.update((path_bytes.len() as u64).to_le_bytes());
        hasher.update(path_bytes);
        hasher.update((contents.len() as u64).to_le_bytes());
        hasher.update(&contents);
    }

    Ok(hasher.finalize().into())
}

/// Recursively collect file paths relative to `root`, skipping excluded dirs/files.
fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), PackageError> {
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip excluded directories
        if path.is_dir() {
            if name_str == "target" || name_str == ".git" {
                continue;
            }
            collect_files(root, &path, out)?;
            continue;
        }

        // Skip signature files
        if name_str.ends_with(".sig") {
            continue;
        }

        // Store relative path using forward slashes for cross-platform determinism
        let rel = path
            .strip_prefix(root)
            .expect("path is under root")
            .to_string_lossy()
            .replace('\\', "/");
        out.push(rel);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_manifest(dir: &Path, content: &str) {
        std::fs::write(dir.join("package.toml"), content).unwrap();
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestMeta {
        category: String,
        #[serde(default)]
        tags: Vec<String>,
    }

    #[test]
    fn valid_manifest_parses() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            tmp.path(),
            r#"
            [package]
            name = "test-pkg"
            version = "1.0.0"
            interface = "my-api"
            interface_version = 1

            [metadata]
            category = "testing"
            tags = ["a", "b"]
            "#,
        );

        let m = load_manifest::<TestMeta>(tmp.path()).unwrap();
        assert_eq!(m.package.name, "test-pkg");
        assert_eq!(m.package.version, "1.0.0");
        assert_eq!(m.package.interface, "my-api");
        assert_eq!(m.package.interface_version, 1);
        assert_eq!(m.metadata.category, "testing");
        assert_eq!(m.metadata.tags, vec!["a", "b"]);
    }

    #[test]
    fn missing_required_metadata_field_fails() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            tmp.path(),
            r#"
            [package]
            name = "bad-pkg"
            version = "1.0.0"
            interface = "my-api"
            interface_version = 1

            [metadata]
            # missing required "category" field
            tags = ["x"]
            "#,
        );

        let result = load_manifest::<TestMeta>(tmp.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("category"), "error should mention missing field: {err}");
    }

    #[test]
    fn missing_manifest_returns_not_found() {
        let tmp = TempDir::new().unwrap();
        let result = load_manifest::<TestMeta>(tmp.path());
        assert!(matches!(result, Err(PackageError::ManifestNotFound { .. })));
    }

    #[test]
    fn extra_metadata_fields_ignored() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            tmp.path(),
            r#"
            [package]
            name = "extra-pkg"
            version = "1.0.0"
            interface = "my-api"
            interface_version = 1

            [metadata]
            category = "testing"
            unknown_field = "ignored"
            "#,
        );

        // TestMeta doesn't have unknown_field — should still parse (serde ignores unknown by default)
        let m = load_manifest::<TestMeta>(tmp.path());
        assert!(m.is_ok());
        assert_eq!(m.unwrap().metadata.category, "testing");
    }

    #[test]
    fn untyped_manifest_accepts_any_metadata() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            tmp.path(),
            r#"
            [package]
            name = "any-pkg"
            version = "1.0.0"
            interface = "my-api"
            interface_version = 1

            [metadata]
            foo = "bar"
            count = 42
            nested = { a = 1, b = 2 }
            "#,
        );

        let m = load_manifest_untyped(tmp.path()).unwrap();
        assert_eq!(m.package.name, "any-pkg");
        assert!(m.metadata.is_table());
    }

    #[test]
    fn digest_is_deterministic() {
        let tmp = TempDir::new().unwrap();
        write_manifest(tmp.path(), "[package]\nname = \"test\"\nversion = \"1.0.0\"\ninterface = \"api\"\ninterface_version = 1\n\n[metadata]\nk = \"v\"\n");
        std::fs::write(tmp.path().join("src.rs"), b"fn main() {}").unwrap();

        let d1 = package_digest(tmp.path()).unwrap();
        let d2 = package_digest(tmp.path()).unwrap();
        assert_eq!(d1, d2);
    }

    #[test]
    fn digest_changes_on_file_modification() {
        let tmp = TempDir::new().unwrap();
        write_manifest(tmp.path(), "[package]\nname = \"test\"\nversion = \"1.0.0\"\ninterface = \"api\"\ninterface_version = 1\n\n[metadata]\nk = \"v\"\n");
        std::fs::write(tmp.path().join("src.rs"), b"fn main() {}").unwrap();

        let d1 = package_digest(tmp.path()).unwrap();

        std::fs::write(tmp.path().join("src.rs"), b"fn main() { evil() }").unwrap();
        let d2 = package_digest(tmp.path()).unwrap();

        assert_ne!(d1, d2);
    }

    #[test]
    fn digest_excludes_target_and_sig() {
        let tmp = TempDir::new().unwrap();
        write_manifest(tmp.path(), "[package]\nname = \"test\"\nversion = \"1.0.0\"\ninterface = \"api\"\ninterface_version = 1\n\n[metadata]\nk = \"v\"\n");
        std::fs::write(tmp.path().join("src.rs"), b"fn main() {}").unwrap();

        let d1 = package_digest(tmp.path()).unwrap();

        // Adding target/ dir and .sig file should not change digest
        std::fs::create_dir(tmp.path().join("target")).unwrap();
        std::fs::write(tmp.path().join("target/output.dylib"), b"binary").unwrap();
        std::fs::write(tmp.path().join("package.sig"), b"sig bytes").unwrap();

        let d2 = package_digest(tmp.path()).unwrap();
        assert_eq!(d1, d2);
    }
}
