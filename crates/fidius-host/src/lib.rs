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

pub mod arch;
pub mod arena;
pub mod error;
pub mod handle;
pub mod host;
pub mod loader;
pub mod package;
pub mod signing;
pub mod types;

pub use error::{CallError, LoadError};
pub use handle::PluginHandle;
pub use host::PluginHost;
pub use loader::{LoadedLibrary, LoadedPlugin};
pub use types::{LoadPolicy, PluginInfo, PluginRuntimeKind};
