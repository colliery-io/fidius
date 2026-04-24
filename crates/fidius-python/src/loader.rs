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

//! Load a Python plugin package and produce a `PythonPluginHandle` whose
//! method-index map lets the host dispatch by index just like the cdylib
//! path.
//!
//! Loading steps:
//!
//! 1. Read the package's `package.toml` and assert `runtime = "python"`.
//! 2. Prepend `<dir>/vendor` and `<dir>` to `sys.path` (idempotent — repeated
//!    loads of the same package don't insert twice).
//! 3. Import the entry module declared in `[python].entry_module`.
//! 4. Validate the module's `__interface_hash__` constant against the
//!    descriptor passed by the host. Mismatch = clean load error.
//! 5. Look up each method by name (in the descriptor's order) so vtable
//!    indices resolve to Python callables at call time.
//!
//! What we don't do here: subprocess spawning, venv creation, or cancellation.
//! All Python work happens in the host's embedded interpreter (T-0085).

use std::path::{Path, PathBuf};

use fidius_core::package::{load_manifest_untyped, PackageRuntime};
use fidius_core::python_descriptor::PythonInterfaceDescriptor;
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyList};
use tracing::{debug, info};

use crate::error::pyerr_to_plugin_error;
use crate::handle::PythonPluginHandle;
use crate::interpreter::ensure_initialized;

/// Errors that can happen during Python plugin load.
#[derive(Debug, thiserror::Error)]
pub enum PythonLoadError {
    #[error("manifest error: {0}")]
    Manifest(#[from] fidius_core::package::PackageError),

    #[error(
        "package at {path} is not a Python plugin (manifest runtime is \"{got}\", not \"python\")"
    )]
    NotPythonRuntime { path: String, got: String },

    #[error("package at {path} is missing the [python] section")]
    MissingPythonSection { path: String },

    #[error("entry module '{module}' import failed: {message}")]
    ImportFailed { module: String, message: String },

    #[error(
        "interface hash mismatch for trait '{interface}': package declares {got:#018x}, host expects {expected:#018x}"
    )]
    InterfaceHashMismatch {
        interface: &'static str,
        got: u64,
        expected: u64,
    },

    #[error("entry module '{module}' is missing required attribute '{attr}'")]
    MissingAttr { module: String, attr: &'static str },

    #[error(
        "method '{method}' on trait '{interface}' is not registered in the entry module: {message}"
    )]
    MethodNotRegistered {
        interface: &'static str,
        method: &'static str,
        message: String,
    },
}

/// Load a Python plugin package against a static interface descriptor.
///
/// `package_dir` must point at an unpacked Python plugin directory (the
/// thing `unpack_package` returns or that lives next to a `package.toml`
/// during local dev).
pub fn load_python_plugin(
    package_dir: &Path,
    descriptor: &'static PythonInterfaceDescriptor,
) -> Result<PythonPluginHandle, PythonLoadError> {
    ensure_initialized();

    let manifest = load_manifest_untyped(package_dir)?;
    if !matches!(manifest.package.runtime(), PackageRuntime::Python) {
        return Err(PythonLoadError::NotPythonRuntime {
            path: package_dir.display().to_string(),
            got: manifest
                .package
                .runtime
                .clone()
                .unwrap_or_else(|| "rust".to_string()),
        });
    }
    let py_meta =
        manifest
            .python
            .as_ref()
            .ok_or_else(|| PythonLoadError::MissingPythonSection {
                path: package_dir.display().to_string(),
            })?;

    info!(
        package = %package_dir.display(),
        interface = descriptor.interface_name,
        entry_module = py_meta.entry_module,
        "loading python plugin"
    );

    Python::with_gil(|py| {
        prepend_sys_path(py, package_dir)?;
        let module = py.import(py_meta.entry_module.as_str()).map_err(|e| {
            PythonLoadError::ImportFailed {
                module: py_meta.entry_module.clone(),
                message: e.to_string(),
            }
        })?;

        validate_interface_hash(&module, descriptor)?;
        let method_callables = resolve_methods(&module, descriptor)?;

        Ok(PythonPluginHandle::new(
            descriptor,
            module.unbind().into(),
            method_callables,
        ))
    })
}

