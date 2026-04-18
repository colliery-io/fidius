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

/// Strip fidius-specific helper attributes (`#[optional]`, `#[method_meta]`,
/// `#[trait_meta]`) from the trait and its methods so the emitted trait
/// compiles as a plain Rust trait definition.
fn strip_optional_attrs(item: &ItemTrait) -> ItemTrait {
    fn is_fidius_helper(attr: &syn::Attribute) -> bool {
        attr.path().is_ident("optional")
            || attr.path().is_ident("method_meta")
            || attr.path().is_ident("trait_meta")
    }

    let mut cleaned = item.clone();
    cleaned.attrs.retain(|attr| !is_fidius_helper(attr));
    for trait_item in &mut cleaned.items {
        if let TraitItem::Fn(method) = trait_item {
            method.attrs.retain(|attr| !is_fidius_helper(attr));
        }
    }
    cleaned
}

/// Generate all code for a `#[plugin_interface]` invocation.
pub fn generate_interface(ir: &InterfaceIR) -> syn::Result<TokenStream> {
    // Both PluginAllocated and Arena are supported; vtable and shim codegen
    // branches on ir.attrs.buffer_strategy.
    match ir.attrs.buffer_strategy {
        BufferStrategyAttr::PluginAllocated | BufferStrategyAttr::Arena => {}
    }

    let cleaned_trait = strip_optional_attrs(&ir.original_trait);
    let vtable = generate_vtable(ir);
    let constants = generate_constants(ir);
    let descriptor_builder = generate_descriptor_builder(ir);
    let method_indices = generate_method_indices(ir);
    let metadata = generate_metadata(ir);
    let client = generate_client(ir);
    let companion_mod = format_ident!("__fidius_{}", ir.trait_name);
    Ok(quote! {
        #cleaned_trait
        /// Generated companion module for the plugin interface.
        ///
        /// Contains the VTable struct, interface hash, capability constants,
        /// vtable constructor, descriptor builder, and method index constants.
        /// Method indices follow trait declaration order (0-based).
        #[allow(non_snake_case, non_upper_case_globals, dead_code)]
        pub mod #companion_mod {
            use super::*;
            #vtable
            #constants
            #method_indices
            #metadata
            #descriptor_builder
        }
        #client
    })
}

/// Emit the static metadata arrays for `#[method_meta]` and `#[trait_meta]`
/// attributes into the companion module. Emits:
///
/// - `__FIDIUS_METHOD_META_<NAME>: [MetaKv; N]` for each method with metadata
/// - `__FIDIUS_METHOD_META_TABLE: [MethodMetaEntry; method_count]` if any method has metadata
/// - `__FIDIUS_TRAIT_META: [MetaKv; N]` if any trait-level metadata exists
///
/// If no metadata is declared anywhere, emits nothing — the descriptor
/// builder falls back to null pointers.
fn generate_metadata(ir: &InterfaceIR) -> TokenStream {
    let crate_path = &ir.attrs.crate_path;
    let trait_name = &ir.trait_name;

    let mut per_method_arrays = Vec::new();
    let mut table_entries = Vec::new();
    let any_method_meta = ir.methods.iter().any(|m| !m.method_metas.is_empty());

    if any_method_meta {
        for method in &ir.methods {
            let arr_name = format_ident!(
                "__FIDIUS_METHOD_META_{}",
                method.name.to_string().to_uppercase()
            );
            if method.method_metas.is_empty() {
                table_entries.push(quote! {
                    #crate_path::descriptor::MethodMetaEntry {
                        kvs: ::std::ptr::null(),
                        kv_count: 0,
                    }
                });
            } else {
                let kv_count = method.method_metas.len() as u32;
                let kv_inits: Vec<TokenStream> = method
                    .method_metas
                    .iter()
                    .map(|kv| {
                        let k = &kv.key;
                        let v = &kv.value;
                        quote! {
                            #crate_path::descriptor::MetaKv {
                                key: concat!(#k, "\0").as_ptr() as *const ::std::ffi::c_char,
                                value: concat!(#v, "\0").as_ptr() as *const ::std::ffi::c_char,
                            }
                        }
                    })
                    .collect();
                let arr_len = method.method_metas.len();
                per_method_arrays.push(quote! {
                    static #arr_name: [#crate_path::descriptor::MetaKv; #arr_len] = [
                        #(#kv_inits),*
                    ];
                });
                table_entries.push(quote! {
                    #crate_path::descriptor::MethodMetaEntry {
                        kvs: #arr_name.as_ptr(),
                        kv_count: #kv_count,
                    }
                });
            }
        }
    }

    let method_count = ir.methods.len();
    let table = if any_method_meta {
        quote! {
            /// Per-method metadata table, one entry per method in declaration order.
            /// Methods with no `#[method_meta]` annotations have null `kvs`.
            pub static __FIDIUS_METHOD_META_TABLE: [#crate_path::descriptor::MethodMetaEntry; #method_count] = [
                #(#table_entries),*
            ];
        }
    } else {
        quote! {}
    };

    let trait_meta = if !ir.trait_metas.is_empty() {
        let kv_inits: Vec<TokenStream> = ir
            .trait_metas
            .iter()
            .map(|kv| {
                let k = &kv.key;
                let v = &kv.value;
                quote! {
                    #crate_path::descriptor::MetaKv {
                        key: concat!(#k, "\0").as_ptr() as *const ::std::ffi::c_char,
                        value: concat!(#v, "\0").as_ptr() as *const ::std::ffi::c_char,
                    }
                }
            })
            .collect();
        let len = ir.trait_metas.len();
        quote! {
            /// Trait-level metadata from `#[trait_meta(...)]` on the interface trait.
            pub static __FIDIUS_TRAIT_META: [#crate_path::descriptor::MetaKv; #len] = [
                #(#kv_inits),*
            ];
        }
    } else {
        quote! {}
    };

    let _ = trait_name;
    quote! {
        #(#per_method_arrays)*
        #table
        #trait_meta
    }
}

