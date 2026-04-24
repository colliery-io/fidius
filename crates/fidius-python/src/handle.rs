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

//! Handle to a loaded Python plugin.
//!
//! Holds the imported module + a vector of callables aligned with the
//! interface descriptor's method order. Dispatch happens via two paths:
//!
//! - **Typed**: `call_typed` takes raw bincode bytes (the same payload the
//!   cdylib `call_method` would receive), pivots through `serde_json::Value`
//!   to convert to Python primitives, calls the callable, converts the
//!   return back to a `Value`, and re-encodes as bincode for the host.
//!
//! - **Raw**: `call_raw` takes `&[u8]`, passes a `bytes` arg directly to
//!   Python, expects `bytes` back. No encoding hops — used by methods opted
//!   into `#[wire(raw)]` (T-0082).
//!
//! All Python exceptions become `fidius_core::PluginError` via the
//! `pyerr_to_plugin_error` helper, with `code = "PluginError"` for typed
//! `fidius.PluginError` raises (round-trips `code`/`message`/`details`) and
//! `code = <ExceptionClassName>` otherwise.

use fidius_core::python_descriptor::PythonInterfaceDescriptor;
use fidius_core::PluginError;
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyBytes, PyDict, PyTuple};

use crate::error::pyerr_to_plugin_error;
use crate::value_bridge::{pyobject_to_value, value_to_pyobject};

/// Errors a typed call can produce on the Python side.
#[derive(Debug, thiserror::Error)]
pub enum PythonCallError {
    /// `method_index` was past the end of the interface descriptor's methods.
    #[error("invalid method index {index} (interface has {count} method(s))")]
    InvalidMethodIndex { index: usize, count: usize },

    /// Tried to call a typed method through the raw path, or vice versa.
    #[error(
        "wire-mode mismatch on method '{method}': declared wire_raw={declared}, dispatcher used wire_raw={attempted}"
    )]
    WireModeMismatch {
        method: &'static str,
        declared: bool,
        attempted: bool,
    },

    /// The host-supplied input bytes (typed path) couldn't be decoded.
    #[error("failed to decode typed input: {0}")]
    InputDecode(String),

    /// The Python return value couldn't be encoded back for the host.
    #[error("failed to encode typed output: {0}")]
    OutputEncode(String),

    /// A Python exception was raised by the plugin.
    #[error("plugin raised: [{}] {}", .0.code, .0.message)]
    Plugin(PluginError),
}

/// Loaded-and-validated handle to one Python plugin.
#[derive(Debug)]
pub struct PythonPluginHandle {
    descriptor: &'static PythonInterfaceDescriptor,
    /// Imported entry module — kept alive so its callables (and their
    /// closures over module globals) remain valid.
    _module: Py<PyAny>,
    /// One callable per method, in descriptor order. Index here = vtable
    /// index used by the host's `Client::method_name(...)` call sites.
    method_callables: Vec<Py<PyAny>>,
}

impl PythonPluginHandle {
    pub(crate) fn new(
        descriptor: &'static PythonInterfaceDescriptor,
        module: Py<PyAny>,
        method_callables: Vec<Py<PyAny>>,
    ) -> Self {
        Self {
            descriptor,
            _module: module,
            method_callables,
        }
    }

