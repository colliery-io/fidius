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

//! Python plugin runtime for Fidius.
//!
//! This crate embeds CPython into the host process via PyO3 and (in later
//! tasks) exposes a `PluginHandle` whose dispatcher calls into a loaded
//! Python module. Hosts opt into Python plugin support by depending on this
//! crate (typically through `fidius-host`'s `python` feature flag).
//!
//! At this stage the crate provides only the foundation: shared interpreter
//! initialisation and Python-exception-to-`PluginError` conversion. The
//! loader, dispatcher, and packaging integration land in subsequent tasks
//! under FIDIUS-I-0020.

pub mod error;
pub mod handle;
pub mod interpreter;
pub mod loader;
pub mod value_bridge;

pub use error::pyerr_to_plugin_error;
pub use handle::{PythonCallError, PythonPluginHandle};
pub use interpreter::ensure_initialized;
pub use loader::{load_python_plugin, PythonLoadError};