/// Generate the `#[repr(C)]` vtable struct.
fn generate_vtable(ir: &InterfaceIR) -> TokenStream {
    let vtable_name = format_ident!("{}_VTable", ir.trait_name);

    // Vtable fn signature varies by buffer strategy:
    // - PluginAllocated: (in_ptr, in_len, out_ptr_ptr, out_len) -> i32
    //   (plugin allocates output; host frees via descriptor.free_buffer)
    // - Arena: (in_ptr, in_len, arena_ptr, arena_cap, out_offset, out_len) -> i32
    //   (host provides arena; plugin writes into it; STATUS_BUFFER_TOO_SMALL
    //    with needed size in out_len if too small)
    let fn_type = match ir.attrs.buffer_strategy {
        BufferStrategyAttr::PluginAllocated => quote! {
            unsafe extern "C" fn(
                *const u8, u32,
                *mut *mut u8, *mut u32,
            ) -> i32
        },
        BufferStrategyAttr::Arena => quote! {
            unsafe extern "C" fn(
                *const u8, u32,
                *mut u8, u32,
                *mut u32, *mut u32,
            ) -> i32
        },
    };

    let fields: Vec<TokenStream> = ir
        .methods
        .iter()
        .map(|m| {
            let field_name = &m.name;
            if m.optional_since.is_some() {
                quote! { pub #field_name: Option<#fn_type> }
            } else {
                quote! { pub #field_name: #fn_type }
            }
        })
        .collect();

    // Constructor function that takes bare fn pointers and wraps optional ones in Some()
    let constructor_name = format_ident!("new_{}_vtable", ir.trait_name.to_string().to_lowercase());

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
    let crate_path = &ir.attrs.crate_path;
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

    let any_method_meta = ir.methods.iter().any(|m| !m.method_metas.is_empty());
    let method_metadata_expr = if any_method_meta {
        quote! { __FIDIUS_METHOD_META_TABLE.as_ptr() }
    } else {
        quote! { ::std::ptr::null() }
    };

    let has_trait_meta = !ir.trait_metas.is_empty();
    let trait_meta_count = ir.trait_metas.len() as u32;
    let trait_metadata_expr = if has_trait_meta {
        quote! { __FIDIUS_TRAIT_META.as_ptr() }
    } else {
        quote! { ::std::ptr::null() }
    };

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
        ) -> #crate_path::descriptor::PluginDescriptor {
            #crate_path::descriptor::PluginDescriptor {
                descriptor_size: std::mem::size_of::<#crate_path::descriptor::PluginDescriptor>() as u32,
                abi_version: #crate_path::descriptor::ABI_VERSION,
                interface_name: #interface_name_cstr_ident.as_ptr(),
                interface_hash: #hash_name,
                interface_version: #version_name,
                capabilities,
                buffer_strategy: #strategy_name,
                plugin_name,
                vtable: vtable as *const std::ffi::c_void,
                free_buffer,
                method_count,
                method_metadata: #method_metadata_expr,
                trait_metadata: #trait_metadata_expr,
                trait_metadata_count: #trait_meta_count,
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

/// Generate a typed `{Trait}Client` struct that wraps a `PluginHandle` and
/// exposes named methods matching the trait signature. Eliminates raw
/// `call_method(index, ...)` boilerplate at host call sites.
///
/// Emission is gated with `#[cfg(feature = "host")]` so plugin cdylibs that
/// don't enable the `host` feature pay zero cost (no `fidius-host`, no
/// `libloading` in the dep tree). Host applications enable the feature on
/// the interface crate to receive the generated Client type.
///
/// Uniform tuple encoding matches the plugin-side shim (see
/// `impl_macro.rs::generate_shims`): args are always serialized as a tuple
/// `(arg1, arg2, ...)` — for zero args this is `()`; for one arg `(arg,)`.
fn generate_client(ir: &InterfaceIR) -> TokenStream {
    let trait_name = &ir.trait_name;
    let client_name = format_ident!("{}Client", trait_name);
    let crate_path = &ir.attrs.crate_path;
    let companion_mod = format_ident!("__fidius_{}", trait_name);
    let hash_name = format_ident!("{}_INTERFACE_HASH", trait_name);

    let methods: Vec<TokenStream> = ir
        .methods
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let method_name = &m.name;
            let index = i;
            let arg_types = &m.arg_types;
            let arg_names = &m.arg_names;

            let ret_type = match &m.return_type {
                Some(ty) => quote! { #ty },
                None => quote! { () },
            };

            // For optional methods, check capability before calling
            let cap_check = if m.optional_since.is_some() {
                let cap_bit = ir
                    .methods
                    .iter()
                    .filter(|mm| mm.optional_since.is_some())
                    .position(|mm| mm.name == m.name)
                    .unwrap_or(0) as u32;
                quote! {
                    if !self.handle.has_capability(#cap_bit) {
                        return Err(#crate_path::CallError::NotImplemented { bit: #cap_bit });
                    }
                }
            } else {
                quote! {}
            };

            // Uniform tuple encoding — matches plugin-side shim deserialization
            // of `(T1, T2, ..., Tn)`. No arg_names → `&()` (unit). One arg →
            // `&(arg,)` (1-tuple). N args → `&(a, b, c,)`.
            quote! {
                pub fn #method_name(
                    &self,
                    #(#arg_names: &#arg_types,)*
                ) -> ::std::result::Result<#ret_type, #crate_path::CallError> {
                    #cap_check
                    self.handle.call_method(#index, &(#(#arg_names,)*))
                }
            }
        })
        .collect();

    quote! {
        /// Typed client for calling plugin methods by name.
        ///
        /// Wraps a `PluginHandle` and provides methods matching the trait's
        /// signatures, eliminating raw index-based `call_method` usage.
        ///
        /// Only available when the downstream crate enables the `host` feature.
        #[cfg(feature = "host")]
        pub struct #client_name {
            handle: #crate_path::PluginHandle,
        }

        #[cfg(feature = "host")]
        impl #client_name {
            /// Create a client from a loaded plugin handle.
            pub fn from_handle(handle: #crate_path::PluginHandle) -> Self {
                Self { handle }
            }

            /// Construct a client that calls a plugin linked into the current
            /// process (not loaded from a dylib). The plugin must have been
            /// registered via `#[plugin_impl]` in an rlib linked into this
            /// binary — typically for in-process testing of plugin code.
            ///
            /// `plugin_name` matches the struct name passed to `#[plugin_impl]`
            /// (e.g., `BasicCalculator`).
            ///
            /// Returns `LoadError::PluginNotFound` if no descriptor with that
            /// name is in the inventory, or `LoadError::InterfaceHashMismatch`
            /// if the plugin was built against a different version of the
            /// interface.
            pub fn in_process(plugin_name: &str) -> ::std::result::Result<Self, #crate_path::LoadError> {
                let desc = #crate_path::PluginHandle::find_in_process_descriptor(plugin_name)?;
                let expected_hash = #companion_mod::#hash_name;
                if desc.interface_hash != expected_hash {
                    return Err(#crate_path::LoadError::InterfaceHashMismatch {
                        got: desc.interface_hash,
                        expected: expected_hash,
                    });
                }
                let handle = #crate_path::PluginHandle::from_descriptor(desc)?;
                Ok(Self::from_handle(handle))
            }

            /// Access the underlying handle for raw method calls or metadata.
            pub fn handle(&self) -> &#crate_path::PluginHandle {
                &self.handle
            }

            #(#methods)*
        }
    }
}
