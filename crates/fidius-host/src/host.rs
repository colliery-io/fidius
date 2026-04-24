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

//! PluginHost builder and plugin discovery.

use std::path::{Path, PathBuf};

use ed25519_dalek::VerifyingKey;
use fidius_core::descriptor::BufferStrategyKind;

use crate::error::LoadError;
use crate::loader::{self, LoadedPlugin};
use crate::signing;
use crate::types::{LoadPolicy, PluginInfo, PluginRuntimeKind};

/// Host for loading and managing plugins.
#[allow(dead_code)] // load_policy will be used for non-security validation (hash/version lenient)
pub struct PluginHost {
    search_paths: Vec<PathBuf>,
    load_policy: LoadPolicy,
    require_signature: bool,
    trusted_keys: Vec<VerifyingKey>,
    expected_hash: Option<u64>,
    expected_strategy: Option<BufferStrategyKind>,
}

/// Builder for configuring a PluginHost.
pub struct PluginHostBuilder {
    search_paths: Vec<PathBuf>,
    load_policy: LoadPolicy,
    require_signature: bool,
    trusted_keys: Vec<VerifyingKey>,
    expected_hash: Option<u64>,
    expected_strategy: Option<BufferStrategyKind>,
}

impl PluginHostBuilder {
    fn new() -> Self {
        Self {
            search_paths: Vec::new(),
            load_policy: LoadPolicy::Strict,
            require_signature: false,
            trusted_keys: Vec::new(),
            expected_hash: None,
            expected_strategy: None,
        }
    }

    /// Add a directory to search for plugin dylibs.
    pub fn search_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.search_paths.push(path.into());
        self
    }

    /// Set the load policy (Strict or Lenient).
    pub fn load_policy(mut self, policy: LoadPolicy) -> Self {
        self.load_policy = policy;
        self
    }

    /// Require plugins to have valid signatures.
    pub fn require_signature(mut self, require: bool) -> Self {
        self.require_signature = require;
        self
    }

    /// Set trusted Ed25519 public keys for signature verification.
    pub fn trusted_keys(mut self, keys: &[VerifyingKey]) -> Self {
        self.trusted_keys = keys.to_vec();
        self
    }

    /// Set the expected interface hash for validation.
    pub fn interface_hash(mut self, hash: u64) -> Self {
        self.expected_hash = Some(hash);
        self
    }

    /// Set the expected buffer strategy for validation.
    pub fn buffer_strategy(mut self, strategy: BufferStrategyKind) -> Self {
        self.expected_strategy = Some(strategy);
        self
    }

    /// Build the PluginHost.
    pub fn build(self) -> Result<PluginHost, LoadError> {
        Ok(PluginHost {
            search_paths: self.search_paths,
            load_policy: self.load_policy,
            require_signature: self.require_signature,
            trusted_keys: self.trusted_keys,
            expected_hash: self.expected_hash,
            expected_strategy: self.expected_strategy,
        })
    }
}

impl PluginHost {
    /// Create a new builder.
    pub fn builder() -> PluginHostBuilder {
        PluginHostBuilder::new()
    }

    /// Discover all valid plugins in the configured search paths.
    ///
    /// Scans each path for both:
    /// - dylib files (cdylib plugins, the existing path), and
    /// - subdirectories containing a `package.toml` with `runtime = "python"`
    ///   (when the `python` feature is enabled).
    ///
    /// Returns owned `PluginInfo` for every valid plugin found, with
    /// `PluginInfo::runtime` distinguishing the two kinds.
    pub fn discover(&self) -> Result<Vec<PluginInfo>, LoadError> {
        #[cfg(feature = "tracing")]
        tracing::info!(search_paths = ?self.search_paths, "discovering plugins");

        let mut plugins = Vec::new();

        for search_path in &self.search_paths {
            if !search_path.is_dir() {
                continue;
            }

            let entries = std::fs::read_dir(search_path)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if is_dylib(&path) {
                    self.discover_cdylib(&path, &mut plugins);
                } else if path.is_dir() && path.join("package.toml").exists() {
                    self.discover_python_package(&path, &mut plugins);
                }
            }
        }

