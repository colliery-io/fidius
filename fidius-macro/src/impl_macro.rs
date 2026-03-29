//! Code generation for `#[plugin_impl(TraitName)]`.
//!
//! Generates: the original impl, extern "C" FFI shims, a static instance,
//! a populated vtable static, a PluginDescriptor static, and for single-plugin
//! dylibs, the FIDIUS_PLUGIN_REGISTRY.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parse, parse::ParseStream, Ident, ImplItem, ItemImpl};

/// Info about an impl method, extracted from the impl block.
struct MethodInfo<'a> {
    name: &'a Ident,
    is_async: bool,
}

/// Arguments to `#[plugin_impl(TraitName)]`.
pub struct PluginImplAttrs {
    pub trait_name: Ident,
}

impl Parse for PluginImplAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let trait_name: Ident = input.parse()?;
        Ok(PluginImplAttrs { trait_name })
    }
}

/// Generate all code for a `#[plugin_impl(TraitName)]` invocation.
pub fn generate_plugin_impl(attrs: &PluginImplAttrs, item: &ItemImpl) -> syn::Result<TokenStream> {
    let trait_name = &attrs.trait_name;
    let impl_type = &item.self_ty;

    // Extract the type name as a string for naming
    let impl_type_str = quote!(#impl_type).to_string().replace(' ', "");
    let impl_ident = format_ident!("{}", impl_type_str);

    // Collect method info from the impl block
    let impl_methods: Vec<MethodInfo> = item
        .items
        .iter()
        .filter_map(|item| {
            if let ImplItem::Fn(method) = item {
                Some(MethodInfo {
                    name: &method.sig.ident,
                    is_async: method.sig.asyncness.is_some(),
                })
            } else {
                None
            }
        })
        .collect();

    let method_names: Vec<&Ident> = impl_methods.iter().map(|m| m.name).collect();
    let has_async = impl_methods.iter().any(|m| m.is_async);

    // Generate shim functions
    let shims = generate_shims(&impl_ident, &impl_methods);

    // Generate static instance
    let instance_name = format_ident!("__FIDIUS_INSTANCE_{}", impl_ident);
    let instance = quote! {
        static #instance_name: #impl_type = #impl_type;
    };

    // Generate vtable static
    let vtable = generate_vtable_static(trait_name, &impl_ident, &method_names);

    // Generate free_buffer function
    let free_fn_name = format_ident!("__fidius_free_buffer_{}", impl_ident);
    let free_buffer = quote! {
        unsafe extern "C" fn #free_fn_name(ptr: *mut u8, len: usize) {
            if !ptr.is_null() && len > 0 {
                drop(unsafe { Vec::from_raw_parts(ptr, len, len) });
            }
        }
    };

    // Generate descriptor
    let descriptor = generate_descriptor(trait_name, &impl_ident, &method_names);

    // Register descriptor via inventory for multi-plugin collection
    let registration = generate_inventory_registration(&impl_ident);

    Ok(quote! {
        #item
        #instance
        #shims
        #free_buffer
        #vtable
        #descriptor
        #registration
    })
}

