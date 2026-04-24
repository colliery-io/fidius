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

//! Foundation smoke test for fidius-python: prove the embedded interpreter
//! starts up and that the error helper produces a sensible PluginError for a
//! Python exception. Subsequent tasks layer the loader and dispatcher on top.

use std::ffi::CString;

use fidius_python::{ensure_initialized, pyerr_to_plugin_error};
use pyo3::prelude::*;

#[test]
fn interpreter_evaluates_simple_expression() {
    ensure_initialized();
    let result = Python::with_gil(|py| -> i64 {
        let code = CString::new("1 + 1").unwrap();
        py.eval(code.as_c_str(), None, None)
            .unwrap()
            .extract()
            .unwrap()
    });
    assert_eq!(result, 2);
}

#[test]
fn pyerr_to_plugin_error_preserves_class_message_and_traceback() {
    ensure_initialized();
    let err = Python::with_gil(|py| -> PyErr {
        let code = CString::new("raise KeyError('nope')").unwrap();
        py.run(code.as_c_str(), None, None).unwrap_err()
    });

    let pe = pyerr_to_plugin_error(err);
    assert_eq!(pe.code, "KeyError");
    assert!(pe.message.contains("nope"));
    let details = pe.details.expect("traceback should be present");
    assert!(details.contains("traceback"));
}
