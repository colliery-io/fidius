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

//! `Pyo3Executor` — the Python execution backend, behind the `python` feature.
//!
//! Thin host-side wrapper over `fidius_python::PythonPluginHandle` (which owns
//! the embedded-interpreter dispatch) that adapts it to the
//! [`crate::executor`] traits so Python plugins flow through the same
//! [`crate::handle::PluginHandle`] as cdylib plugins.
//!
//! Typed calls cross as a self-describing [`Value`]. The Python layer already
//! speaks self-describing JSON (`call_typed_json`), so the adapter bridges
//! `Value ↔ JSON` with `serde_json` — `Value` serialises to exactly the JSON
//! the Python `value_bridge` expects, so this is behaviour-identical to the
//! pre-unification path (`serde_json::to_vec(input) → call_typed_json`), just
//! routed through the neutral `Value` currency.

use fidius_core::Value;
use fidius_python::PythonPluginHandle;

use crate::error::CallError;
use crate::executor::{PluginExecutor, ValueExecutor};
use crate::types::PluginInfo;

/// Python-backed executor: an embedded-interpreter plugin handle plus the
/// host-facing [`PluginInfo`] (built from the package manifest + interface
/// descriptor at load time).
pub struct Pyo3Executor {
    py: PythonPluginHandle,
    info: PluginInfo,
}

impl Pyo3Executor {
    /// Wrap a loaded `PythonPluginHandle` with its owned metadata.
    pub fn new(py: PythonPluginHandle, info: PluginInfo) -> Self {
        Self { py, info }
    }
}

impl PluginExecutor for Pyo3Executor {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn method_count(&self) -> u32 {
        self.py.method_count() as u32
    }

    fn call_raw(&self, method: usize, input: &[u8]) -> Result<Vec<u8>, CallError> {
        // #[wire(raw)] methods: bytes straight through, no encoding.
        self.py.call_raw(method, input).map_err(CallError::from)
    }
}

impl ValueExecutor for Pyo3Executor {
    fn call(&self, method: usize, args: Value) -> Result<Value, CallError> {
        // Value -> JSON -> (Python) -> JSON -> Value. `Value`'s Serialize emits
        // the same JSON shape `call_typed_json` already consumed, so results
        // match the pre-unification typed path exactly.
        let json =
            serde_json::to_vec(&args).map_err(|e| CallError::Serialization(e.to_string()))?;
        let out = self
            .py
            .call_typed_json(method, &json)
            .map_err(CallError::from)?;
        serde_json::from_slice(&out).map_err(|e| CallError::Deserialization(e.to_string()))
    }
}