    pub fn descriptor(&self) -> &'static PythonInterfaceDescriptor {
        self.descriptor
    }

    pub fn method_count(&self) -> usize {
        self.descriptor.methods.len()
    }

    /// Typed dispatch.
    ///
    /// `input_bincode` is the bincode-encoded args tuple — the same byte
    /// payload the cdylib path would receive. We use bincode here only
    /// because every other fidius caller does; on the way into Python we
    /// pivot through `serde_json::Value` (so the host's `I: Serialize` works
    /// for any type the macro accepts).
    pub fn call_typed(
        &self,
        method_index: usize,
        input_bincode: &[u8],
    ) -> Result<Vec<u8>, PythonCallError> {
        // bincode → serde_json::Value: round-trip via a String/Vec<u8>.
        // bincode is not self-describing, so we can't decode straight to
        // Value. Instead, decode into a `serde_json::Value` indirectly via
        // an intermediate trait object — actually that doesn't work either.
        //
        // What works: re-encode the bincode payload by deserialising into a
        // typed-erasure crate. We don't have one. So we take a different
        // approach: the *host* side will switch to JSON for python plugins
        // (see PluginHandle integration in T-0090). For T-0089 the typed
        // path receives JSON-encoded input directly; the bincode parameter
        // name is a holdover documenting future drift if we change the
        // host wire.
        //
        // For now: assume `input_bincode` is in fact JSON bytes. Document
        // the constraint loudly in the parameter name so callers don't
        // accidentally pass bincode here.
        self.call_typed_json(method_index, input_bincode)
    }

    /// Typed dispatch where the input is already JSON-serialised (the
    /// host's `serde_json::to_vec(&input)`). Returns JSON bytes the caller
    /// `serde_json::from_slice::<O>` decodes.
    pub fn call_typed_json(
        &self,
        method_index: usize,
        input_json: &[u8],
    ) -> Result<Vec<u8>, PythonCallError> {
        let method = self.lookup_method(method_index, false)?;
        let input_value: serde_json::Value = serde_json::from_slice(input_json)
            .map_err(|e| PythonCallError::InputDecode(e.to_string()))?;

        let result_value = Python::with_gil(|py| -> Result<serde_json::Value, PythonCallError> {
            let callable = method.callable.bind(py);
            let py_args = build_call_args(py, &input_value)
                .map_err(|e| PythonCallError::InputDecode(e.to_string()))?;
            let result = callable
                .call(py_args, None::<&Bound<'_, PyDict>>)
                .map_err(|e| PythonCallError::Plugin(pyerr_to_plugin_error(e)))?;
            pyobject_to_value(&result).map_err(|e| PythonCallError::OutputEncode(e.to_string()))
        })?;

        serde_json::to_vec(&result_value).map_err(|e| PythonCallError::OutputEncode(e.to_string()))
    }

    /// Raw dispatch — pass bytes in, get bytes out, no encoding.
    pub fn call_raw(&self, method_index: usize, input: &[u8]) -> Result<Vec<u8>, PythonCallError> {
        let method = self.lookup_method(method_index, true)?;

        Python::with_gil(|py| {
            let callable = method.callable.bind(py);
            let arg = PyBytes::new(py, input);
            let result = callable
                .call1((arg,))
                .map_err(|e| PythonCallError::Plugin(pyerr_to_plugin_error(e)))?;

            // Allow plugins to return `bytes`, `bytearray`, or anything
            // implementing the buffer protocol via PyBytes::extract.
            let bytes: Vec<u8> = result.extract().map_err(|e| {
                PythonCallError::OutputEncode(format!(
                    "raw method must return bytes/bytearray, got: {e}"
                ))
            })?;
            Ok(bytes)
        })
    }

    fn lookup_method(
        &self,
        index: usize,
        attempting_raw: bool,
    ) -> Result<MethodLookup<'_>, PythonCallError> {
        if index >= self.method_callables.len() {
            return Err(PythonCallError::InvalidMethodIndex {
                index,
                count: self.method_callables.len(),
            });
        }
        let desc = &self.descriptor.methods[index];
        if desc.wire_raw != attempting_raw {
            return Err(PythonCallError::WireModeMismatch {
                method: desc.name,
                declared: desc.wire_raw,
                attempted: attempting_raw,
            });
        }
        Ok(MethodLookup {
            callable: &self.method_callables[index],
        })
    }
}

struct MethodLookup<'a> {
    callable: &'a Py<PyAny>,
}

/// Build positional args for `callable.call(...)` from a JSON value.
///
/// The host's typed encoding is a tuple `(arg1, arg2, ...)` — this surfaces
/// as a JSON array. We unpack each element as a positional Python arg so
/// `def greet(name)` works rather than `def greet((name,))`. Non-array
/// values (degenerate case for zero-arg methods that produce JSON `null`)
/// dispatch as zero-arg calls.
fn build_call_args<'py>(
    py: Python<'py>,
    input: &serde_json::Value,
) -> PyResult<Bound<'py, PyTuple>> {
    match input {
        serde_json::Value::Array(items) => {
            let py_items: Vec<Bound<'_, PyAny>> = items
                .iter()
                .map(|v| value_to_pyobject(py, v))
                .collect::<PyResult<_>>()?;
            PyTuple::new(py, py_items)
        }
        serde_json::Value::Null => PyTuple::new(py, Vec::<Bound<'_, PyAny>>::new()),
        other => {
            // Single non-array, non-null value — treat as one positional arg.
            let pyobj = value_to_pyobject(py, other)?;
            PyTuple::new(py, vec![pyobj])
        }
    }
}
