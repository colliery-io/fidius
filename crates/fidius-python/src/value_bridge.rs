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

//! Bridge between `serde_json::Value` and Python objects.
//!
//! For typed plugin calls, the host serialises arguments through
//! `serde_json::to_value` (any `Serialize` type works) and we then convert
//! the JSON tree into Python primitives. The reverse is symmetric: take a
//! Python return value and turn it back into a `serde_json::Value` the host
//! can `serde_json::from_value::<O>` into the expected return type.
//!
//! JSON is the wire format because it's the common ground between
//! self-describing-serde-on-the-Rust-side and natural-Python-types-on-the-
//! Python-side. The hot path for bulk data uses `#[wire(raw)]` (T-0082)
//! which bypasses this layer entirely.

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyBytes, PyDict, PyFloat, PyInt, PyList, PyString, PyTuple};
use serde_json::{Map, Value};

/// Convert a `serde_json::Value` into a Python object owned by `py`.
pub fn value_to_pyobject<'py>(py: Python<'py>, value: &Value) -> PyResult<Bound<'py, PyAny>> {
    match value {
        Value::Null => Ok(py.None().into_bound(py)),
        Value::Bool(b) => Ok(PyBool::new(py, *b).to_owned().into_any()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.into_any())
            } else if let Some(u) = n.as_u64() {
                Ok(u.into_pyobject(py)?.into_any())
            } else {
                let f = n.as_f64().ok_or_else(|| {
                    pyo3::exceptions::PyValueError::new_err("non-finite number in JSON value")
                })?;
                Ok(f.into_pyobject(py)?.into_any())
            }
        }
        Value::String(s) => Ok(PyString::new(py, s).into_any()),
        Value::Array(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(value_to_pyobject(py, item)?)?;
            }
            Ok(list.into_any())
        }
        Value::Object(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, value_to_pyobject(py, v)?)?;
            }
            Ok(dict.into_any())
        }
    }
}

/// Convert a Python object back into a `serde_json::Value`.
///
/// `bytes` are encoded as a JSON object `{ "__bytes__": [...] }` so the
/// host can recover them via a custom serde adapter (today only used by
/// internal error paths — typed bytes go through `#[wire(raw)]` instead).
pub fn pyobject_to_value(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if obj.is_none() {
        return Ok(Value::Null);
    }
    if let Ok(b) = obj.downcast::<PyBool>() {
        return Ok(Value::Bool(b.is_true()));
    }
    if let Ok(s) = obj.downcast::<PyString>() {
        return Ok(Value::String(s.extract()?));
    }
    // PyBytes: encode as a JSON array of byte values; host-side adapters
    // can recover Vec<u8>. Plugins that want zero-copy bulk bytes should
    // use #[wire(raw)] instead.
    if let Ok(b) = obj.downcast::<PyBytes>() {
        let bytes: &[u8] = b.as_bytes();
        let arr: Vec<Value> = bytes
            .iter()
            .map(|x| Value::Number((*x as i64).into()))
            .collect();
        return Ok(Value::Array(arr));
    }
    if let Ok(_f) = obj.downcast::<PyFloat>() {
        let f: f64 = obj.extract()?;
        let n = serde_json::Number::from_f64(f).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err("non-finite float cannot be encoded to JSON")
        })?;
        return Ok(Value::Number(n));
    }
    if let Ok(_i) = obj.downcast::<PyInt>() {
        // Try i64, then u64, then fall back to f64.
        if let Ok(i) = obj.extract::<i64>() {
            return Ok(Value::Number(i.into()));
        }
        if let Ok(u) = obj.extract::<u64>() {
            return Ok(Value::Number(u.into()));
        }
        let f: f64 = obj.extract()?;
        let n = serde_json::Number::from_f64(f)
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("integer too large for JSON"))?;
        return Ok(Value::Number(n));
    }
    if let Ok(list) = obj.downcast::<PyList>() {
        let mut out = Vec::with_capacity(list.len());
        for item in list.iter() {
            out.push(pyobject_to_value(&item)?);
        }
        return Ok(Value::Array(out));
    }
    if let Ok(tuple) = obj.downcast::<PyTuple>() {
        let mut out = Vec::with_capacity(tuple.len());
        for item in tuple.iter() {
            out.push(pyobject_to_value(&item)?);
        }
        return Ok(Value::Array(out));
    }
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = Map::new();
        for (k, v) in dict.iter() {
            let key: String = k
                .extract()
                .or_else(|_| -> PyResult<String> { k.str()?.extract() })?;
            map.insert(key, pyobject_to_value(&v)?);
        }
        return Ok(Value::Object(map));
    }

    // Fallback: try str(obj) so we don't lose the value entirely.
    let s: String = obj.str()?.extract()?;
    Ok(Value::String(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn roundtrip_primitives() {
        crate::ensure_initialized();
        Python::with_gil(|py| {
            for v in [
                json!(null),
                json!(true),
                json!(42i64),
                json!(3.5f64),
                json!("hello"),
                json!([1, "two", false]),
                json!({"a": 1, "b": [2, 3]}),
            ] {
                let py_obj = value_to_pyobject(py, &v).unwrap();
                let back = pyobject_to_value(&py_obj).unwrap();
                assert_eq!(back, v, "round-trip failed for {v:?}");
            }
        });
    }
}
