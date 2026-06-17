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

//! `PluginHandle` — the unified, caller-facing proxy over a loaded plugin.
//!
//! A `PluginHandle` is backend-agnostic: callers use the same
//! `call_method` / `call_method_raw` API whether the plugin is a cdylib, a
//! Python package, or (Phase 2) a WASM component. The backend lives in the
//! private [`Backend`] enum.
//!
//! ## Why an enum backend (FIDIUS-I-0021)
//!
//! The backends don't share a typed wire: cdylib decodes concrete-type
//! **bincode** (not reconstructable from an erased value) while Python/WASM
//! consume a self-describing [`fidius_core::Value`]. An enum (rather than
//! `Box<dyn PluginExecutor>`) lets the generic `call_method<I, O>` branch with
//! the concrete `I`/`O` in scope and serialise with each backend's native
//! currency — so the **cdylib path stays byte-identical** to before this
//! refactor (`bincode(input)` straight to the FFI; `Value` is never involved).

use serde::de::DeserializeOwned;
use serde::Serialize;

use fidius_core::descriptor::PluginDescriptor;

use crate::error::{CallError, LoadError};
use crate::executor::cdylib::CdylibExecutor;
#[cfg(feature = "python")]
use crate::executor::python::Pyo3Executor;
#[cfg(feature = "python")]
use crate::executor::{PluginExecutor, ValueExecutor};
use crate::types::PluginInfo;

/// The execution backend behind a [`PluginHandle`].
///
/// One variant per runtime. The WASM variant lands in Phase 2.
enum Backend {
    Cdylib(CdylibExecutor),
    /// `.py` package via `fidius-python`'s embedded interpreter. Only present
    /// when the `python` feature is enabled.
    #[cfg(feature = "python")]
    Python(Pyo3Executor),
}

/// A handle to a loaded plugin, ready for calling methods.
///
/// Holds the active execution backend. `call_method()` handles serialization,
/// dispatch, and cleanup; concurrent calls from multiple threads are safe as
/// long as the underlying plugin is thread-safe (the cdylib macro enforces
/// `&self`-only methods; the Python backend serialises through the GIL).
pub struct PluginHandle {
    backend: Backend,
}

impl PluginHandle {
    /// Create a `PluginHandle` from a freshly loaded cdylib plugin.
    pub fn from_loaded(plugin: crate::loader::LoadedPlugin) -> Self {
        Self {
            backend: Backend::Cdylib(CdylibExecutor::from_loaded(plugin)),
        }
    }

    /// Create a `PluginHandle` from a descriptor already registered in the
    /// current process's inventory (a `#[plugin_impl]` linked as a normal
    /// rlib). No dylib is loaded. Used by `Client::in_process(plugin_name)`.
    pub fn from_descriptor(desc: &'static PluginDescriptor) -> Result<Self, LoadError> {
        Ok(Self {
            backend: Backend::Cdylib(CdylibExecutor::from_descriptor(desc)?),
        })
    }

    /// Look up a descriptor in the current process's inventory registry by
    /// `plugin_name` (the Rust struct name passed to `#[plugin_impl]`).
    pub fn find_in_process_descriptor(
        plugin_name: &str,
    ) -> Result<&'static PluginDescriptor, LoadError> {
        CdylibExecutor::find_in_process_descriptor(plugin_name)
    }

    /// Create a `PluginHandle` backed by a loaded Python plugin. `info` is
    /// built by the loader from the package manifest + interface descriptor.
    /// Only available with the `python` feature.
    #[cfg(feature = "python")]
    pub fn from_python(py: fidius_python::PythonPluginHandle, info: PluginInfo) -> Self {
        Self {
            backend: Backend::Python(Pyo3Executor::new(py, info)),
        }
    }

    /// Call a plugin method by vtable index.
    ///
    /// Serializes the input with the backend's native wire (cdylib → bincode;
    /// Python/WASM → [`fidius_core::Value`]), dispatches, and decodes the
    /// result into `O`. No built-in timeout — see the `fidius` crate docs.
    pub fn call_method<I: Serialize, O: DeserializeOwned>(
        &self,
        index: usize,
        input: &I,
    ) -> Result<O, CallError> {
        match &self.backend {
            // cdylib: serialise the concrete type with bincode directly — byte
            // for byte what the plugin's shim decodes (no `Value` hop).
            Backend::Cdylib(e) => e.call_method(index, input),
            // python: cross via the self-describing `Value` currency.
            #[cfg(feature = "python")]
            Backend::Python(e) => {
                let args = fidius_core::to_value(input)
                    .map_err(|err| CallError::Serialization(err.to_string()))?;
                let out = ValueExecutor::call(e, index, args)?;
                fidius_core::from_value(out)
                    .map_err(|err| CallError::Deserialization(err.to_string()))
            }
        }
    }

    /// Call a `#[wire(raw)]` method: raw bytes in, raw bytes out, no bincode.
    pub fn call_method_raw(&self, index: usize, input: &[u8]) -> Result<Vec<u8>, CallError> {
        match &self.backend {
            Backend::Cdylib(e) => e.call_method_raw(index, input),
            #[cfg(feature = "python")]
            Backend::Python(e) => PluginExecutor::call_raw(e, index, input),
        }
    }

    /// Check if an optional method is supported (capability bit set).
    /// Returns `false` for `bit >= 64` and for backends without capabilities.
    pub fn has_capability(&self, bit: u32) -> bool {
        if bit >= 64 {
            return false;
        }
        self.info().capabilities & (1u64 << bit) != 0
    }

    /// Access the plugin's owned metadata.
    pub fn info(&self) -> &PluginInfo {
        match &self.backend {
            Backend::Cdylib(e) => e.info(),
            #[cfg(feature = "python")]
            Backend::Python(e) => PluginExecutor::info(e),
        }
    }

    /// Static `#[method_meta(...)]` key/value metadata for the given method,
    /// in declaration order. Empty for out-of-range ids, for interfaces that
    /// declared none, and for backends without descriptor metadata.
    pub fn method_metadata(&self, method_id: u32) -> Vec<(&str, &str)> {
        match &self.backend {
            Backend::Cdylib(e) => e.method_metadata(method_id),
            // Python plugins carry no descriptor-level method metadata.
            #[cfg(feature = "python")]
            Backend::Python(_) => Vec::new(),
        }
    }

    /// Static `#[trait_meta(...)]` key/value metadata declared on the trait.
    /// Empty when none was declared or for backends without descriptor metadata.
    pub fn trait_metadata(&self) -> Vec<(&str, &str)> {
        match &self.backend {
            Backend::Cdylib(e) => e.trait_metadata(),
            #[cfg(feature = "python")]
            Backend::Python(_) => Vec::new(),
        }
    }
}
