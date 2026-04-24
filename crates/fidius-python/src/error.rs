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

//! Bridge Python exceptions into fidius's `PluginError`.
//!
//! Every Python exception raised by plugin code crosses this helper on its
//! way back to the host. The mapping rules are:
//!
//! - `code` ← the exception class name (e.g. `"ValueError"`, `"KeyError"`).
//! - `message` ← `str(exc)` — the user-facing message Python produced.
//! - `details` ← a JSON-encoded object containing the formatted traceback
//!   (and, in later tasks, any structured fields a `fidius.PluginError`
//!   raise carried).
//!
//! This file deliberately stays minimal: later tasks (FIDIUS-T-0086,
//! FIDIUS-T-0089) extend it with `fidius.PluginError`-aware unwrapping so
//! plugin code can raise typed errors without their fields being flattened.

use fidius_core::PluginError;
use pyo3::prelude::*;
use pyo3::types::PyTraceback;
use serde_json::json;

/// Convert a `PyErr` into a `PluginError`, preserving class name, message,
/// and a formatted traceback in `details`.
///
/// Acquires the GIL internally, so callers can pass a `PyErr` they captured
/// outside `Python::with_gil` (the typical case for `?` propagation).
pub fn pyerr_to_plugin_error(err: PyErr) -> PluginError {
    Python::with_gil(|py| {
        let value = err.value(py);

        // Class name → code. `__class__.__name__` in Python.
        let code = value
            .getattr("__class__")
            .and_then(|cls| cls.getattr("__name__"))
            .and_then(|name| name.extract::<String>())
            .unwrap_or_else(|_| "UNKNOWN_PYTHON_ERROR".to_string());

        let message = value
            .str()
            .and_then(|s| s.extract::<String>())
            .unwrap_or_else(|_| "<unprintable Python exception>".to_string());

        let traceback = err
            .traceback(py)
            .and_then(|tb| format_traceback(py, tb).ok())
            .unwrap_or_default();

        let details = json!({ "traceback": traceback }).to_string();

        PluginError {
            code,
            message,
            details: Some(details),
        }
    })
}

/// Format a Python traceback into a plain string by calling
/// `traceback.format_tb(tb)` and joining the result. Best-effort: returns an
/// empty string on internal failure rather than recursively raising.
fn format_traceback(py: Python<'_>, tb: Bound<'_, PyTraceback>) -> PyResult<String> {
    let traceback_mod = py.import("traceback")?;
    let frames = traceback_mod.call_method1("format_tb", (tb,))?;
    let parts: Vec<String> = frames.extract()?;
    Ok(parts.join(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_value_error_to_plugin_error() {
        crate::ensure_initialized();
        let err = Python::with_gil(|py| -> PyErr {
            py.eval(
                std::ffi::CString::new("(_ for _ in ()).throw(ValueError('boom'))")
                    .unwrap()
                    .as_c_str(),
                None,
                None,
            )
            .unwrap_err()
        });

        let pe = pyerr_to_plugin_error(err);
        assert_eq!(pe.code, "ValueError");
        assert!(pe.message.contains("boom"));
        let details = pe.details.expect("details should be set");
        assert!(details.contains("traceback"));
    }
}
