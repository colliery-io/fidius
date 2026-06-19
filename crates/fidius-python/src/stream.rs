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

//! Server-streaming dispatch for Python plugins (FIDIUS-I-0026).
//!
//! A streaming plugin method returns a Python **generator** (or any iterable).
//! [`super::handle::PythonPluginHandle::call_streaming_start`] calls the method
//! to obtain its iterator and wraps it in a [`PythonStream`].
//!
//! [`PythonStream`] is the *sync, GIL-aware* half of the bridge: it advances the
//! iterator one item at a time ([`PythonStream::next`], holding the GIL only for
//! that step) and runs the generator's cleanup on cancellation
//! ([`PythonStream::cancel`] → `gen.close()` → `GeneratorExit`/`finally`). The
//! *async* half — pumping it onto a `ChunkStream` over a bounded channel with
//! backpressure — lives in `fidius-host`'s `Pyo3Executor`, which owns the tokio
//! machinery. Keeping PyO3 here and tokio there respects the existing crate
//! split (`fidius-python` has no async runtime).

use pyo3::prelude::*;
use pyo3::types::PyAnyMethods;

use fidius_core::PluginError;

use crate::error::pyerr_to_plugin_error;
use crate::value_bridge::pyobject_to_value;

/// One step of advancing a Python plugin's server-streaming iterator.
pub enum PyStreamStep {
    /// One produced item, already converted to the JSON currency.
    Item(serde_json::Value),
    /// Clean end of stream (`StopIteration`).
    End,
    /// The generator raised, or an item failed to convert. Terminal.
    Error(PluginError),
}

/// A handle to an in-flight Python server-stream — the iterator obtained by
/// calling a streaming plugin method.
///
/// `Send` (a `Py<PyAny>` is `Send + Sync`) so the host can drive it from a
/// dedicated pump thread, acquiring the GIL only for each `next`/`cancel`.
pub struct PythonStream {
    iter: Py<PyAny>,
}

impl PythonStream {
    pub(crate) fn new(iter: Py<PyAny>) -> Self {
        Self { iter }
    }

    /// Advance one item. Holds the GIL only for the duration of this call, so a
    /// slow downstream consumer never pins the interpreter.
    pub fn next(&self) -> PyStreamStep {
        Python::with_gil(|py| {
            let it = self.iter.bind(py);
            match it.call_method0("__next__") {
                Ok(item) => match pyobject_to_value(&item) {
                    Ok(v) => PyStreamStep::Item(v),
                    Err(e) => PyStreamStep::Error(PluginError::new("OutputEncode", e.to_string())),
                },
                Err(e) if e.is_instance_of::<pyo3::exceptions::PyStopIteration>(py) => {
                    PyStreamStep::End
                }
                Err(e) => PyStreamStep::Error(pyerr_to_plugin_error(e)),
            }
        })
    }

    /// Cancel the stream: run the generator's cleanup by calling `close()`,
    /// which raises `GeneratorExit` inside the generator so its `finally`/
    /// context-manager `__exit__` runs (closing DB cursors, releasing handles).
    /// A no-op for iterables without `close` (e.g. a plain `list_iterator`).
    pub fn cancel(&self) {
        Python::with_gil(|py| {
            let it = self.iter.bind(py);
            if let Ok(close) = it.getattr("close") {
                let _ = close.call0();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ensure_initialized;

    /// Build a `PythonStream` from a snippet that evaluates to an iterator.
    fn stream_from(code: &str) -> PythonStream {
        ensure_initialized();
        Python::with_gil(|py| {
            let obj = py
                .eval(&std::ffi::CString::new(code).unwrap(), None, None)
                .expect("eval");
            let iter = obj.try_iter().expect("iterable");
            PythonStream::new(iter.into_any().unbind())
        })
    }

    fn item_i64(step: PyStreamStep) -> i64 {
        match step {
            PyStreamStep::Item(v) => v.as_i64().expect("int item"),
            other => panic!("expected item, got {}", step_name(&other)),
        }
    }

    fn step_name(s: &PyStreamStep) -> &'static str {
        match s {
            PyStreamStep::Item(_) => "Item",
            PyStreamStep::End => "End",
            PyStreamStep::Error(_) => "Error",
        }
    }

    #[test]
    fn yields_items_then_end() {
        let s = stream_from("iter([1, 2, 3])");
        assert_eq!(item_i64(s.next()), 1);
        assert_eq!(item_i64(s.next()), 2);
        assert_eq!(item_i64(s.next()), 3);
        assert!(matches!(s.next(), PyStreamStep::End));
        // Past the end stays End (fused on the host side anyway).
        assert!(matches!(s.next(), PyStreamStep::End));
    }

    #[test]
    fn generator_exception_becomes_error() {
        // A generator that yields once, then raises.
        let s = gen_from_def("def g():\n    yield 7\n    raise ValueError('boom')\nit = g()");
        assert_eq!(item_i64(s.next()), 7);
        match s.next() {
            PyStreamStep::Error(pe) => {
                assert!(pe.message.contains("boom"), "message was: {}", pe.message)
            }
            other => panic!("expected error, got {}", step_name(&other)),
        }
        // After a terminal error the iterator is exhausted.
        assert!(matches!(s.next(), PyStreamStep::End));
    }

    /// Run a snippet that binds `it` to an iterator/generator in fresh globals.
    fn gen_from_def(code: &str) -> PythonStream {
        ensure_initialized();
        Python::with_gil(|py| {
            let globals = pyo3::types::PyDict::new(py);
            py.run(&std::ffi::CString::new(code).unwrap(), Some(&globals), None)
                .unwrap();
            let it = globals.get_item("it").unwrap().unwrap();
            PythonStream::new(it.unbind())
        })
    }

    #[test]
    fn cancel_runs_generator_finally() {
        ensure_initialized();
        let (stream, globals) = Python::with_gil(|py| {
            let globals = pyo3::types::PyDict::new(py);
            py.run(
                &std::ffi::CString::new(
                    "ran = {'cleanup': False}\n\
                     def g():\n    \
                         try:\n        \
                             yield 1\n        \
                             yield 2\n    \
                         finally:\n        \
                             ran['cleanup'] = True\n\
                     it = g()",
                )
                .unwrap(),
                Some(&globals),
                None,
            )
            .unwrap();
            let it = globals.get_item("it").unwrap().unwrap();
            (PythonStream::new(it.unbind()), globals.unbind())
        });

        // Pull one item, then cancel mid-stream.
        assert_eq!(item_i64(stream.next()), 1);
        stream.cancel();

        // The generator's `finally` must have run.
        Python::with_gil(|py| {
            let g = globals.bind(py);
            let ran = g.get_item("ran").unwrap().unwrap();
            let cleanup: bool = ran.get_item("cleanup").unwrap().extract().unwrap();
            assert!(cleanup, "generator `finally` should run on cancel()");
        });
    }
}
