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

//! Wire format serialization for Fidius plugin FFI boundary.
//!
//! Fidius uses bincode as the single wire format for all FFI data. Prior to
//! 0.1.0 the format varied by build profile (JSON in debug, bincode in
//! release) — that was removed because profile-mixed host/plugin load
//! rejections caused repeated dev-loop friction with no real inspection
//! benefit to offset them.

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Errors that can occur during wire serialization or deserialization.
#[derive(Debug, thiserror::Error)]
pub enum WireError {
    /// Bincode serialization/deserialization error.
    #[error("bincode wire error: {0}")]
    Bincode(#[from] bincode::Error),
}

/// Serialize a value as bincode for transport across the FFI boundary.
pub fn serialize<T: Serialize>(val: &T) -> Result<Vec<u8>, WireError> {
    bincode::serialize(val).map_err(WireError::Bincode)
}

/// Deserialize a value from bincode bytes received across the FFI boundary.
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WireError> {
    bincode::deserialize(bytes).map_err(WireError::Bincode)
}
