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
    /// Python-runtime fields. Required when `package.runtime == "python"`,
    /// rejected otherwise. Validated by [`PackageManifest::validate_runtime`]
    /// after deserialization, since serde alone can't enforce cross-section
    /// invariants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python: Option<PythonPackageMeta>,
}

impl<M> PackageManifest<M> {
    /// Cross-section validation: runtime + python section must agree.
    ///
    /// - `runtime = "rust"` (or absent → "rust") with a `[python]` section is rejected.
    /// - `runtime = "python"` without a `[python]` section is rejected.
    /// - Unknown runtime values are rejected (forward compat: a future
    ///   `runtime = "node"` package shouldn't silently fall back to rust).
    pub fn validate_runtime(&self) -> Result<(), PackageError> {
        let runtime = self.package.runtime();
        match runtime {
            PackageRuntime::Rust => {
                if self.python.is_some() {
                    return Err(PackageError::InvalidManifest(
                        "[python] section is only valid when runtime = \"python\"".into(),
                    ));
                }
                Ok(())
            }
            PackageRuntime::Python => {
                if self.python.is_none() {
                    return Err(PackageError::InvalidManifest(
                        "runtime = \"python\" requires a [python] section with `entry_module`"
                            .into(),
                    ));
                }
                Ok(())
            }
        }
    }
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
    /// Plugin runtime. `"rust"` (default) → cdylib; `"python"` → Python package
    /// loaded by `fidius-python`. Unknown values are rejected at validation
    /// time (see [`PackageManifest::validate_runtime`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
}

impl PackageHeader {
    /// Returns the package extension, defaulting to `"fid"`.
    pub fn extension(&self) -> &str {
        self.extension.as_deref().unwrap_or("fid")
    }

    /// Returns the runtime kind, defaulting to `Rust` when absent. Returns
    /// `PackageRuntime::Rust` for unknown values; callers that need to reject
    /// unknown runtimes should use [`Self::runtime_strict`].
    pub fn runtime(&self) -> PackageRuntime {
        match self.runtime.as_deref() {
            None | Some("rust") => PackageRuntime::Rust,
            Some("python") => PackageRuntime::Python,
            // Unknown values fall back to Rust for `runtime()`, but the
            // strict validator rejects them. Keep the lenient form so display
            // code never panics on an unfamiliar manifest.
            _ => PackageRuntime::Rust,
        }
    }

    /// Returns the runtime kind, erroring on unknown values.
    pub fn runtime_strict(&self) -> Result<PackageRuntime, PackageError> {
        match self.runtime.as_deref() {
            None | Some("rust") => Ok(PackageRuntime::Rust),
            Some("python") => Ok(PackageRuntime::Python),
            Some(other) => Err(PackageError::InvalidManifest(format!(
                "unknown runtime '{other}': allowed values are \"rust\", \"python\""
            ))),
        }
    }
}

/// Plugin runtime kind. Determines which loader the host's `PluginHost`
/// dispatches to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageRuntime {
    /// Default. Plugin is a cdylib + `PluginRegistry`. Loaded by the existing
    /// dylib loader in `fidius-host`.
    Rust,
    /// Plugin is a directory of `.py` files (+ optional `vendor/`) loaded by
    /// `fidius-python` via an embedded interpreter. Requires the host crate
    /// to enable the `python` feature.
    Python,
}

impl PackageRuntime {
    /// Returns the canonical string form used in `package.toml`.
    pub fn as_str(&self) -> &'static str {
        match self {
            PackageRuntime::Rust => "rust",
            PackageRuntime::Python => "python",
        }
    }
}

impl std::fmt::Display for PackageRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Fields under the `[python]` section of `package.toml`. Required when
/// `package.runtime == "python"`, rejected otherwise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonPackageMeta {
    /// Python module the loader imports first. Dotted-path form (e.g.
    /// `"my_plugin.entry"`) corresponding to a file inside the package
    /// directory or its `vendor/` tree.
    pub entry_module: String,
    /// Path to the requirements file consumed by `fidius pack` to vendor
    /// dependencies into `vendor/`. Defaults to `"requirements.txt"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirements: Option<String>,
}

