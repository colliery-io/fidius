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

//! Per-dylib async runtime for plugin methods.
//!
//! When the `async` feature is enabled, this module provides a lazily-initialized
//! tokio runtime shared across all plugin implementations in the dylib.
//! The generated shims call `FIDIUS_RUNTIME.block_on(...)` for async methods.

/// The shared tokio runtime for this dylib.
///
/// Initialized on first use. One runtime per dylib, shared across all
/// plugin implementations.
pub static FIDIUS_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
    std::sync::LazyLock::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create fidius async runtime")
    });
