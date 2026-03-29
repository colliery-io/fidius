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
//! fidius_core::fidius_plugin_registry!();
//! ```

// Re-export macros
pub use fidius_macro::{plugin_impl, plugin_interface};

// Re-export core types that plugin/interface authors need
pub use fidius_core::descriptor::{
    BufferStrategyKind, PluginDescriptor, PluginRegistry, WireFormat,
    ABI_VERSION, FIDIUS_MAGIC, REGISTRY_VERSION,
};
pub use fidius_core::error::PluginError;
pub use fidius_core::hash::{fnv1a, interface_hash};
pub use fidius_core::status::*;
pub use fidius_core::wire;

// Re-export inventory so fidius_plugin_registry!() works via fidius_core
pub use fidius_core::inventory;

// Re-export the registry module for fidius_plugin_registry!() macro
pub use fidius_core::registry;

// The fidius_plugin_registry!() macro is #[macro_export] in fidius_core,
// so it's available as fidius_core::fidius_plugin_registry!() automatically.