impl PythonPackageMeta {
    /// Returns the requirements file path, defaulting to `"requirements.txt"`.
    pub fn requirements_path(&self) -> &str {
        self.requirements.as_deref().unwrap_or("requirements.txt")
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

    /// Manifest passed serde parsing but failed cross-section validation
    /// (e.g. `runtime = "python"` without a `[python]` section, or unknown
    /// runtime value).
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),

    /// Archive entry contains a `..` component that would escape `dest`.
    #[error("archive entry '{entry}' contains '..' component — rejected")]
    PathTraversal { entry: String },

    /// Archive entry has an absolute path (root or drive prefix).
    #[error("archive entry '{entry}' is an absolute path — rejected")]
    AbsolutePath { entry: String },

    /// Archive contains a symlink entry, which could be used to overwrite
    /// arbitrary files outside `dest` on a follow-up write.
    #[error("archive entry '{entry}' is a symlink — rejected")]
    SymlinkRejected { entry: String },

    /// Archive contains a hardlink entry, same threat model as symlinks.
    #[error("archive entry '{entry}' is a hardlink — rejected")]
    HardlinkRejected { entry: String },

    /// Cumulative decompressed size exceeded the configured cap.
    #[error("archive decompressed size {actual} exceeds limit of {limit} bytes")]
    SizeLimitExceeded { limit: u64, actual: u64 },

    /// Archive contains more entries than the configured cap allows.
    #[error("archive contains more than {limit} entries — rejected")]
    TooManyEntries { limit: u32 },
}

/// Options controlling archive extraction safety limits.
///
/// Construct with `UnpackOptions::default()` for strict defaults suitable for
/// untrusted input. Override individual fields for known-trusted archives that
/// legitimately exceed the default caps (e.g. packages that vendor large
/// native dependencies).
#[derive(Debug, Clone)]
pub struct UnpackOptions {
    /// Maximum total declared uncompressed size of all entries, in bytes.
    /// Archives exceeding this are rejected as potential decompression bombs.
    pub max_decompressed: u64,
    /// Maximum ratio of total declared uncompressed size to compressed
    /// archive size. Archives exceeding this are rejected.
    pub max_ratio: u64,
    /// Maximum number of entries in the archive. Guards against archives
    /// that exhaust inodes or directory-entry limits via tiny-file spam.
    pub max_entries: u32,
}

