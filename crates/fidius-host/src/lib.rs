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
#[cfg(feature = "streaming")]
pub mod client_stream;
pub mod error;
pub mod executor;
pub mod handle;
pub mod host;
pub mod loader;
pub mod package;
pub mod signing;
#[cfg(feature = "streaming")]
pub mod stream;
pub mod types;

pub use error::{CallError, LoadError};
pub use executor::PluginExecutor;
pub use handle::PluginHandle;
pub use host::{PluginHost, PluginHostBuilder};
pub use loader::{LoadedLibrary, LoadedPlugin};
#[cfg(feature = "streaming")]
pub use stream::{ChunkStream, StreamExecutor};
pub use types::{LoadPolicy, PluginInfo, PluginRuntimeKind};
// The WASM egress contract (FIDIUS-I-0027): an embedder names these to implement an
// `EgressPolicy` for `PluginHost::builder().egress(..)`. Lifted to the crate root so
// downstreams (incl. the `fidius` facade) can name them without the internal module path.
#[cfg(feature = "wasm")]
pub use executor::wasm::{EgressDenied, EgressPolicy};
// The `http` crate, re-exported so an embedder can name `http::request::Parts` (+ `Uri`,
// `HeaderMap`, …) in their `EgressPolicy::authorize` impl without a separate `http` dep.
#[cfg(feature = "wasm")]
pub use ::http as http_types;
