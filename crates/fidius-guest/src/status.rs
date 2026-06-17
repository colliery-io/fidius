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

//! FFI status codes returned by plugin method shims.
//!
//! These `i32` values are the return type of every `extern "C"` function
//! in a plugin vtable. The host checks the status code before reading
//! the output buffer.

/// Method executed successfully. Output buffer contains the serialized result.
pub const STATUS_OK: i32 = 0;

/// Output buffer was too small (CallerAllocated/Arena strategies only).
/// The `out_len` parameter contains the required size. Retry with a larger buffer.
pub const STATUS_BUFFER_TOO_SMALL: i32 = -1;

/// Serialization or deserialization failed at the FFI boundary.
/// This indicates a bug in the generated shims or a type mismatch.
pub const STATUS_SERIALIZATION_ERROR: i32 = -2;

/// The plugin method returned an error. The output buffer contains a
/// serialized `PluginError` with details.
pub const STATUS_PLUGIN_ERROR: i32 = -3;

/// A panic was caught at the `extern "C"` boundary via `catch_unwind`.
/// The output buffer may contain a panic message string, but this is not guaranteed.
pub const STATUS_PANIC: i32 = -4;