        Ok(plugins)
    }

    fn discover_cdylib(&self, path: &Path, plugins: &mut Vec<PluginInfo>) {
        // Verify signature before dlopen to prevent code execution from untrusted dylibs
        if self.require_signature && signing::verify_signature(path, &self.trusted_keys).is_err() {
            return;
        }

        let Ok(loaded) = loader::load_library(path) else {
            return; // Skip invalid dylibs during discovery
        };
        for plugin in &loaded.plugins {
            if loader::validate_against_interface(
                plugin,
                self.expected_hash,
                self.expected_strategy,
            )
            .is_ok()
            {
                plugins.push(plugin.info.clone());
            }
        }
    }

    fn discover_python_package(&self, dir: &Path, plugins: &mut Vec<PluginInfo>) {
        let Ok(manifest) = fidius_core::package::load_manifest_untyped(dir) else {
            return;
        };
        if !matches!(
            manifest.package.runtime(),
            fidius_core::package::PackageRuntime::Python
        ) {
            return;
        }
        plugins.push(PluginInfo {
            name: manifest.package.name.clone(),
            interface_name: manifest.package.interface.clone(),
            // Hash is unknown until load (the host validates against the
            // descriptor at load time, not discovery). Surface 0 so callers
            // know discovery alone hasn't validated the package.
            interface_hash: 0,
            interface_version: manifest.package.interface_version,
            capabilities: 0,
            buffer_strategy: BufferStrategyKind::PluginAllocated,
            runtime: PluginRuntimeKind::Python,
        });
    }

    /// Load a specific plugin by name.
    ///
    /// Searches all configured paths for a dylib containing a plugin
    /// with the given name. Returns the loaded plugin ready for calling.
    pub fn load(&self, name: &str) -> Result<LoadedPlugin, LoadError> {
        #[cfg(feature = "tracing")]
        tracing::info!(plugin_name = name, "loading plugin");

        for search_path in &self.search_paths {
            if !search_path.is_dir() {
                continue;
            }

            let entries = std::fs::read_dir(search_path)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if !is_dylib(&path) {
                    continue;
                }

                // Verify signature if required — always enforced regardless of LoadPolicy
                if self.require_signature {
                    signing::verify_signature(&path, &self.trusted_keys)?;
                }

                match loader::load_library(&path) {
                    Ok(loaded) => {
                        for plugin in loaded.plugins {
                            if plugin.info.name == name {
                                loader::validate_against_interface(
                                    &plugin,
                                    self.expected_hash,
                                    self.expected_strategy,
                                )?;
                                return Ok(plugin);
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        }

        Err(LoadError::PluginNotFound {
            name: name.to_string(),
        })
    }

    /// Find a python plugin package directory by name across the configured
    /// search paths. The plugin name is matched against `package.toml`'s
    /// `[package].name`. Returns the directory path on success.
    pub fn find_python_package(&self, name: &str) -> Result<PathBuf, LoadError> {
        for search_path in &self.search_paths {
            if !search_path.is_dir() {
                continue;
            }
            let entries = std::fs::read_dir(search_path)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                if !path.join("package.toml").exists() {
                    continue;
                }
                let Ok(manifest) = fidius_core::package::load_manifest_untyped(&path) else {
                    continue;
                };
                if matches!(
                    manifest.package.runtime(),
                    fidius_core::package::PackageRuntime::Python
                ) && manifest.package.name == name
                {
                    return Ok(path);
                }
            }
        }
        Err(LoadError::PluginNotFound {
            name: name.to_string(),
        })
    }

    /// Load a Python plugin package by name and validate it against the
    /// supplied interface descriptor.
    ///
    /// The caller passes the static `<TraitName>_PYTHON_DESCRIPTOR` emitted
    /// by the interface crate's `#[plugin_interface]` macro — that's the
    /// out-of-band hint the loader needs to map method names to vtable
    /// indices and to check the interface hash.
    ///
    /// Available only when fidius-host is built with the `python` feature.
    #[cfg(feature = "python")]
    pub fn load_python(
        &self,
        name: &str,
        descriptor: &'static fidius_core::python_descriptor::PythonInterfaceDescriptor,
    ) -> Result<fidius_python::PythonPluginHandle, LoadError> {
        let dir = self.find_python_package(name)?;
        fidius_python::load_python_plugin(&dir, descriptor)
            .map_err(|e| LoadError::PythonLoad(e.to_string()))
    }
}

/// Check if a path has a platform-appropriate dylib extension.
fn is_dylib(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if cfg!(target_os = "macos") {
        ext == "dylib"
    } else if cfg!(target_os = "windows") {
        ext == "dll"
    } else {
        ext == "so"
    }
}
