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

//! Code generation for `#[plugin_interface]`.
//!
//! Generates: the original trait, a `#[repr(C)]` vtable struct, interface hash constant,
//! capability bit constants, version/strategy constants, and a descriptor builder function.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemTrait, TraitItem};

use crate::ir::{BufferStrategyAttr, InterfaceIR};

/// Strip `#[optional(...)]` attributes from trait methods so the emitted trait compiles.
fn strip_optional_attrs(item: &ItemTrait) -> ItemTrait {
    let mut cleaned = item.clone();
    for trait_item in &mut cleaned.items {
        if let TraitItem::Fn(method) = trait_item {
            method
                .attrs
                .retain(|attr| !attr.path().is_ident("optional"));
        }
    }
    cleaned
}

/// Generate all code for a `#[plugin_interface]` invocation.
pub fn generate_interface(ir: &InterfaceIR) -> syn::Result<TokenStream> {
    match ir.attrs.buffer_strategy {
        BufferStrategyAttr::CallerAllocated => {
            return Err(syn::Error::new_spanned(
                &ir.original_trait.ident,
                "CallerAllocated buffer strategy is not yet supported",
            ));
        }
        BufferStrategyAttr::Arena => {
            return Err(syn::Error::new_spanned(
                &ir.original_trait.ident,
                "Arena buffer strategy is not yet supported",
            ));
        }
        BufferStrategyAttr::PluginAllocated => {}
    }

    let cleaned_trait = strip_optional_attrs(&ir.original_trait);
    let vtable = generate_vtable(ir);
    let constants = generate_constants(ir);
    let descriptor_builder = generate_descriptor_builder(ir);
    let method_indices = generate_method_indices(ir);
    let companion_mod = format_ident!("__fidius_{}", ir.trait_name);

    Ok(quote! {
        #cleaned_trait
        /// Generated companion module for the plugin interface.
        ///
        /// Contains the VTable struct, interface hash, capability constants,
        /// vtable constructor, descriptor builder, method index constants,
        /// and a typed `Client` struct for host-side calling.
        /// Method indices follow trait declaration order (0-based).
        #[allow(non_snake_case, non_upper_case_globals, dead_code)]
        pub mod #companion_mod {
            use super::*;
            #vtable
            #constants
            #method_indices
            #descriptor_builder
        }
    })
}

