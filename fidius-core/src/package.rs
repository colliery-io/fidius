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
use std::path::{Path, PathBuf};

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
    /// Custom file extension for `.fid` archives (e.g., `"cloacina"`).
    /// Defaults to `"fid"` when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension: Option<String>,
}

impl PackageHeader {
    /// Returns the package extension, defaulting to `"fid"`.
    pub fn extension(&self) -> &str {
        self.extension.as_deref().unwrap_or("fid")
    }
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

    /// An error occurred creating or reading an archive.
    #[error("archive error: {0}")]
    ArchiveError(String),

    /// The archive does not contain a valid package.
    #[error("invalid archive: {0}")]
    InvalidArchive(String),
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
pub fn load_manifest_untyped(dir: &Path) -> Result<PackageManifest<toml::Value>, PackageError> {
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

/// Recursively collect file paths for archiving (includes `.sig` files).
fn collect_archive_files(
    root: &Path,
    dir: &Path,
    out: &mut Vec<String>,
) -> Result<(), PackageError> {
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if path.is_dir() {
            if name_str == "target" || name_str == ".git" {
                continue;
            }
            collect_archive_files(root, &path, out)?;
            continue;
        }

        let rel = path
            .strip_prefix(root)
            .expect("path is under root")
            .to_string_lossy()
            .replace('\\', "/");
        out.push(rel);
    }
    Ok(())
}

/// Result of packing a package, including any warnings.
#[derive(Debug)]
pub struct PackResult {
    /// Path to the created `.fid` archive.
    pub path: PathBuf,
    /// Whether the package was unsigned (no `package.sig` found).
    pub unsigned: bool,
}

/// Create a `.fid` archive (tar + bzip2) from a package directory.
///
/// The archive contains a single top-level directory `{name}-{version}/`
/// with all source files. Excludes `target/` and `.git/` directories.
/// Includes `package.sig` if present.
///
/// If `output` is `None`, the archive is written to the current directory
/// as `{name}-{version}.fid`.
pub fn pack_package(dir: &Path, output: Option<&Path>) -> Result<PackResult, PackageError> {
    use bzip2::write::BzEncoder;
    use bzip2::Compression;

    let manifest = load_manifest_untyped(dir)?;
    let pkg = &manifest.package;
    let prefix = format!("{}-{}", pkg.name, pkg.version);
    let ext = pkg.extension();

    let unsigned = !dir.join("package.sig").exists();

    let out_path = match output {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from(format!("{prefix}.{ext}")),
    };

    let file = std::fs::File::create(&out_path).map_err(|e| {
        PackageError::ArchiveError(format!("failed to create {}: {e}", out_path.display()))
    })?;

    let encoder = BzEncoder::new(file, Compression::best());
    let mut tar = tar::Builder::new(encoder);

    let mut files = Vec::new();
    collect_archive_files(dir, dir, &mut files)?;
    files.sort();

    for rel_path in &files {
        let abs_path = dir.join(rel_path);
        let archive_path = format!("{prefix}/{rel_path}");
        tar.append_path_with_name(&abs_path, &archive_path)
            .map_err(|e| PackageError::ArchiveError(format!("failed to add {rel_path}: {e}")))?;
    }

    tar.into_inner()
        .map_err(|e| PackageError::ArchiveError(format!("failed to finish bz2 stream: {e}")))?
        .finish()
        .map_err(|e| PackageError::ArchiveError(format!("failed to finish bz2 stream: {e}")))?;

    Ok(PackResult {
        path: out_path,
        unsigned,
    })
}

/// Extract a `.fid` archive (tar + bzip2) to a destination directory.
///
/// Returns the path to the extracted top-level package directory.
/// Validates that a `package.toml` exists in the extracted contents.
pub fn unpack_package(archive: &Path, dest: &Path) -> Result<PathBuf, PackageError> {
    use bzip2::read::BzDecoder;

    let file = std::fs::File::open(archive).map_err(|e| {
        PackageError::ArchiveError(format!("failed to open {}: {e}", archive.display()))
    })?;

    let decoder = BzDecoder::new(file);
    let mut tar = tar::Archive::new(decoder);

    tar.unpack(dest).map_err(|e| {
        PackageError::ArchiveError(format!("failed to extract {}: {e}", archive.display()))
    })?;

    // Find the top-level directory that was extracted
    let entries = std::fs::read_dir(dest).map_err(PackageError::Io)?;
    let mut pkg_dir: Option<PathBuf> = None;
    for entry in entries {
        let entry = entry.map_err(PackageError::Io)?;
        let path = entry.path();
        if path.is_dir() && path.join("package.toml").exists() {
            pkg_dir = Some(path);
            break;
        }
    }

    let pkg_dir = pkg_dir.ok_or_else(|| {
        PackageError::InvalidArchive("archive does not contain a package.toml".to_string())
    })?;

    Ok(pkg_dir)
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
        assert!(
            err.contains("category"),
            "error should mention missing field: {err}"
        );
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

    fn make_package(dir: &Path) {
        write_manifest(
            dir,
            r#"
            [package]
            name = "test-pkg"
            version = "2.0.0"
            interface = "my-api"
            interface_version = 1

            [metadata]
            category = "testing"
            "#,
        );
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src/lib.rs"), b"fn hello() {}").unwrap();
    }

    #[test]
    fn pack_unpack_round_trip() {
        let pkg_dir = TempDir::new().unwrap();
        make_package(pkg_dir.path());

        let out_dir = TempDir::new().unwrap();
        let fid_path = out_dir.path().join("test-pkg-2.0.0.fid");

        let result = pack_package(pkg_dir.path(), Some(&fid_path)).unwrap();
        assert_eq!(result.path, fid_path);
        assert!(fid_path.exists());
        assert!(result.unsigned);

        let extract_dir = TempDir::new().unwrap();
        let extracted = unpack_package(&fid_path, extract_dir.path()).unwrap();

        assert!(extracted.join("package.toml").exists());
        assert!(extracted.join("src/lib.rs").exists());
        assert_eq!(
            extracted.file_name().unwrap().to_str().unwrap(),
            "test-pkg-2.0.0"
        );
    }

    #[test]
    fn pack_includes_sig_file() {
        let pkg_dir = TempDir::new().unwrap();
        make_package(pkg_dir.path());
        std::fs::write(pkg_dir.path().join("package.sig"), b"fake-sig").unwrap();

        let out_dir = TempDir::new().unwrap();
        let fid_path = out_dir.path().join("out.fid");

        let result = pack_package(pkg_dir.path(), Some(&fid_path)).unwrap();
        assert!(!result.unsigned);

        let extract_dir = TempDir::new().unwrap();
        let extracted = unpack_package(&fid_path, extract_dir.path()).unwrap();
        assert!(extracted.join("package.sig").exists());
    }

    #[test]
    fn pack_excludes_target_and_git() {
        let pkg_dir = TempDir::new().unwrap();
        make_package(pkg_dir.path());
        std::fs::create_dir(pkg_dir.path().join("target")).unwrap();
        std::fs::write(pkg_dir.path().join("target/out.dylib"), b"bin").unwrap();
        std::fs::create_dir(pkg_dir.path().join(".git")).unwrap();
        std::fs::write(pkg_dir.path().join(".git/HEAD"), b"ref").unwrap();

        let out_dir = TempDir::new().unwrap();
        let fid_path = out_dir.path().join("out.fid");
        pack_package(pkg_dir.path(), Some(&fid_path)).unwrap();

        let extract_dir = TempDir::new().unwrap();
        let extracted = unpack_package(&fid_path, extract_dir.path()).unwrap();
        assert!(!extracted.join("target").exists());
        assert!(!extracted.join(".git").exists());
    }

    #[test]
    fn unpack_invalid_archive_no_manifest() {
        let pkg_dir = TempDir::new().unwrap();
        // Create a valid bz2 tar but with no package.toml
        std::fs::create_dir_all(pkg_dir.path().join("src")).unwrap();
        std::fs::write(pkg_dir.path().join("src/lib.rs"), b"fn x() {}").unwrap();

        let out_dir = TempDir::new().unwrap();
        let fid_path = out_dir.path().join("bad.fid");

        // Manually create a tar.bz2 without package.toml
        {
            use bzip2::write::BzEncoder;
            use bzip2::Compression;

            let file = std::fs::File::create(&fid_path).unwrap();
            let encoder = BzEncoder::new(file, Compression::default());
            let mut tar = tar::Builder::new(encoder);
            tar.append_path_with_name(
                pkg_dir.path().join("src/lib.rs"),
                "no-manifest-1.0.0/src/lib.rs",
            )
            .unwrap();
            tar.into_inner().unwrap().finish().unwrap();
        }

        let extract_dir = TempDir::new().unwrap();
        let result = unpack_package(&fid_path, extract_dir.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("package.toml"), "error was: {err}");
    }

    #[test]
    fn pack_default_output_name() {
        let pkg_dir = TempDir::new().unwrap();
        make_package(pkg_dir.path());

        let out_dir = TempDir::new().unwrap();
        let out_path = out_dir.path().join("test-pkg-2.0.0.fid");

        let result = pack_package(pkg_dir.path(), Some(&out_path)).unwrap();
        assert_eq!(result.path, out_path);
        assert!(out_path.exists());
    }

    #[test]
    fn pack_custom_extension() {
        let pkg_dir = TempDir::new().unwrap();
        write_manifest(
            pkg_dir.path(),
            r#"
            [package]
            name = "my-plugin"
            version = "0.3.0"
            interface = "my-api"
            interface_version = 1
            extension = "cloacina"

            [metadata]
            category = "testing"
            "#,
        );
        std::fs::create_dir_all(pkg_dir.path().join("src")).unwrap();
        std::fs::write(pkg_dir.path().join("src/lib.rs"), b"fn hello() {}").unwrap();

        let out_dir = TempDir::new().unwrap();
        let out_path = out_dir.path().join("my-plugin-0.3.0.cloacina");

        let result = pack_package(pkg_dir.path(), Some(&out_path)).unwrap();
        assert_eq!(result.path, out_path);
        assert!(out_path.exists());

        // Verify it unpacks correctly
        let extract_dir = TempDir::new().unwrap();
        let extracted = unpack_package(&out_path, extract_dir.path()).unwrap();
        assert!(extracted.join("package.toml").exists());
    }

    #[test]
    fn extension_defaults_to_fid() {
        let header = PackageHeader {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            interface: "api".to_string(),
            interface_version: 1,
            extension: None,
        };
        assert_eq!(header.extension(), "fid");

        let header_custom = PackageHeader {
            extension: Some("cloacina".to_string()),
            ..header
        };
        assert_eq!(header_custom.extension(), "cloacina");
    }
}
