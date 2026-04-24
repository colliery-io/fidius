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
//!
//! # What fidius does *not* provide: timeouts and cancellation
//!
//! Fidius has **no built-in timeout, deadline, or cancellation mechanism**
//! for plugin method calls. A call to `PluginHandle::call_method` (or
//! `call_method_raw`) runs to completion, panics, or — in the case of a
//! truly stuck plugin — never returns. There is no `PluginError::Timeout`
//! variant and the framework will not interrupt a misbehaving plugin.
//!
//! This is a deliberate consequence of the cdylib + in-process architecture:
//! plugin code runs synchronously on the host's calling thread, and Rust
//! cannot safely interrupt a thread mid-FFI-call. Any honest timeout
//! implementation requires running the plugin in a separate, killable
//! process — out of scope for the current framework.
//!
//! **If your threat model includes runaway plugins, you must add a
//! watchdog yourself.** The usual pattern is to run the host process
//! itself with a supervisor that can SIGKILL it on deadline; per-call
//! timeouts inside a single host process are not safely achievable for
//! arbitrary plugin code.
//!
//! Future work: the `fidius-python` initiative is the natural carrier for
//! a real subprocess-isolated execution tier, which would be the only path
//! by which fidius could grow first-class timeout semantics.

// Re-export macros.
//
// # Wire mode
//
// By default every method argument and return value is bincode-encoded
// across the FFI boundary. For methods whose argument and return are
// already byte buffers (image data, ML tensors, pre-encoded Arrow IPC,
// protobuf payloads, etc.) bincode is wasted overhead — it just adds
// length-prefixing and an extra alloc+memcpy on each side.
//
// Annotate such methods with `#[wire(raw)]` on both the trait declaration
// (in the interface crate) and the impl method (in the plugin crate). The
// signature must be exactly:
//
// ```ignore
// #[wire(raw)]
// fn process(&self, data: Vec<u8>) -> Vec<u8>;
// // or
// #[wire(raw)]
// fn process(&self, data: Vec<u8>) -> Result<Vec<u8>, MyError>;
// ```
//
// Errors and panic messages still go through bincode (small typed payloads).
// Host/plugin disagreement on wire mode for a given method surfaces as an
// interface-hash mismatch at load time — never as silent data corruption.
//
// Mix freely: one trait can have raw methods alongside normal typed methods.
pub use fidius_macro::{plugin_impl, plugin_interface};

// Re-export modules so generated code can use fidius::descriptor::, fidius::status::, etc.
pub use fidius_core::descriptor;
pub use fidius_core::error;
pub use fidius_core::hash;
pub use fidius_core::python_descriptor;
pub use fidius_core::status;
pub use fidius_core::wire;

// Also re-export key types at the crate root for convenience
pub use fidius_core::descriptor::{
    BufferStrategyKind, PluginDescriptor, PluginRegistry, ABI_VERSION, FIDIUS_MAGIC,
    REGISTRY_VERSION,
};
pub use fidius_core::error::PluginError;
pub use fidius_core::hash::{fnv1a, interface_hash};

#[cfg(feature = "async")]
pub use fidius_core::async_runtime;

// Host-side API surface — only present when the `host` feature is enabled.
// Plugin crates (cdylibs) do not enable this feature and therefore do not
// pull libloading or other host-only dependencies.
#[cfg(feature = "host")]
pub use fidius_host::{CallError, LoadError, LoadPolicy, PluginHandle, PluginHost, PluginInfo};

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
