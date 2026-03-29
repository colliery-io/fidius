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

//! Fidius — a Rust plugin framework for trait-to-dylib plugin systems.
//!
//! This is the facade crate. Interface crates should depend on `fidius` only.
//! It re-exports everything needed to define interfaces and implement plugins.
//!
//! # For interface crate authors
//!
//! ```ignore
//! pub use fidius::{plugin_impl, PluginError};
//!
//! #[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
//! pub trait MyPlugin: Send + Sync {
//!     fn process(&self, input: String) -> String;
//! }
//! ```
//!
//! # For plugin crate authors
//!
//! ```ignore
//! use my_interface::{plugin_impl, MyPlugin, PluginError};
//!
//! pub struct MyImpl;
//!
//! #[plugin_impl(MyPlugin)]
//! impl MyPlugin for MyImpl {
//!     fn process(&self, input: String) -> String {
//!         format!("processed: {input}")
//!     }
//! }
//!
//! fidius::fidius_plugin_registry!();
//! ```

// Re-export macros
pub use fidius_macro::{plugin_impl, plugin_interface};

// Re-export modules so generated code can use fidius::descriptor::, fidius::status::, etc.
pub use fidius_core::descriptor;
pub use fidius_core::error;
pub use fidius_core::hash;
pub use fidius_core::status;
pub use fidius_core::wire;

// Also re-export key types at the crate root for convenience
pub use fidius_core::descriptor::{
    BufferStrategyKind, PluginDescriptor, PluginRegistry, WireFormat, ABI_VERSION, FIDIUS_MAGIC,
    REGISTRY_VERSION,
};
pub use fidius_core::error::PluginError;
pub use fidius_core::hash::{fnv1a, interface_hash};

#[cfg(feature = "async")]
pub use fidius_core::async_runtime;

// Re-export inventory so fidius_plugin_registry!() works via fidius_core
pub use fidius_core::inventory;

// Re-export the registry module for fidius_plugin_registry!() macro
pub use fidius_core::registry;

// Re-export the fidius_plugin_registry!() macro.
// Because it uses $crate:: paths, it resolves to fidius_core:: when called
// as fidius_core::fidius_plugin_registry!(), which works because fidius
// re-exports the necessary modules. Plugin authors call it as:
//   fidius::fidius_plugin_registry!();
pub use fidius_core::fidius_plugin_registry;