/// Prepend `<dir>/vendor` and `<dir>` to `sys.path` if not already present.
/// Both are pushed at index 0 so they shadow anything else with the same
/// module name — important for the vendored-deps story.
fn prepend_sys_path(py: Python<'_>, dir: &Path) -> Result<(), PythonLoadError> {
    let sys = py.import("sys").map_err(|e| import_failure("sys", e))?;
    let path_attr = sys
        .getattr("path")
        .map_err(|e| import_failure("sys.path", e))?;
    let path: Bound<'_, PyList> = path_attr
        .downcast::<PyList>()
        .map_err(|e| PythonLoadError::ImportFailed {
            module: "sys".into(),
            message: format!("sys.path is not a list: {e}"),
        })?
        .clone();

    let candidates: Vec<PathBuf> = vec![dir.join("vendor"), dir.to_path_buf()];

    for candidate in candidates.into_iter().rev() {
        let s = candidate.to_string_lossy().into_owned();
        let already_present = path.iter().any(|item| {
            item.extract::<String>()
                .map(|existing| existing == s)
                .unwrap_or(false)
        });
        if !already_present {
            debug!(path = %s, "prepending to sys.path");
            path.insert(0, &s)
                .map_err(|e| import_failure("sys.path.insert", e))?;
        }
    }
    Ok(())
}

fn validate_interface_hash(
    module: &Bound<'_, PyModule>,
    descriptor: &'static PythonInterfaceDescriptor,
) -> Result<(), PythonLoadError> {
    let attr = module
        .getattr("__interface_hash__")
        .map_err(|_| PythonLoadError::MissingAttr {
            module: module.name().map(|n| n.to_string()).unwrap_or_default(),
            attr: "__interface_hash__",
        })?;
    let got: u64 = attr.extract().map_err(|e| PythonLoadError::ImportFailed {
        module: module.name().map(|n| n.to_string()).unwrap_or_default(),
        message: format!("__interface_hash__ is not a u64: {e}"),
    })?;
    if got != descriptor.interface_hash {
        return Err(PythonLoadError::InterfaceHashMismatch {
            interface: descriptor.interface_name,
            got,
            expected: descriptor.interface_hash,
        });
    }
    Ok(())
}

fn resolve_methods(
    module: &Bound<'_, PyModule>,
    descriptor: &'static PythonInterfaceDescriptor,
) -> Result<Vec<Py<PyAny>>, PythonLoadError> {
    // Resolve callables by direct attribute lookup on the loaded module.
    // The fidius SDK's @method decorator returns the function unchanged
    // (it's a registration-only marker), so module-attribute lookup is the
    // canonical way to find a callable. Skipping the SDK registry here also
    // avoids the cross-module ambiguity that arises when many plugins
    // share the embedded interpreter.
    let module_name = module
        .name()
        .map(|n| n.to_string())
        .unwrap_or_else(|_| descriptor.interface_name.to_string());

    let mut callables = Vec::with_capacity(descriptor.methods.len());
    for method in descriptor.methods {
        let callable = module
            .getattr(method.name)
            .map_err(|e| PythonLoadError::MethodNotRegistered {
                interface: descriptor.interface_name,
                method: method.name,
                message: format!("module '{module_name}': {e}"),
            })?
            .unbind();
        callables.push(callable);
    }
    Ok(callables)
}

fn import_failure(what: &str, err: PyErr) -> PythonLoadError {
    let pe = pyerr_to_plugin_error(err);
    PythonLoadError::ImportFailed {
        module: what.to_string(),
        message: pe.message,
    }
}
