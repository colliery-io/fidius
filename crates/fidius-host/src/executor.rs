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

//! `PluginExecutor` — the dispatch seam across execution backends.
//!
//! Fidius historically carried one dispatch implementation per backend: the
//! cdylib vtable/FFI path lived inside `PluginHandle`, and the Python (PyO3)
//! path lived in a *separate* `PythonPluginHandle` in `fidius-python`. This
//! module collapses that duplication: each backend is an executor, and the
//! caller-facing [`crate::handle::PluginHandle`] wraps them in an **enum**
//! (`Backend`) so its generic `call_method<I, O>` can serialise with each
//! backend's native currency.
//!
//! ## Why an enum, not `Box<dyn>` — and why two traits
//!
//! The backends do **not** share a typed wire. cdylib decodes concrete-type
//! **bincode** (not self-describing — it can't be reconstructed from an erased
//! value), while Python (and the future WASM component backend) consume a
//! self-describing [`fidius_core::Value`]. A single `call(method, Value)` trait
//! method therefore cannot serve cdylib without breaking its ABI (see
//! FIDIUS-A-0003 / FIDIUS-I-0021 amendment). So:
//!
//! - [`PluginExecutor`] is the **common** surface every backend shares:
//!   metadata plus the raw byte path. For cdylib, `call_raw` is *also* the
//!   carrier for typed calls (the wrapper bincode-wraps the concrete type).
//! - [`ValueExecutor`] adds the typed [`fidius_core::Value`] call, implemented
//!   only by the self-describing backends (Python, WASM). cdylib does not
//!   implement it — `PluginHandle` routes cdylib typed calls through its own
//!   bincode `call_method`, keeping the bytes byte-identical to pre-refactor.

pub mod cdylib;
#[cfg(feature = "python")]
pub mod python;
#[cfg(feature = "wasm")]
pub mod wasm;

use fidius_core::Value;

use crate::error::CallError;
use crate::types::PluginInfo;

pub use cdylib::CdylibExecutor;
#[cfg(feature = "python")]
pub use python::Pyo3Executor;
#[cfg(feature = "wasm")]
pub use wasm::{
    precompile_component, validate_component, EgressDenied, EgressPolicy, WasmComponentExecutor,
    WasmMethod,
};

/// The surface every execution backend shares.
///
/// Implementations must be `Send + Sync`: methods take `&self`, so a handle can
/// be shared across threads as long as the backend is internally thread-safe.
pub trait PluginExecutor: Send + Sync {
    /// Owned metadata describing the loaded plugin.
    fn info(&self) -> &PluginInfo;

    /// Number of methods the plugin exposes, in interface (vtable) order.
    fn method_count(&self) -> u32;

    /// Raw bulk-bytes dispatch for `#[wire(raw)]` methods: opaque bytes in,
    /// opaque bytes out, no per-element marshalling. Opaque bytes are
    /// language-neutral (a WIT `list<u8>`), so this is uniform across backends.
    fn call_raw(&self, method: usize, input: &[u8]) -> Result<Vec<u8>, CallError>;
}

/// Backends whose typed boundary is the self-describing [`Value`] model —
/// Python today, and the Phase-2 WASM component executor.
///
/// cdylib deliberately does **not** implement this: its typed path is
/// concrete-type bincode, which `Value` cannot reproduce. `PluginHandle`
/// dispatches cdylib typed calls directly via bincode instead.
pub trait ValueExecutor: PluginExecutor {
    /// Typed dispatch by method index. Arguments and returns cross as a
    /// self-describing [`Value`]; the backend maps it to its native form
    /// (Python → `PyObject`, WASM → `component::Val`).
    fn call(&self, method: usize, args: Value) -> Result<Value, CallError>;
}
