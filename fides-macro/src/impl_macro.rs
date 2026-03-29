//! Code generation for `#[plugin_impl(TraitName)]`.
//!
//! Generates: the original impl, extern "C" FFI shims, a static instance,
//! a populated vtable static, a PluginDescriptor static, and for single-plugin
//! dylibs, the FIDES_PLUGIN_REGISTRY.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parse, parse::ParseStream, Ident, ImplItem, ItemImpl, Path};

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

    // Collect method names from the impl block
    let impl_methods: Vec<&Ident> = item
        .items
        .iter()
        .filter_map(|item| {
            if let ImplItem::Fn(method) = item {
                Some(&method.sig.ident)
            } else {
                None
            }
        })
        .collect();

    // Generate shim functions
    let shims = generate_shims(trait_name, &impl_ident, &impl_methods);

    // Generate static instance
    let instance_name = format_ident!("__FIDES_INSTANCE_{}", impl_ident);
    let instance = quote! {
        static #instance_name: #impl_type = #impl_type;
    };

    // Generate vtable static
    let vtable = generate_vtable_static(trait_name, &impl_ident, &impl_methods);

    // Generate free_buffer function
    let free_fn_name = format_ident!("__fides_free_buffer_{}", impl_ident);
    let free_buffer = quote! {
        unsafe extern "C" fn #free_fn_name(ptr: *mut u8, len: usize) {
            if !ptr.is_null() && len > 0 {
                drop(unsafe { Vec::from_raw_parts(ptr, len, len) });
            }
        }
    };

    // Generate descriptor
    let descriptor = generate_descriptor(trait_name, &impl_ident, &impl_methods);

    // Generate single-plugin registry (T-0009 will change this for multi-plugin)
    let registry = generate_single_registry(&impl_ident);

    Ok(quote! {
        #item
        #instance
        #shims
        #free_buffer
        #vtable
        #descriptor
        #registry
    })
}

/// Generate extern "C" shim functions for each method.
fn generate_shims(trait_name: &Ident, impl_ident: &Ident, methods: &[&Ident]) -> TokenStream {
    let instance_name = format_ident!("__FIDES_INSTANCE_{}", impl_ident);

    let shim_fns: Vec<TokenStream> = methods
        .iter()
        .map(|method_name| {
            let shim_name = format_ident!("__fides_shim_{}_{}", impl_ident, method_name);

            quote! {
                unsafe extern "C" fn #shim_name(
                    in_ptr: *const u8,
                    in_len: u32,
                    out_ptr: *mut *mut u8,
                    out_len: *mut u32,
                ) -> i32 {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let in_slice = unsafe { std::slice::from_raw_parts(in_ptr, in_len as usize) };
                        let args = match fides_core::wire::deserialize(in_slice) {
                            Ok(v) => v,
                            Err(_) => return fides_core::status::STATUS_SERIALIZATION_ERROR,
                        };

                        let output = #instance_name.#method_name(args);

                        let output_bytes = match fides_core::wire::serialize(&output) {
                            Ok(v) => v,
                            Err(_) => return fides_core::status::STATUS_SERIALIZATION_ERROR,
                        };

                        let len = output_bytes.len();
                        let ptr = output_bytes.as_ptr() as *mut u8;
                        std::mem::forget(output_bytes);
                        unsafe {
                            *out_ptr = ptr;
                            *out_len = len as u32;
                        }
                        fides_core::status::STATUS_OK
                    }));

                    match result {
                        Ok(status) => status,
                        Err(_) => fides_core::status::STATUS_PANIC,
                    }
                }
            }
        })
        .collect();

    quote! { #(#shim_fns)* }
}

/// Generate the static vtable with function pointers.
fn generate_vtable_static(
    trait_name: &Ident,
    impl_ident: &Ident,
    methods: &[&Ident],
) -> TokenStream {
    let vtable_type = format_ident!("{}_VTable", trait_name);
    let vtable_name = format_ident!("__FIDES_VTABLE_{}", impl_ident);

    let field_inits: Vec<TokenStream> = methods
        .iter()
        .map(|method_name| {
            let shim_name = format_ident!("__fides_shim_{}_{}", impl_ident, method_name);
            // For now, all methods are assigned directly.
            // Optional methods that are implemented get Some(fn_ptr).
            // TODO: Detect which methods are optional vs required from the interface IR.
            // For MVP, assume all impl'd methods map to required vtable fields.
            quote! { #method_name: #shim_name }
        })
        .collect();

    quote! {
        static #vtable_name: #vtable_type = #vtable_type {
            #(#field_inits,)*
        };
    }
}

/// Generate the PluginDescriptor static.
fn generate_descriptor(
    trait_name: &Ident,
    impl_ident: &Ident,
    methods: &[&Ident],
) -> TokenStream {
    let vtable_name = format_ident!("__FIDES_VTABLE_{}", impl_ident);
    let descriptor_name = format_ident!("__FIDES_DESCRIPTOR_{}", impl_ident);
    let free_fn_name = format_ident!("__fides_free_buffer_{}", impl_ident);
    let builder_fn = format_ident!(
        "__fides_build_{}_descriptor",
        trait_name.to_string().to_lowercase()
    );
    let plugin_name_const = format_ident!("__FIDES_PLUGIN_NAME_{}", impl_ident);
    let impl_name_str = impl_ident.to_string();

    // TODO: compute capabilities from which optional methods are present
    let capabilities = 0u64;

    quote! {
        const #plugin_name_const: &std::ffi::CStr = unsafe {
            std::ffi::CStr::from_bytes_with_nul_unchecked(concat!(#impl_name_str, "\0").as_bytes())
        };

        static #descriptor_name: fides_core::descriptor::PluginDescriptor = unsafe {
            #builder_fn(
                #plugin_name_const.as_ptr(),
                &#vtable_name as *const _ as *const _,
                #capabilities,
                Some(#free_fn_name),
            )
        };
    }
}

/// Generate a single-plugin FIDES_PLUGIN_REGISTRY.
fn generate_single_registry(impl_ident: &Ident) -> TokenStream {
    let descriptor_name = format_ident!("__FIDES_DESCRIPTOR_{}", impl_ident);
    let desc_ptr_name = format_ident!("__FIDES_DESC_PTR_{}", impl_ident);

    quote! {
        static #desc_ptr_name: fides_core::descriptor::DescriptorPtr =
            fides_core::descriptor::DescriptorPtr(&#descriptor_name as *const fides_core::descriptor::PluginDescriptor);

        #[no_mangle]
        pub static FIDES_PLUGIN_REGISTRY: fides_core::descriptor::PluginRegistry =
            fides_core::descriptor::PluginRegistry {
                magic: fides_core::descriptor::FIDES_MAGIC,
                registry_version: fides_core::descriptor::REGISTRY_VERSION,
                plugin_count: 1,
                descriptors: &#desc_ptr_name.0 as *const *const fides_core::descriptor::PluginDescriptor,
            };
    }
}