/// Generate the `#[repr(C)]` vtable struct.
fn generate_vtable(ir: &InterfaceIR) -> TokenStream {
    let vtable_name = format_ident!("{}_VTable", ir.trait_name);

    let fields: Vec<TokenStream> = ir
        .methods
        .iter()
        .map(|m| {
            let field_name = &m.name;
            // PluginAllocated signature: (in_ptr, in_len, out_ptr_ptr, out_len) -> i32
            let fn_type = quote! {
                unsafe extern "C" fn(
                    *const u8, u32,
                    *mut *mut u8, *mut u32,
                ) -> i32
            };

            if m.optional_since.is_some() {
                quote! { pub #field_name: Option<#fn_type> }
            } else {
                quote! { pub #field_name: #fn_type }
            }
        })
        .collect();

    // Constructor function that takes bare fn pointers and wraps optional ones in Some()
    let constructor_name = format_ident!("new_{}_vtable", ir.trait_name.to_string().to_lowercase());

    let fn_type = quote! {
        unsafe extern "C" fn(*const u8, u32, *mut *mut u8, *mut u32) -> i32
    };

    let params: Vec<TokenStream> = ir
        .methods
        .iter()
        .map(|m| {
            let name = &m.name;
            quote! { #name: #fn_type }
        })
        .collect();

    let field_assigns: Vec<TokenStream> = ir
        .methods
        .iter()
        .map(|m| {
            let name = &m.name;
            if m.optional_since.is_some() {
                quote! { #name: Some(#name) }
            } else {
                quote! { #name: #name }
            }
        })
        .collect();

    quote! {
        #[repr(C)]
        pub struct #vtable_name {
            #(#fields,)*
        }

        pub const fn #constructor_name(#(#params),*) -> #vtable_name {
            #vtable_name {
                #(#field_assigns,)*
            }
        }
    }
}

/// Generate interface hash, capability bit constants, version, and buffer strategy constants.
fn generate_constants(ir: &InterfaceIR) -> TokenStream {
    let trait_name = &ir.trait_name;

    // Interface hash: computed from sorted required method signature strings
    let required_sigs: Vec<&str> = ir
        .methods
        .iter()
        .filter(|m| m.is_required())
        .map(|m| m.signature_string.as_str())
        .collect();

    let hash_value = fidius_core::hash::interface_hash(&required_sigs);

    let hash_name = format_ident!("{}_INTERFACE_HASH", trait_name);
    let version_name = format_ident!("{}_INTERFACE_VERSION", trait_name);
    let strategy_name = format_ident!("{}_BUFFER_STRATEGY", trait_name);

    let version_val = ir.attrs.version;
    let strategy_val = ir.attrs.buffer_strategy as u8;

    // Capability bit constants for optional methods
    let cap_constants: Vec<TokenStream> = ir
        .methods
        .iter()
        .filter(|m| m.optional_since.is_some())
        .enumerate()
        .map(|(bit, m)| {
            let const_name =
                format_ident!("{}_CAP_{}", trait_name, m.name.to_string().to_uppercase());
            let bit_val = 1u64 << bit;
            quote! { pub const #const_name: u64 = #bit_val; }
        })
        .collect();

    let optional_names_ident = format_ident!("{}_OPTIONAL_METHODS", trait_name);
    let optional_names: Vec<String> = ir
        .methods
        .iter()
        .filter(|m| m.optional_since.is_some())
        .map(|m| m.name.to_string())
        .collect();

    quote! {
        pub const #hash_name: u64 = #hash_value;
        pub const #version_name: u32 = #version_val;
        pub const #strategy_name: u8 = #strategy_val;
        #(#cap_constants)*
        pub const #optional_names_ident: &[&str] = &[#(#optional_names),*];
    }
}

/// Generate the descriptor builder function used by `#[plugin_impl]`.
fn generate_descriptor_builder(ir: &InterfaceIR) -> TokenStream {
    let trait_name = &ir.trait_name;
    let vtable_name = format_ident!("{}_VTable", trait_name);
    let fn_name = format_ident!(
        "__fidius_build_{}_descriptor",
        trait_name.to_string().to_lowercase()
    );
    let hash_name = format_ident!("{}_INTERFACE_HASH", trait_name);
    let version_name = format_ident!("{}_INTERFACE_VERSION", trait_name);
    let strategy_name = format_ident!("{}_BUFFER_STRATEGY", trait_name);
    let interface_name_str = trait_name.to_string();
    let interface_name_cstr_ident = format_ident!("__FIDIUS_INTERFACE_NAME_{}", trait_name);

    quote! {
        /// Null-terminated interface name for the descriptor.
        const #interface_name_cstr_ident: &std::ffi::CStr = {
            // Use c"..." literal syntax (stable since Rust 1.77)
            // We can't use c"..." with a variable, so we use unsafe from_bytes_with_nul_unchecked
            // on a concat! with \0
            unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(concat!(#interface_name_str, "\0").as_bytes()) }
        };

        /// Build a `PluginDescriptor` for this interface.
        ///
        /// # Safety
        ///
        /// `plugin_name` must be a static, null-terminated C string.
        /// `vtable` must point to a valid, static `#vtable_name`.
        /// `free_buffer` must be `Some` (PluginAllocated strategy).
        pub const unsafe fn #fn_name(
            plugin_name: *const std::ffi::c_char,
            vtable: *const #vtable_name,
            capabilities: u64,
            free_buffer: Option<unsafe extern "C" fn(*mut u8, usize)>,
            method_count: u32,
        ) -> fidius::descriptor::PluginDescriptor {
            fidius::descriptor::PluginDescriptor {
                abi_version: fidius::descriptor::ABI_VERSION,
                interface_name: #interface_name_cstr_ident.as_ptr(),
                interface_hash: #hash_name,
                interface_version: #version_name,
                capabilities,
                wire_format: fidius::wire::WIRE_FORMAT as u8,
                buffer_strategy: #strategy_name,
                plugin_name,
                vtable: vtable as *const std::ffi::c_void,
                free_buffer,
                method_count,
            }
        }
    }
}

/// Generate method index constants.
fn generate_method_indices(ir: &InterfaceIR) -> TokenStream {
    let indices: Vec<TokenStream> = ir
        .methods
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let const_name = format_ident!("METHOD_{}", m.name.to_string().to_uppercase());
            let doc = format!("Vtable index for `{}`.", m.name);
            quote! {
                #[doc = #doc]
                pub const #const_name: usize = #i;
            }
        })
        .collect();

    quote! { #(#indices)* }
}

// NOTE: Typed client generation deferred — requires fidius-host types
// which the interface crate doesn't depend on. Method index constants
// provide the key benefit: named indices instead of magic numbers.
#[allow(dead_code)]
fn _generate_client_deferred(ir: &InterfaceIR) -> TokenStream {
    let trait_name = &ir.trait_name;
    let client_name = format_ident!("{}Client", trait_name);

    let methods: Vec<TokenStream> = ir
        .methods
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let method_name = &m.name;
            let index = i;

            // Get arg types (excluding &self)
            let arg_types = &m.arg_types;
            let arg_names = &m.arg_names;

            // Get return type
            let ret_type = match &m.return_type {
                Some(ty) => quote! { #ty },
                None => quote! { () },
            };

            // For optional methods, check capability first
            let cap_check = if m.optional_since.is_some() {
                let cap_bit = ir.methods.iter()
                    .filter(|mm| mm.optional_since.is_some())
                    .position(|mm| mm.name == m.name)
                    .unwrap_or(0) as u32;
                quote! {
                    if !self.handle.has_capability(#cap_bit) {
                        return Err(fidius_host::CallError::NotImplemented { bit: #cap_bit });
                    }
                }
            } else {
                quote! {}
            };

            if arg_types.len() == 1 {
                // Single arg — pass directly
                let arg_type = &arg_types[0];
                let arg_name = &arg_names[0];
                quote! {
                    pub fn #method_name(&self, #arg_name: &#arg_type) -> Result<#ret_type, fidius_host::CallError> {
                        #cap_check
                        self.handle.call_method(#index, #arg_name)
                    }
                }
            } else if arg_types.is_empty() {
                // No args — pass unit
                quote! {
                    pub fn #method_name(&self) -> Result<#ret_type, fidius_host::CallError> {
                        #cap_check
                        self.handle.call_method(#index, &())
                    }
                }
            } else {
                // Multiple args — pass as tuple
                quote! {
                    pub fn #method_name(&self, #(#arg_names: &#arg_types),*) -> Result<#ret_type, fidius_host::CallError> {
                        #cap_check
                        self.handle.call_method(#index, &(#(#arg_names.clone()),*))
                    }
                }
            }
        })
        .collect();

    quote! {
        /// Typed client for calling plugin methods by name.
        ///
        /// Wraps a `PluginHandle` and provides named methods with correct types,
        /// eliminating raw index-based `call_method` usage.
        pub struct #client_name {
            handle: fidius_host::PluginHandle,
        }

        impl #client_name {
            /// Create a client from a loaded plugin handle.
            pub fn from_handle(handle: fidius_host::PluginHandle) -> Self {
                Self { handle }
            }

            /// Access the underlying handle for raw method calls or metadata.
            pub fn handle(&self) -> &fidius_host::PluginHandle {
                &self.handle
            }

            #(#methods)*
        }
    }
}
