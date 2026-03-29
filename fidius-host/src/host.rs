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
use fidius_core::descriptor::{BufferStrategyKind, WireFormat};

use crate::error::LoadError;
use crate::loader::{self, LoadedPlugin};
use crate::signing;
use crate::types::{LoadPolicy, PluginInfo};

/// Host for loading and managing plugins.
pub struct PluginHost {
    search_paths: Vec<PathBuf>,
    load_policy: LoadPolicy,
    require_signature: bool,
    trusted_keys: Vec<VerifyingKey>,
    expected_hash: Option<u64>,
    expected_wire: Option<WireFormat>,
    expected_strategy: Option<BufferStrategyKind>,
}

/// Builder for configuring a PluginHost.
pub struct PluginHostBuilder {
    search_paths: Vec<PathBuf>,
    load_policy: LoadPolicy,
    require_signature: bool,
    trusted_keys: Vec<VerifyingKey>,
    expected_hash: Option<u64>,
    expected_wire: Option<WireFormat>,
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
            expected_wire: None,
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

    /// Set the expected wire format for validation.
    pub fn wire_format(mut self, format: WireFormat) -> Self {
        self.expected_wire = Some(format);
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
            expected_wire: self.expected_wire,
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
    /// Scans directories for dylib files, loads each, validates,
    /// and returns metadata for all valid plugins found.
    pub fn discover(&self) -> Result<Vec<PluginInfo>, LoadError> {
        let mut plugins = Vec::new();

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

                match loader::load_library(&path) {
                    Ok(loaded) => {
                        for plugin in &loaded.plugins {
                            if let Ok(()) = loader::validate_against_interface(
                                plugin,
                                self.expected_hash,
                                self.expected_wire,
                                self.expected_strategy,
                            ) {
                                plugins.push(plugin.info.clone());
                            }
                        }
                    }
                    Err(_) => {
                        // Skip invalid dylibs during discovery
                        continue;
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// Load a specific plugin by name.
    ///
    /// Searches all configured paths for a dylib containing a plugin
    /// with the given name. Returns the loaded plugin ready for calling.
    pub fn load(&self, name: &str) -> Result<LoadedPlugin, LoadError> {
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

                // Verify signature if required
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
                                    self.expected_wire,
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
