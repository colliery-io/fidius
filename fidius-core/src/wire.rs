//! Wire format serialization for Fidius plugin FFI boundary.
//!
//! In debug builds (`cfg(debug_assertions)`), data is serialized as JSON for
//! human readability. In release builds, bincode is used for compact, fast
//! serialization. The `WIRE_FORMAT` constant encodes which format is active
//! so the host can reject mismatched plugins at load time.

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::descriptor::WireFormat;

/// The wire format active in this build.
#[cfg(debug_assertions)]
pub const WIRE_FORMAT: WireFormat = WireFormat::Json;

/// The wire format active in this build.
#[cfg(not(debug_assertions))]
pub const WIRE_FORMAT: WireFormat = WireFormat::Bincode;

/// Errors that can occur during wire serialization or deserialization.
#[derive(Debug, thiserror::Error)]
pub enum WireError {
    /// JSON serialization/deserialization error.
    #[error("json wire error: {0}")]
    Json(#[from] serde_json::Error),

    /// Bincode serialization/deserialization error.
    #[error("bincode wire error: {0}")]
    Bincode(#[from] bincode::Error),
}

/// Serialize a value using the active wire format.
///
/// Returns JSON bytes in debug builds, bincode bytes in release builds.
#[cfg(debug_assertions)]
pub fn serialize<T: Serialize>(val: &T) -> Result<Vec<u8>, WireError> {
    serde_json::to_vec(val).map_err(WireError::Json)
}

/// Deserialize a value from the active wire format.
///
/// Expects JSON bytes in debug builds, bincode bytes in release builds.
#[cfg(debug_assertions)]
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WireError> {
    serde_json::from_slice(bytes).map_err(WireError::Json)
}

/// Serialize a value using the active wire format.
///
/// Returns JSON bytes in debug builds, bincode bytes in release builds.
#[cfg(not(debug_assertions))]
pub fn serialize<T: Serialize>(val: &T) -> Result<Vec<u8>, WireError> {
    bincode::serialize(val).map_err(WireError::Bincode)
}

/// Deserialize a value from the active wire format.
///
/// Expects JSON bytes in debug builds, bincode bytes in release builds.
#[cfg(not(debug_assertions))]
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WireError> {
    bincode::deserialize(bytes).map_err(WireError::Bincode)
}
