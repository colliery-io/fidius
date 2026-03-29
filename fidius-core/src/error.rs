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

//! Error types for the Fidius plugin framework.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error returned by plugin method implementations to signal business logic failures.
///
/// Serialized across the FFI boundary via the wire format. The host deserializes
/// this from the output buffer when the FFI shim returns `STATUS_PLUGIN_ERROR`.
///
/// The `details` field is stored as a JSON string (not `serde_json::Value`)
/// so that it serializes correctly under both JSON and bincode wire formats.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginError {
    /// Machine-readable error code (e.g., `"INVALID_INPUT"`, `"NOT_FOUND"`).
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Optional structured details as a JSON string.
    pub details: Option<String>,
}

impl PluginError {
    /// Create a new `PluginError` without details.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Create a new `PluginError` with structured details.
    ///
    /// The `serde_json::Value` is serialized to a JSON string for storage.
    pub fn with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: Some(details.to_string()),
        }
    }

    /// Parse the `details` field back into a `serde_json::Value`.
    ///
    /// Returns `None` if details is absent or fails to parse.
    pub fn details_value(&self) -> Option<serde_json::Value> {
        self.details
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
    }
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for PluginError {}
