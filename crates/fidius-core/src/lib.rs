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

// Host-only modules (archive/compression/filesystem, inventory collection).
pub mod package;
pub mod registry;

#[cfg(feature = "async")]
pub mod async_runtime;

// Guest-essential, wasm-buildable modules now live in `fidius-guest`. They are
// re-exported here so every existing `fidius_core::*` path (and the `fidius`
// facade re-exports) resolves unchanged — the split (FIDIUS-I-0022) is internal.
pub use fidius_guest::{
    descriptor, error, frame, hash, python_descriptor, status, stream_ffi, stream_marker, value,
    wasm_descriptor, wire,
};

/// Brokered outbound HTTP for sandboxed WASM connectors (FIDIUS-I-0028) —
/// present only in `wasm32-wasip2` builds.
#[cfg(target_family = "wasm")]
pub use fidius_guest::http;

pub use descriptor::*;
pub use error::PluginError;
pub use status::*;
pub use stream_marker::Stream;
pub use value::{from_value, to_value, Value, ValueError};

// Re-export inventory so generated code can reference it via fidius_core::inventory
pub use inventory;
