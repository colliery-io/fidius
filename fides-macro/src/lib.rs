mod impl_macro;
mod interface;
mod ir;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemImpl, ItemTrait};

use impl_macro::PluginImplAttrs;
use ir::InterfaceAttrs;

/// Define a plugin interface from a trait.
///
/// Generates a `#[repr(C)]` vtable struct, interface hash constant,
/// capability bit constants, and a descriptor builder function.
///
/// # Example
///
/// ```ignore
/// #[plugin_interface(version = 1, buffer = PluginAllocated)]
/// pub trait Greeter: Send + Sync {
///     fn greet(&self, name: String) -> String;
///
///     #[optional(since = 2)]
///     fn greet_fancy(&self, name: String) -> String;
/// }
/// ```
#[proc_macro_attribute]
pub fn plugin_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as InterfaceAttrs);
    let item_trait = parse_macro_input!(item as ItemTrait);

    match ir::parse_interface(attrs, &item_trait) {
        Ok(ir) => match interface::generate_interface(&ir) {
            Ok(tokens) => tokens.into(),
            Err(err) => err.to_compile_error().into(),
        },
        Err(err) => err.to_compile_error().into(),
    }
}

/// Implement a plugin interface for a concrete type.
///
/// Generates extern "C" FFI shims, a static vtable, a plugin descriptor,
/// and a plugin registry.
///
/// # Example
///
/// ```ignore
/// pub struct MyGreeter;
///
/// #[plugin_impl(Greeter)]
/// impl Greeter for MyGreeter {
///     fn greet(&self, name: String) -> String {
///         format!("Hello, {name}!")
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn plugin_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as PluginImplAttrs);
    let item_impl = parse_macro_input!(item as ItemImpl);

    match impl_macro::generate_plugin_impl(&attrs, &item_impl) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
