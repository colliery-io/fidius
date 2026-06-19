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

/// Bounded channel depth between the GIL-holding pump thread and the host's
/// async consumer. Small on purpose: it *is* the backpressure window (REQ-003)
/// and the bound on buffered items (NFR-003) — the pump blocks once it's full,
/// so a paused consumer parks the generator rather than draining it into memory.
#[cfg(feature = "streaming")]
const STREAM_CHANNEL_CAP: usize = 4;

#[cfg(feature = "streaming")]
#[async_trait::async_trait]
impl crate::stream::StreamExecutor for Pyo3Executor {
    async fn call_streaming(
        &self,
        method: usize,
        args: Value,
    ) -> Result<crate::stream::ChunkStream, CallError> {
        // Same Value -> JSON arg encoding as the unary typed path.
        let json =
            serde_json::to_vec(&args).map_err(|e| CallError::Serialization(e.to_string()))?;
        // Start the generator (GIL held briefly here, synchronously).
        let stream = self
            .py
            .call_streaming_start(method, &json)
            .map_err(CallError::from)?;

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<Value, CallError>>(STREAM_CHANNEL_CAP);

        // Dedicated pump thread (not a tokio worker — it uses blocking_send and
        // holds the GIL per item). Native `Value` items, no framing: the Value
        // rail produces `Value`s directly (bincode can't reconstruct one).
        std::thread::spawn(move || {
            use fidius_python::PyStreamStep;
            loop {
                match stream.next() {
                    // Clean end: drop `tx` → the host stream ends (None).
                    PyStreamStep::End => break,
                    // Producer error: surface one Err, then end.
                    PyStreamStep::Error(pe) => {
                        let _ = tx.blocking_send(Err(CallError::Plugin(pe)));
                        break;
                    }
                    PyStreamStep::Item(jv) => {
                        // JSON is self-describing, so `Value` reconstructs fine
                        // here (unlike bincode).
                        let item = match serde_json::from_value::<Value>(jv) {
                            Ok(v) => Ok(v),
                            Err(e) => Err(CallError::Deserialization(e.to_string())),
                        };
                        let is_err = item.is_err();
                        // blocking_send parks the GIL-free thread when the
                        // channel is full → backpressure. `Err` means the
                        // consumer dropped the stream → cancel the generator.
                        if tx.blocking_send(item).is_err() {
                            stream.cancel();
                            break;
                        }
                        if is_err {
                            break;
                        }
                    }
                }
            }
        });

        // Adapt the receiver into the host-facing item stream.
        let body = futures::stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|item| (item, rx))
        });
        Ok(crate::stream::ChunkStream::new(body))
    }
}
