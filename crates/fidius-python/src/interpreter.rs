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

//! Embedded interpreter lifecycle.
//!
//! Fidius-python links libpython through PyO3's `auto-initialize` feature.
//! The interpreter is brought up lazily on first use and lives for the
//! remainder of the host process — fidius does not currently support
//! tearing it down or reinitialising it. This matches cloacina's pattern.
//!
//! All Python work is gated by PyO3's `Python::with_gil`, which is the
//! correct concurrency primitive for embedded interpreters: there is no
//! separate `Mutex<PyInterpreter>` to manage.

use std::sync::Once;

use tracing::trace;

static INIT: Once = Once::new();

/// Idempotent: ensure the embedded Python interpreter is initialised.
///
/// `auto-initialize` already brings it up on the first `Python::with_gil`
/// call, so this function is mostly a documented entry point and a tracing
/// hook for diagnosing "did Python actually start up" issues. Calling it
/// repeatedly is cheap.
pub fn ensure_initialized() {
    INIT.call_once(|| {
        trace!("initialising embedded Python interpreter");
        // Touching `Python::with_gil` triggers PyO3's auto-initialize.
        pyo3::Python::with_gil(|_py| {
            trace!("embedded Python interpreter is ready");
        });
    });
}