impl Default for UnpackOptions {
    fn default() -> Self {
        Self {
            max_decompressed: 500 * 1024 * 1024,
            max_ratio: 10,
            max_entries: 10_000,
        }
    }
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
    // Reject unknown runtime values + cross-section invariants. We do this
    // here (not in serde) because the python-section presence depends on
    // the runtime field, which serde can't express in a single derive.
    manifest.package.runtime_strict()?;
    manifest.validate_runtime()?;
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

/// Vendor Python dependencies into `<dir>/vendor/` by invoking
/// `python3 -m pip install -r <requirements> --target ./vendor/`.
///
/// - If `vendor/` already exists, leave it alone — the plugin author may have
///   pre-vendored deliberately for reproducibility.
/// - If the declared requirements file is missing AND `vendor/` is missing,
///   emit a tracing warning and proceed (zero-dep python plugin).
/// - If pip fails, surface its stderr as `PackageError::ArchiveError` so the
///   user sees the resolver/build error directly.
fn vendor_python_deps(dir: &Path, py: &PythonPackageMeta) -> Result<(), PackageError> {
    let vendor_dir = dir.join("vendor");
    if vendor_dir.exists() {
        tracing::debug!(
            vendor = %vendor_dir.display(),
            "pre-existing vendor/ directory — using as-is, skipping pip"
        );
        return Ok(());
    }

    let req_path = dir.join(py.requirements_path());
    if !req_path.exists() {
        tracing::warn!(
            package = %dir.display(),
            requirements = %req_path.display(),
            "python package has no requirements file and no vendor/ — packaging without deps"
        );
        return Ok(());
    }

    tracing::info!(
        requirements = %req_path.display(),
        vendor = %vendor_dir.display(),
        "vendoring python deps via pip"
    );

    // `python3 -m pip` rather than bare `pip` so we use whichever interpreter
    // happens to be on PATH and avoid relying on a separately-installed pip
    // shim. `Command` invokes the binary directly, bypassing shell aliases.
    let output = std::process::Command::new("python3")
        .arg("-m")
        .arg("pip")
        .arg("install")
        .arg("-r")
        .arg(&req_path)
        .arg("--target")
        .arg(&vendor_dir)
        .arg("--quiet")
        .output()
        .map_err(|e| {
            PackageError::ArchiveError(format!(
                "failed to invoke `python3 -m pip` (is python3 on PATH?): {e}"
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PackageError::ArchiveError(format!(
            "pip install failed (exit {}):\n{}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        )));
    }

    Ok(())
}

/// Create a `.fid` archive (tar + bzip2) from a package directory.
///
/// The archive contains a single top-level directory `{name}-{version}/`
/// with all source files. Excludes `target/` and `.git/` directories.
/// Includes `package.sig` if present.
///
/// For Python packages (`runtime = "python"`), if a `requirements.txt` is
/// declared and a `vendor/` directory does not yet exist, `pip install -r
/// <requirements> --target ./vendor/` runs first and the result is included
/// in the archive. Pre-existing `vendor/` is respected and used as-is.
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

    // For Python packages: vendor declared deps into vendor/ before archiving.
    // Pre-existing vendor/ is respected (plugin author may pre-vendor for
    // reproducibility), missing requirements + missing vendor/ produces a
    // tracing warning but is not fatal (a Python plugin with no deps is fine).
    if matches!(pkg.runtime(), PackageRuntime::Python) {
        if let Some(py_meta) = manifest.python.as_ref() {
            vendor_python_deps(dir, py_meta)?;
        }
    }

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

/// Extract a `.fid` archive (tar + bzip2) to a destination directory using
/// strict safety defaults.
///
/// Returns the path to the extracted top-level package directory, which is
/// guaranteed to exist inside `dest` and contain a `package.toml`.
///
/// This function validates every archive entry before extracting and rejects
/// archives containing: path-traversal components (`..`), absolute paths,
/// symlinks, hardlinks, more than 10,000 entries, or a cumulative declared
/// decompressed size exceeding 500 MB or 10× the compressed archive size.
///
/// Extraction is staged inside a temporary directory under `dest` and the
/// package directory is moved into place atomically on success. If validation
/// fails mid-archive, no files are left in `dest`.
///
/// For archives that legitimately exceed the default caps, use
/// [`unpack_package_with_options`].
pub fn unpack_package(archive: &Path, dest: &Path) -> Result<PathBuf, PackageError> {
    unpack_package_with_options(archive, dest, &UnpackOptions::default())
}

/// Extract a `.fid` archive with caller-provided safety limits.
///
/// See [`unpack_package`] for the default-strict variant. Use this when the
/// archive's size or entry count legitimately exceeds the defaults.
pub fn unpack_package_with_options(
    archive: &Path,
    dest: &Path,
    options: &UnpackOptions,
) -> Result<PathBuf, PackageError> {
    use bzip2::read::BzDecoder;
    use std::path::Component;

    let file = std::fs::File::open(archive).map_err(|e| {
        PackageError::ArchiveError(format!("failed to open {}: {e}", archive.display()))
    })?;
    let compressed_size = file.metadata().map(|m| m.len()).unwrap_or(0);

    let decoder = BzDecoder::new(file);
    let mut tar = tar::Archive::new(decoder);

    // Stage extraction inside `dest` so a failed or rejected archive leaves
    // nothing behind. `dest` must already exist.
    std::fs::create_dir_all(dest).map_err(PackageError::Io)?;
    let staging = tempfile::TempDir::new_in(dest).map_err(PackageError::Io)?;
    let staging_path = staging.path();

    let ratio_cap = compressed_size.saturating_mul(options.max_ratio);
    let mut total: u64 = 0;
    let mut count: u32 = 0;

    let entries = tar.entries().map_err(|e| {
        PackageError::ArchiveError(format!("failed to read {}: {e}", archive.display()))
    })?;

    for entry in entries {
        let mut entry = entry.map_err(|e| {
            PackageError::ArchiveError(format!("failed to read archive entry: {e}"))
        })?;

        count = count.saturating_add(1);
        if count > options.max_entries {
            return Err(PackageError::TooManyEntries {
                limit: options.max_entries,
            });
        }

        let path = entry
            .path()
            .map_err(|e| PackageError::ArchiveError(format!("invalid entry path: {e}")))?
            .into_owned();
        let entry_display = path.display().to_string();

        // 1. Reject link entries. A symlink or hardlink followed by a regular
        // file at the same path can overwrite files outside `dest`.
        let entry_type = entry.header().entry_type();
        if entry_type.is_symlink() {
            return Err(PackageError::SymlinkRejected {
                entry: entry_display,
            });
        }
        if entry_type.is_hard_link() {
            return Err(PackageError::HardlinkRejected {
                entry: entry_display,
            });
        }

        // 2. Reject `..` components and absolute paths. The `tar` crate has
        // best-effort guards but they are platform-dependent; check explicitly.
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    return Err(PackageError::PathTraversal {
                        entry: entry_display,
                    });
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(PackageError::AbsolutePath {
                        entry: entry_display,
                    });
                }
                _ => {}
            }
        }

        // 3. Enforce cumulative declared-size budget. Tar's own parsing
        // enforces that actual entry bytes match the declared header size,
        // so trusting the header here is safe against bomb archives.
        let declared = entry.header().size().unwrap_or(0);
        total = total.saturating_add(declared);
        if total > options.max_decompressed {
            return Err(PackageError::SizeLimitExceeded {
                limit: options.max_decompressed,
                actual: total,
            });
        }
        if compressed_size > 0 && options.max_ratio > 0 && total > ratio_cap {
            return Err(PackageError::SizeLimitExceeded {
                limit: ratio_cap,
                actual: total,
            });
        }

        // 4. Extract into the staging area. `unpack_in` itself rejects paths
        // that escape the base directory, but our explicit checks above mean
        // we never get here with a dangerous path.
        entry.unpack_in(staging_path).map_err(|e| {
            PackageError::ArchiveError(format!("failed to extract entry '{}': {e}", path.display()))
        })?;
    }

    // Find the top-level package directory inside staging.
    let mut pkg_dir_staging: Option<PathBuf> = None;
    for entry in std::fs::read_dir(staging_path).map_err(PackageError::Io)? {
        let entry = entry.map_err(PackageError::Io)?;
        let path = entry.path();
        if path.is_dir() && path.join("package.toml").exists() {
            pkg_dir_staging = Some(path);
            break;
        }
    }
    let pkg_dir_staging = pkg_dir_staging.ok_or_else(|| {
        PackageError::InvalidArchive("archive does not contain a package.toml".to_string())
    })?;

    // Atomically move the validated package directory to its final location
    // inside `dest`. If a directory with the same name already exists it is
    // removed first, matching the prior `tar::Archive::unpack` behaviour.
    let pkg_name = pkg_dir_staging
        .file_name()
        .ok_or_else(|| {
            PackageError::InvalidArchive("extracted package has no directory name".to_string())
        })?
        .to_os_string();
    let final_path = dest.join(&pkg_name);
    if final_path.exists() {
        std::fs::remove_dir_all(&final_path).map_err(PackageError::Io)?;
    }
    std::fs::rename(&pkg_dir_staging, &final_path).map_err(PackageError::Io)?;

    // `staging` TempDir drops here; any residual files are cleaned up.
    Ok(final_path)
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
            runtime: None,
        };
        assert_eq!(header.extension(), "fid");

        let header_custom = PackageHeader {
            extension: Some("cloacina".to_string()),
            ..header
        };
        assert_eq!(header_custom.extension(), "cloacina");
    }

    // ---- Python runtime manifest tests ----

    #[test]
    fn rust_runtime_default_when_absent() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            tmp.path(),
            r#"
            [package]
            name = "rust-pkg"
            version = "1.0.0"
            interface = "api"
            interface_version = 1

            [metadata]
            category = "rust"
            "#,
        );
        let m = load_manifest::<TestMeta>(tmp.path()).unwrap();
        assert_eq!(m.package.runtime(), PackageRuntime::Rust);
        assert!(m.python.is_none());
    }

    #[test]
    fn python_runtime_with_python_section_parses() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            tmp.path(),
            r#"
            [package]
            name = "py-pkg"
            version = "0.1.0"
            interface = "api"
            interface_version = 1
            runtime = "python"

            [metadata]
            category = "python"

            [python]
            entry_module = "py_pkg.entry"
            requirements = "deps.txt"
            "#,
        );
        let m = load_manifest::<TestMeta>(tmp.path()).unwrap();
        assert_eq!(m.package.runtime(), PackageRuntime::Python);
        let py = m.python.as_ref().expect("python section");
        assert_eq!(py.entry_module, "py_pkg.entry");
        assert_eq!(py.requirements_path(), "deps.txt");
    }

    #[test]
    fn python_runtime_requirements_default() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            tmp.path(),
            r#"
            [package]
            name = "py-pkg"
            version = "0.1.0"
            interface = "api"
            interface_version = 1
            runtime = "python"

            [metadata]
            category = "python"

            [python]
            entry_module = "py_pkg.entry"
            "#,
        );
        let m = load_manifest::<TestMeta>(tmp.path()).unwrap();
        assert_eq!(
            m.python.as_ref().unwrap().requirements_path(),
            "requirements.txt"
        );
    }

    #[test]
    fn python_runtime_without_python_section_rejected() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            tmp.path(),
            r#"
            [package]
            name = "py-pkg"
            version = "0.1.0"
            interface = "api"
            interface_version = 1
            runtime = "python"

            [metadata]
            category = "python"
            "#,
        );
        let err = load_manifest::<TestMeta>(tmp.path()).unwrap_err();
        match err {
            PackageError::InvalidManifest(msg) => {
                assert!(
                    msg.contains("entry_module"),
                    "expected message about entry_module, got: {msg}"
                );
            }
            other => panic!("expected InvalidManifest, got {other:?}"),
        }
    }

    #[test]
    fn python_section_without_python_runtime_rejected() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            tmp.path(),
            r#"
            [package]
            name = "rust-pkg"
            version = "1.0.0"
            interface = "api"
            interface_version = 1

            [metadata]
            category = "rust"

            [python]
            entry_module = "py_pkg.entry"
            "#,
        );
        let err = load_manifest::<TestMeta>(tmp.path()).unwrap_err();
        assert!(matches!(err, PackageError::InvalidManifest(_)));
    }

    #[test]
    fn unknown_runtime_rejected() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            tmp.path(),
            r#"
            [package]
            name = "node-pkg"
            version = "0.1.0"
            interface = "api"
            interface_version = 1
            runtime = "node"

            [metadata]
            category = "node"
            "#,
        );
        let err = load_manifest::<TestMeta>(tmp.path()).unwrap_err();
        match err {
            PackageError::InvalidManifest(msg) => {
                assert!(msg.contains("node"), "got: {msg}");
            }
            other => panic!("expected InvalidManifest, got {other:?}"),
        }
    }

    #[test]
    fn package_runtime_display_and_str() {
        assert_eq!(PackageRuntime::Rust.as_str(), "rust");
        assert_eq!(PackageRuntime::Python.as_str(), "python");
        assert_eq!(format!("{}", PackageRuntime::Python), "python");
    }

    // ---- Attack-class tests for unpack_package ----

    use bzip2::write::BzEncoder;
    use bzip2::Compression;
    use std::io::Read;
    use tar::{EntryType, Header};

    /// Build a bz2-compressed tar archive from a builder callback.
    fn build_archive<F>(path: &Path, build: F)
    where
        F: FnOnce(&mut tar::Builder<BzEncoder<std::fs::File>>),
    {
        let file = std::fs::File::create(path).unwrap();
        let encoder = BzEncoder::new(file, Compression::default());
        let mut tar = tar::Builder::new(encoder);
        build(&mut tar);
        tar.into_inner().unwrap().finish().unwrap();
    }

    /// Write a raw entry name directly into a GNU tar header, bypassing
    /// `set_path`'s safety validation. This is only safe in tests where we
    /// deliberately craft malicious paths.
    fn write_name(header: &mut Header, path: &str) {
        let gnu = header.as_gnu_mut().expect("gnu header");
        let bytes = path.as_bytes();
        assert!(bytes.len() < gnu.name.len(), "test path too long");
        for slot in gnu.name.iter_mut() {
            *slot = 0;
        }
        gnu.name[..bytes.len()].copy_from_slice(bytes);
    }

    fn write_linkname(header: &mut Header, link: &str) {
        let gnu = header.as_gnu_mut().expect("gnu header");
        let bytes = link.as_bytes();
        assert!(bytes.len() < gnu.linkname.len(), "test linkname too long");
        for slot in gnu.linkname.iter_mut() {
            *slot = 0;
        }
        gnu.linkname[..bytes.len()].copy_from_slice(bytes);
    }

    /// Append a regular file entry with explicit path and content bytes.
    /// Uses the low-level name-writing helper so arbitrary (including
    /// malicious) paths can be tested.
    fn append_regular(tar: &mut tar::Builder<BzEncoder<std::fs::File>>, path: &str, data: &[u8]) {
        let mut header = Header::new_gnu();
        write_name(&mut header, path);
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_entry_type(EntryType::Regular);
        header.set_cksum();
        tar.append(&header, data).unwrap();
    }

    /// Append a link entry with a chosen EntryType (symlink/hardlink).
    fn append_link(
        tar: &mut tar::Builder<BzEncoder<std::fs::File>>,
        path: &str,
        link_target: &str,
        ty: EntryType,
    ) {
        let mut header = Header::new_gnu();
        write_name(&mut header, path);
        write_linkname(&mut header, link_target);
        header.set_size(0);
        header.set_mode(0o644);
        header.set_entry_type(ty);
        header.set_cksum();
        tar.append(&header, std::io::empty()).unwrap();
    }

    #[test]
    fn unpack_rejects_parent_dir_component() {
        let out = TempDir::new().unwrap();
        let archive = out.path().join("evil.fid");
        build_archive(&archive, |tar| {
            append_regular(tar, "../escaped", b"pwn");
        });

        let extract = TempDir::new().unwrap();
        let err = unpack_package(&archive, extract.path()).unwrap_err();
        assert!(
            matches!(err, PackageError::PathTraversal { .. }),
            "expected PathTraversal, got: {err:?}"
        );
        // Nothing leaked outside staging.
        assert!(!out.path().join("escaped").exists());
    }

    #[test]
    fn unpack_rejects_absolute_path() {
        let out = TempDir::new().unwrap();
        let archive = out.path().join("evil.fid");
        build_archive(&archive, |tar| {
            append_regular(tar, "/tmp/fidius-escape", b"pwn");
        });

        let extract = TempDir::new().unwrap();
        let err = unpack_package(&archive, extract.path()).unwrap_err();
        assert!(
            matches!(err, PackageError::AbsolutePath { .. }),
            "expected AbsolutePath, got: {err:?}"
        );
    }

    #[test]
    fn unpack_rejects_symlink() {
        let out = TempDir::new().unwrap();
        let archive = out.path().join("evil.fid");
        build_archive(&archive, |tar| {
            append_link(tar, "link", "/etc/passwd", EntryType::Symlink);
        });

        let extract = TempDir::new().unwrap();
        let err = unpack_package(&archive, extract.path()).unwrap_err();
        assert!(
            matches!(err, PackageError::SymlinkRejected { .. }),
            "expected SymlinkRejected, got: {err:?}"
        );
    }

    #[test]
    fn unpack_rejects_hardlink() {
        let out = TempDir::new().unwrap();
        let archive = out.path().join("evil.fid");
        build_archive(&archive, |tar| {
            append_link(tar, "link", "existing-file", EntryType::Link);
        });

        let extract = TempDir::new().unwrap();
        let err = unpack_package(&archive, extract.path()).unwrap_err();
        assert!(
            matches!(err, PackageError::HardlinkRejected { .. }),
            "expected HardlinkRejected, got: {err:?}"
        );
    }

    #[test]
    fn unpack_symlink_then_file_rejected_at_first_entry() {
        // Classic symlink-overwrite attack: entry 1 is a symlink to /tmp/foo,
        // entry 2 is a regular file at the same path. Our checks reject entry 1
        // so entry 2 is never extracted.
        let out = TempDir::new().unwrap();
        let sentinel_dir = TempDir::new().unwrap();
        let sentinel = sentinel_dir.path().join("target");
        std::fs::write(&sentinel, b"original").unwrap();

        let archive = out.path().join("evil.fid");
        build_archive(&archive, |tar| {
            append_link(tar, "bad", sentinel.to_str().unwrap(), EntryType::Symlink);
            append_regular(tar, "bad", b"clobber");
        });

        let extract = TempDir::new().unwrap();
        let err = unpack_package(&archive, extract.path()).unwrap_err();
        assert!(matches!(err, PackageError::SymlinkRejected { .. }));

        // The sentinel file outside the extraction directory is untouched.
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"original");
    }

    #[test]
    fn unpack_rejects_declared_size_bomb() {
        let out = TempDir::new().unwrap();
        let archive = out.path().join("bomb.fid");

        // Build a tar manually where a header declares a size far above the cap,
        // then write matching zero bytes so tar parsing stays consistent.
        let file = std::fs::File::create(&archive).unwrap();
        let encoder = BzEncoder::new(file, Compression::best());
        let mut tar = tar::Builder::new(encoder);

        let declared: u64 = 600 * 1024 * 1024; // > 500 MB default cap
        let mut header = Header::new_gnu();
        header.set_path("bomb.bin").unwrap();
        header.set_size(declared);
        header.set_mode(0o644);
        header.set_entry_type(EntryType::Regular);
        header.set_cksum();

        // Use a zero-filled reader so the compressed size stays tiny.
        let zeros = std::io::repeat(0u8).take(declared);
        tar.append(&header, zeros).unwrap();
        tar.into_inner().unwrap().finish().unwrap();

        let extract = TempDir::new().unwrap();
        let err = unpack_package(&archive, extract.path()).unwrap_err();
        assert!(
            matches!(err, PackageError::SizeLimitExceeded { .. }),
            "expected SizeLimitExceeded, got: {err:?}"
        );
    }

    #[test]
    fn unpack_rejects_ratio_bomb() {
        // Small compressed archive with many small entries whose cumulative
        // declared size exceeds `compressed_size * max_ratio` but is still
        // under the absolute cap — should be rejected by the ratio check.
        let out = TempDir::new().unwrap();
        let archive = out.path().join("ratio.fid");

        // Default max_ratio is 10. Use a 4 KB-per-entry file that compresses well.
        let payload = vec![b'A'; 4096];
        build_archive(&archive, |tar| {
            for i in 0..10_000u32 {
                append_regular(tar, &format!("file-{i:05}.txt"), &payload);
            }
        });

        let extract = TempDir::new().unwrap();
        // Tighten both caps so this triggers on ratio rather than absolute cap.
        let options = UnpackOptions {
            max_decompressed: u64::MAX,
            max_ratio: 2,
            max_entries: 20_000,
        };
        let err = unpack_package_with_options(&archive, extract.path(), &options).unwrap_err();
        assert!(
            matches!(err, PackageError::SizeLimitExceeded { .. }),
            "expected SizeLimitExceeded, got: {err:?}"
        );
    }

    #[test]
    fn unpack_rejects_too_many_entries() {
        let out = TempDir::new().unwrap();
        let archive = out.path().join("spam.fid");
        build_archive(&archive, |tar| {
            for i in 0..50u32 {
                append_regular(tar, &format!("f-{i}"), b"");
            }
        });

        let extract = TempDir::new().unwrap();
        let options = UnpackOptions {
            max_entries: 10,
            ..UnpackOptions::default()
        };
        let err = unpack_package_with_options(&archive, extract.path(), &options).unwrap_err();
        assert!(
            matches!(err, PackageError::TooManyEntries { limit: 10 }),
            "expected TooManyEntries, got: {err:?}"
        );
    }

    #[test]
    fn unpack_staging_cleans_up_on_rejection() {
        let out = TempDir::new().unwrap();
        let archive = out.path().join("evil.fid");
        build_archive(&archive, |tar| {
            append_regular(tar, "ok/file.txt", b"ok");
            append_regular(tar, "../escape", b"bad");
        });

        let extract = TempDir::new().unwrap();
        let _ = unpack_package(&archive, extract.path()).unwrap_err();

        // After rejection `extract` must be empty — the partial `ok/` tree
        // lived in a TempDir that has since been dropped.
        let remaining: Vec<_> = std::fs::read_dir(extract.path())
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(
            remaining.is_empty(),
            "extraction dir not cleaned up: {remaining:?}"
        );
    }

    #[test]
    fn unpack_with_options_accepts_large_archive() {
        // Round-trip a legitimate package under a looser cap to exercise the
        // options path end-to-end.
        let pkg_dir = TempDir::new().unwrap();
        make_package(pkg_dir.path());

        let out_dir = TempDir::new().unwrap();
        let fid_path = out_dir.path().join("ok.fid");
        pack_package(pkg_dir.path(), Some(&fid_path)).unwrap();

        let extract = TempDir::new().unwrap();
        let options = UnpackOptions {
            max_decompressed: u64::MAX,
            max_ratio: u64::MAX,
            max_entries: u32::MAX,
        };
        let extracted = unpack_package_with_options(&fid_path, extract.path(), &options).unwrap();
        assert!(extracted.join("package.toml").exists());
    }

    // ---- Python pack-time vendoring tests ----

    /// Build a minimal Python package directory (manifest + entry .py).
    fn make_python_package(dir: &Path, with_requirements: Option<&str>) {
        let req_line = if with_requirements.is_some() {
            "requirements = \"requirements.txt\"\n"
        } else {
            ""
        };
        write_manifest(
            dir,
            &format!(
                r#"
                [package]
                name = "py-pack-test"
                version = "0.1.0"
                interface = "api"
                interface_version = 1
                runtime = "python"

                [metadata]
                category = "python"

                [python]
                entry_module = "py_pack_test"
                {req_line}
                "#
            ),
        );
        std::fs::write(
            dir.join("py_pack_test.py"),
            b"def hello():\n    return 'hi'\n",
        )
        .unwrap();
        if let Some(req) = with_requirements {
            std::fs::write(dir.join("requirements.txt"), req.as_bytes()).unwrap();
        }
    }

    #[test]
    fn pack_python_with_prevendored_directory_skips_pip() {
        // If vendor/ is present we don't invoke pip — even with a requirements
        // file pointing at something pip couldn't possibly resolve. Simulating
        // pre-vendoring by hand.
        let pkg_dir = TempDir::new().unwrap();
        make_python_package(
            pkg_dir.path(),
            Some("definitely-not-a-real-package==999.999.999"),
        );
        let vendor = pkg_dir.path().join("vendor");
        std::fs::create_dir(&vendor).unwrap();
        std::fs::write(
            vendor.join("fake_module.py"),
            b"# pre-vendored placeholder\n",
        )
        .unwrap();

        let out_dir = TempDir::new().unwrap();
        let fid = out_dir.path().join("py.fid");
        pack_package(pkg_dir.path(), Some(&fid))
            .expect("pack should not invoke pip when vendor/ exists");

        let extract = TempDir::new().unwrap();
        let extracted = unpack_package(&fid, extract.path()).unwrap();
        assert!(extracted.join("vendor/fake_module.py").exists());
        assert!(extracted.join("py_pack_test.py").exists());
    }

    #[test]
    fn pack_python_with_no_requirements_or_vendor_warns_but_succeeds() {
        let pkg_dir = TempDir::new().unwrap();
        make_python_package(pkg_dir.path(), None);

        let out_dir = TempDir::new().unwrap();
        let fid = out_dir.path().join("py.fid");
        pack_package(pkg_dir.path(), Some(&fid))
            .expect("zero-dep python plugin should pack successfully");

        let extract = TempDir::new().unwrap();
        let extracted = unpack_package(&fid, extract.path()).unwrap();
        assert!(extracted.join("py_pack_test.py").exists());
        assert!(!extracted.join("vendor").exists());
    }

    #[test]
    fn pack_python_with_unresolvable_requirement_surfaces_pip_error() {
        // Pip is genuinely invoked here — needs python3+pip on PATH. The test
        // is testing the failure-surfacing path: we deliberately ask pip to
        // install a package that doesn't exist and assert the error is clear.
        // Skipped if python3/pip aren't reachable so CI environments without
        // them don't fail.
        let probe = std::process::Command::new("python3")
            .arg("-m")
            .arg("pip")
            .arg("--version")
            .output();
        if probe.map(|o| !o.status.success()).unwrap_or(true) {
            eprintln!("skipping: python3 -m pip not available in this environment");
            return;
        }

        let pkg_dir = TempDir::new().unwrap();
        make_python_package(
            pkg_dir.path(),
            Some("fidius-this-package-does-not-exist-9999==1.0\n"),
        );

        let out_dir = TempDir::new().unwrap();
        let fid = out_dir.path().join("py.fid");
        let err = pack_package(pkg_dir.path(), Some(&fid)).unwrap_err();
        match err {
            PackageError::ArchiveError(msg) => {
                assert!(
                    msg.contains("pip install failed"),
                    "expected pip-install error, got: {msg}"
                );
            }
            other => panic!("expected ArchiveError, got {other:?}"),
        }
    }
}
