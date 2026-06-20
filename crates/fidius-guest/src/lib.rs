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

//! `fidius-guest` — the wasm-buildable subset of the Fidius shared types.
//!
//! Extracted from `fidius-core` (FIDIUS-I-0022) so plugin guests — including
//! Rust WASM components built from `#[plugin_impl]` — can depend on the
//! interface hashing, descriptors, the neutral [`value::Value`] model, and the
//! bincode wire format **without** dragging in the host-only packaging stack
//! (archive/compression/filesystem). This crate has no such dependencies and
//! compiles cleanly to `wasm32-wasip2`.
//!
//! `fidius-core` re-exports every module here, so `fidius_core::descriptor`,
//! `fidius_core::hash`, `fidius_core::value`, … (and the `fidius` facade
//! re-exports) resolve unchanged — this split is internal, with no public-API
//! churn.
//!
//! Note: `descriptor::ABI_VERSION` derives from this crate's `CARGO_PKG_VERSION`
//! (per ADR-0002), so `fidius-guest` is versioned in lockstep with `fidius-core`.

pub mod descriptor;
pub mod error;
pub mod frame;
pub mod hash;
/// Brokered outbound HTTP for sandboxed WASM connectors (FIDIUS-I-0028).
/// Only present in components built for `wasm32-wasip2`.
#[cfg(target_family = "wasm")]
pub mod http;
pub mod python_descriptor;
pub mod status;
pub mod stream_ffi;
pub mod stream_marker;
pub mod value;
pub mod wasm_descriptor;
pub mod wire;

pub use descriptor::*;
pub use error::PluginError;
pub use frame::{Frame, FrameError};
pub use status::*;
pub use stream_ffi::FidiusStreamHandle;
pub use stream_marker::Stream;
pub use value::{from_value, to_value, Value, ValueError};
