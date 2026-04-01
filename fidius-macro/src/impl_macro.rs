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

//! Code generation for `#[plugin_impl(TraitName)]`.
//!
//! Generates: the original impl, extern "C" FFI shims, a static instance,
//! a populated vtable static, a PluginDescriptor static, and for single-plugin
//! dylibs, the FIDIUS_PLUGIN_REGISTRY.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::Parse, parse::ParseStream, FnArg, Ident, ImplItem, ItemImpl, LitStr, Pat, Path,
    ReturnType, Token, Type,
};

/// Info about an impl method, extracted from the impl block.
struct MethodInfo<'a> {
    name: &'a Ident,
    is_async: bool,
    returns_result: bool,
    /// Argument types (excluding `self`).
    arg_types: Vec<&'a Type>,
    /// Argument names (excluding `self`).
    arg_names: Vec<Ident>,
}

/// Check if a return type looks like `Result<T, ...>`.
fn is_result_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident == "Result")
            .unwrap_or(false)
    } else {
        false
    }
}

/// Arguments to `#[plugin_impl(TraitName)]` or `#[plugin_impl(TraitName, crate = "...")]`.
pub struct PluginImplAttrs {
    pub trait_name: Ident,
    /// The path to the fidius crate. Defaults to `fidius` when not specified.
    pub crate_path: Path,
}

impl Parse for PluginImplAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let trait_name: Ident = input.parse()?;
        let mut crate_path = None;

        while !input.is_empty() {
            let _comma: Token![,] = input.parse()?;
            if input.peek(Token![crate]) {
                let _kw: Token![crate] = input.parse()?;
                let _eq: Token![=] = input.parse()?;
                let lit: LitStr = input.parse()?;
                let path: Path = lit.parse()?;
                crate_path = Some(path);
            }
        }

        let crate_path = crate_path.unwrap_or_else(|| syn::parse_str::<Path>("fidius").unwrap());

        Ok(PluginImplAttrs {
            trait_name,
            crate_path,
        })
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
                let returns_result = match &method.sig.output {
                    ReturnType::Type(_, ty) => is_result_type(ty),
                    ReturnType::Default => false,
                };
                let mut arg_types = Vec::new();
                let mut arg_names = Vec::new();
                for arg in &method.sig.inputs {
                    if let FnArg::Typed(pat_type) = arg {
                        arg_types.push(pat_type.ty.as_ref());
                        if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                            arg_names.push(pat_ident.ident.clone());
                        } else {
                            arg_names.push(format_ident!("_arg"));
                        }
                    }
                }
                Some(MethodInfo {
                    name: &method.sig.ident,
                    is_async: method.sig.asyncness.is_some(),
                    returns_result,
                    arg_types,
                    arg_names,
                })
            } else {
                None
            }
        })
        .collect();

    let method_names: Vec<&Ident> = impl_methods.iter().map(|m| m.name).collect();
    let _has_async = impl_methods.iter().any(|m| m.is_async);

    let crate_path = &attrs.crate_path;

    // Generate shim functions
    let shims = generate_shims(&impl_ident, &impl_methods, crate_path);

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
    let descriptor = generate_descriptor(trait_name, &impl_ident, &method_names, crate_path);

    // Register descriptor via inventory for multi-plugin collection
    let registration = generate_inventory_registration(&impl_ident, crate_path);

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
fn generate_shims(impl_ident: &Ident, methods: &[MethodInfo], crate_path: &Path) -> TokenStream {
    let instance_name = format_ident!("__FIDIUS_INSTANCE_{}", impl_ident);

    let shim_fns: Vec<TokenStream> = methods
        .iter()
        .map(|method| {
            let method_name = method.name;
            let shim_name = format_ident!("__fidius_shim_{}_{}", impl_ident, method_name);

            let arg_types = &method.arg_types;
            let arg_names = &method.arg_names;

            // Deserialize input as a tuple of all argument types
            let deserialize_args = quote! {
                let (#(#arg_names,)*) = match #crate_path::wire::deserialize::<(#(#arg_types,)*)>(in_slice) {
                    Ok(v) => v,
                    Err(_) => return #crate_path::status::STATUS_SERIALIZATION_ERROR,
                };
            };

            // The method call — either sync or async via block_on
            let method_call = if method.is_async {
                quote! {
                    #crate_path::async_runtime::FIDIUS_RUNTIME.block_on(
                        #instance_name.#method_name(#(#arg_names),*)
                    )
                }
            } else {
                quote! { #instance_name.#method_name(#(#arg_names),*) }
            };

            // Generate the output handling based on whether the method returns Result
            let output_handling = if method.returns_result {
                quote! {
                    match output {
                        Ok(val) => {
                            match #crate_path::wire::serialize(&val) {
                                Ok(v) => (v, #crate_path::status::STATUS_OK),
                                Err(_) => return #crate_path::status::STATUS_SERIALIZATION_ERROR,
                            }
                        }
                        Err(err) => {
                            match #crate_path::wire::serialize(&err) {
                                Ok(v) => (v, #crate_path::status::STATUS_PLUGIN_ERROR),
                                Err(_) => return #crate_path::status::STATUS_SERIALIZATION_ERROR,
                            }
                        }
                    }
                }
            } else {
                quote! {
                    match #crate_path::wire::serialize(&output) {
                        Ok(v) => (v, #crate_path::status::STATUS_OK),
                        Err(_) => return #crate_path::status::STATUS_SERIALIZATION_ERROR,
                    }
                }
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
                        #deserialize_args

                        let output = #method_call;

                        let (mut output_bytes, status) = #output_handling;

                        // Ensure capacity == len so free_buffer can safely
                        // reconstruct the Vec with capacity == len.
                        output_bytes.shrink_to_fit();
                        let len = output_bytes.len();
                        let ptr = output_bytes.as_ptr() as *mut u8;
                        std::mem::forget(output_bytes);
                        unsafe {
                            *out_ptr = ptr;
                            *out_len = len as u32;
                        }
                        status
                    }));

                    match result {
                        Ok(status) => status,
                        Err(panic_payload) => {
                            // Extract panic message and serialize into output buffer
                            let msg = panic_payload
                                .downcast_ref::<&str>()
                                .map(|s| s.to_string())
                                .or_else(|| panic_payload.downcast_ref::<String>().cloned())
                                .unwrap_or_else(|| "unknown panic".to_string());

                            if let Ok(msg_bytes) = #crate_path::wire::serialize(&msg) {
                                let mut msg_bytes = msg_bytes;
                                msg_bytes.shrink_to_fit();
                                let len = msg_bytes.len();
                                let ptr = msg_bytes.as_ptr() as *mut u8;
                                std::mem::forget(msg_bytes);
                                unsafe {
                                    *out_ptr = ptr;
                                    *out_len = len as u32;
                                }
                            }
                            #crate_path::status::STATUS_PANIC
                        }
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
    let companion = format_ident!("__fidius_{}", trait_name);
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
        static #vtable_name: #companion::#vtable_type = #companion::#constructor(#(#shim_args),*);
    }
}