/// Generate extern "C" shim functions for each method.
fn generate_shims(impl_ident: &Ident, methods: &[MethodInfo]) -> TokenStream {
    let instance_name = format_ident!("__FIDIUS_INSTANCE_{}", impl_ident);

    let shim_fns: Vec<TokenStream> = methods
        .iter()
        .map(|method| {
            let method_name = method.name;
            let shim_name = format_ident!("__fidius_shim_{}_{}", impl_ident, method_name);

            // The method call — either sync or async via block_on
            let method_call = if method.is_async {
                quote! {
                    fidius_core::async_runtime::FIDIUS_RUNTIME.block_on(
                        #instance_name.#method_name(args)
                    )
                }
            } else {
                quote! { #instance_name.#method_name(args) }
            };

            quote! {
                unsafe extern "C" fn #shim_name(
                    in_ptr: *const u8,
                    in_len: u32,
                    out_ptr: *mut *mut u8,
                    out_len: *mut u32,
                ) -> i32 {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let in_slice = unsafe { std::slice::from_raw_parts(in_ptr, in_len as usize) };
                        let args = match fidius_core::wire::deserialize(in_slice) {
                            Ok(v) => v,
                            Err(_) => return fidius_core::status::STATUS_SERIALIZATION_ERROR,
                        };

                        let output = #method_call;

                        let output_bytes = match fidius_core::wire::serialize(&output) {
                            Ok(v) => v,
                            Err(_) => return fidius_core::status::STATUS_SERIALIZATION_ERROR,
                        };

                        let len = output_bytes.len();
                        let ptr = output_bytes.as_ptr() as *mut u8;
                        std::mem::forget(output_bytes);
                        unsafe {
                            *out_ptr = ptr;
                            *out_len = len as u32;
                        }
                        fidius_core::status::STATUS_OK
                    }));

                    match result {
                        Ok(status) => status,
                        Err(_) => fidius_core::status::STATUS_PANIC,
                    }
                }
            }
        })
        .collect();

    quote! { #(#shim_fns)* }
}

/// Generate the static vtable with function pointers.
///
/// Uses the `new_{trait}` constructor generated by `#[plugin_interface]`,
/// which knows which fields are optional (Option<fn>) vs required (fn).
fn generate_vtable_static(
    trait_name: &Ident,
    impl_ident: &Ident,
    methods: &[&Ident],
) -> TokenStream {
    let vtable_type = format_ident!("{}_VTable", trait_name);
    let vtable_name = format_ident!("__FIDIUS_VTABLE_{}", impl_ident);
    let constructor = format_ident!("new_{}_vtable", trait_name.to_string().to_lowercase());

    let shim_args: Vec<TokenStream> = methods
        .iter()
        .map(|method_name| {
            let shim_name = format_ident!("__fidius_shim_{}_{}", impl_ident, method_name);
            quote! { #shim_name }
        })
        .collect();

    quote! {
        static #vtable_name: #vtable_type = #constructor(#(#shim_args),*);
    }
}

/// Generate the PluginDescriptor static.
fn generate_descriptor(
    trait_name: &Ident,
    impl_ident: &Ident,
    methods: &[&Ident],
) -> TokenStream {
    let vtable_name = format_ident!("__FIDIUS_VTABLE_{}", impl_ident);
    let descriptor_name = format_ident!("__FIDIUS_DESCRIPTOR_{}", impl_ident);
    let free_fn_name = format_ident!("__fidius_free_buffer_{}", impl_ident);
    let builder_fn = format_ident!(
        "__fidius_build_{}_descriptor",
        trait_name.to_string().to_lowercase()
    );
    let plugin_name_const = format_ident!("__FIDIUS_PLUGIN_NAME_{}", impl_ident);
    let impl_name_str = impl_ident.to_string();

    // TODO: compute capabilities from which optional methods are present
    let capabilities = 0u64;

    quote! {
        const #plugin_name_const: &std::ffi::CStr = unsafe {
            std::ffi::CStr::from_bytes_with_nul_unchecked(concat!(#impl_name_str, "\0").as_bytes())
        };

        static #descriptor_name: fidius_core::descriptor::PluginDescriptor = unsafe {
            #builder_fn(
                #plugin_name_const.as_ptr(),
                &#vtable_name as *const _ as *const _,
                #capabilities,
                Some(#free_fn_name),
            )
        };
    }
}

/// Register the descriptor via inventory for multi-plugin support.
fn generate_inventory_registration(impl_ident: &Ident) -> TokenStream {
    let descriptor_name = format_ident!("__FIDIUS_DESCRIPTOR_{}", impl_ident);

    quote! {
        fidius_core::inventory::submit! {
            fidius_core::registry::DescriptorEntry {
                descriptor: &#descriptor_name,
            }
        }
    }
}

