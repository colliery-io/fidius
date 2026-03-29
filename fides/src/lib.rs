//! Fides — a Rust plugin framework for trait-to-dylib plugin systems.
//!
//! This is the facade crate. Interface crates should depend on `fides` only.
//! It re-exports everything needed to define interfaces and implement plugins.
//!
//! # For interface crate authors
//!
//! ```ignore
//! pub use fides::{plugin_impl, PluginError};
//!
//! #[fides::plugin_interface(version = 1, buffer = PluginAllocated)]
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
//! fides_core::fides_plugin_registry!();
//! ```

// Re-export macros
pub use fides_macro::{plugin_impl, plugin_interface};

// Re-export core types that plugin/interface authors need
pub use fides_core::descriptor::{
    BufferStrategyKind, PluginDescriptor, PluginRegistry, WireFormat,
    ABI_VERSION, FIDES_MAGIC, REGISTRY_VERSION,
};
pub use fides_core::error::PluginError;
pub use fides_core::hash::{fnv1a, interface_hash};
pub use fides_core::status::*;
pub use fides_core::wire;

// Re-export inventory so fides_plugin_registry!() works via fides_core
pub use fides_core::inventory;

// Re-export the registry module for fides_plugin_registry!() macro
pub use fides_core::registry;

// The fides_plugin_registry!() macro is #[macro_export] in fides_core,
// so it's available as fides_core::fides_plugin_registry!() automatically.