/// Generate the PluginDescriptor static.
fn generate_descriptor(
    trait_name: &Ident,
    impl_ident: &Ident,
    methods: &[&Ident],
    crate_path: &Path,
) -> TokenStream {
    let companion = format_ident!("__fidius_{}", trait_name);
    let vtable_name = format_ident!("__FIDIUS_VTABLE_{}", impl_ident);
    let descriptor_name = format_ident!("__FIDIUS_DESCRIPTOR_{}", impl_ident);
    let free_fn_name = format_ident!("__fidius_free_buffer_{}", impl_ident);
    let builder_fn = format_ident!(
        "__fidius_build_{}_descriptor",
        trait_name.to_string().to_lowercase()
    );
    let plugin_name_const = format_ident!("__FIDIUS_PLUGIN_NAME_{}", impl_ident);
    let impl_name_str = impl_ident.to_string();

    let optional_methods_ident = format_ident!("{}_OPTIONAL_METHODS", trait_name);
    let method_strs: Vec<String> = methods.iter().map(|m| m.to_string()).collect();
    let method_count = methods.len() as u32;

    quote! {
        const #plugin_name_const: &std::ffi::CStr = unsafe {
            std::ffi::CStr::from_bytes_with_nul_unchecked(concat!(#impl_name_str, "\0").as_bytes())
        };

        static #descriptor_name: #crate_path::descriptor::PluginDescriptor = unsafe {
            // Compute capabilities inline: check which impl'd methods
            // appear in the optional methods list.
            // Uses manual byte-by-byte comparison because stable Rust does not
            // support str::eq in const contexts.
            const CAPS: u64 = {
                let optional = #companion::#optional_methods_ident;
                let impl_methods: &[&str] = &[#(#method_strs),*];
                let mut caps: u64 = 0;
                let mut opt_idx = 0;
                while opt_idx < optional.len() {
                    let opt_name = optional[opt_idx];
                    let mut impl_idx = 0;
                    while impl_idx < impl_methods.len() {
                        let impl_name = impl_methods[impl_idx];
                        if opt_name.len() == impl_name.len() {
                            let ob = opt_name.as_bytes();
                            let ib = impl_name.as_bytes();
                            let mut j = 0;
                            let mut eq = true;
                            while j < ob.len() {
                                if ob[j] != ib[j] { eq = false; }
                                j += 1;
                            }
                            if eq {
                                caps |= 1u64 << opt_idx;
                            }
                        }
                        impl_idx += 1;
                    }
                    opt_idx += 1;
                }
                caps
            };

            #companion::#builder_fn(
                #plugin_name_const.as_ptr(),
                &#vtable_name as *const _ as *const _,
                CAPS,
                Some(#free_fn_name),
                #method_count,
            )
        };
    }
}

/// Register the descriptor via inventory for multi-plugin support.
fn generate_inventory_registration(impl_ident: &Ident, crate_path: &Path) -> TokenStream {
    let descriptor_name = format_ident!("__FIDIUS_DESCRIPTOR_{}", impl_ident);

    quote! {
        #crate_path::inventory::submit! {
            #crate_path::registry::DescriptorEntry {
                descriptor: &#descriptor_name,
            }
        }
    }
}
