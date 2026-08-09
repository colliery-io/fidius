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

//! Code generation for `#[host_interface]` — the plugin → host callback
//! channel (the reverse direction of `#[plugin_interface]`).
//!
//! From one trait, generates both ends of the channel:
//!
//! - **Plugin side** (`<Trait>Client` + companion statics): a once-bindable
//!   table cell, the `bind` shim the host installs a [`HostFunctionTable`]
//!   through (with defensive hash/version re-validation), a
//!   `HostImportDescriptor` registered with the plugin's host-import
//!   registry, and a typed client whose methods bincode-encode arguments
//!   and dispatch through the bound table.
//! - **Host side** (`<Trait>Binding`): a dispatch shim that decodes
//!   arguments, calls an `Arc<dyn Trait>` implementation with a panic
//!   catch, and encodes results; plus bind entry points for dynamically
//!   loaded libraries (`bind`, feature `"host"`) and in-process plugins
//!   (`bind_in_process`).
//!
//! Both plugin runtimes are served: **dylib** plugins receive a
//! function-pointer table at bind time (gated at load), and **wasm** plugins
//! dispatch through the `fidius:host-call` component import (gated on every
//! call by the identity triple the client sends). The table/bind machinery
//! is `#[cfg(not(target_family = "wasm"))]`; the wasm client is
//! `#[cfg(target_family = "wasm")]`. See the macro's rustdoc in `lib.rs`
//! for the threading/reentrancy contract.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    FnArg, Ident, ItemTrait, LitInt, LitStr, Pat, Path, ReturnType, Token, TraitItem, TraitItemFn,
    Type,
};

/// Parsed attributes from `#[host_interface(version = N)]`.
pub struct HostInterfaceAttrs {
    pub version: u32,
    /// Path to the fidius crate (`crate = "my_crate::fidius"`); defaults to `fidius`.
    pub crate_path: Path,
}

impl Parse for HostInterfaceAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut version = None;
        let mut crate_path = None;

        while !input.is_empty() {
            let key_str = if input.peek(Token![crate]) {
                let _kw: Token![crate] = input.parse()?;
                "crate".to_string()
            } else {
                let ident: Ident = input.parse()?;
                ident.to_string()
            };
            let _eq: Token![=] = input.parse()?;

            match key_str.as_str() {
                "version" => {
                    let lit: LitInt = input.parse()?;
                    version = Some(lit.base10_parse::<u32>()?);
                }
                "crate" => {
                    let lit: LitStr = input.parse()?;
                    crate_path = Some(lit.parse::<Path>()?);
                }
                other => {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!("unknown attribute `{other}`, expected `version` or `crate`"),
                    ))
                }
            }
            if !input.is_empty() {
                let _comma: Token![,] = input.parse()?;
            }
        }

        Ok(HostInterfaceAttrs {
            version: version
                .ok_or_else(|| syn::Error::new(Span::call_site(), "missing `version` attribute"))?,
            crate_path: crate_path.unwrap_or_else(|| syn::parse_str::<Path>("fidius").unwrap()),
        })
    }
}

/// IR for a single host-function method.
struct HostMethod {
    name: Ident,
    arg_names: Vec<Ident>,
    arg_types: Vec<Type>,
    /// The full declared return type (`None` for `-> ()`).
    return_type: Option<Type>,
    /// `Some(ok_type)` when the return is `Result<T, PluginError>`.
    result_ok: Option<Type>,
    signature_string: String,
}

/// Return `Some(&last_segment)` if `ty` is a path type.
fn last_segment(ty: &Type) -> Option<&syn::PathSegment> {
    match ty {
        Type::Path(p) => p.path.segments.last(),
        _ => None,
    }
}

/// If `ty` is `Result<T, E>`, return `(T, E)`.
fn result_types(ty: &Type) -> Option<(&Type, &Type)> {
    let seg = last_segment(ty)?;
    if seg.ident != "Result" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    let mut types = args.args.iter().filter_map(|a| match a {
        syn::GenericArgument::Type(t) => Some(t),
        _ => None,
    });
    Some((types.next()?, types.next()?))
}

/// Return `true` if the type's final path segment is `Stream` with one
/// generic argument (the `fidius::Stream<T>` streaming marker).
fn is_stream_marker(ty: &Type) -> bool {
    match last_segment(ty) {
        Some(seg) if seg.ident == "Stream" => {
            matches!(&seg.arguments, syn::PathArguments::AngleBracketed(a) if a.args.len() == 1)
        }
        _ => false,
    }
}

/// Parse and validate one trait method into a [`HostMethod`].
fn parse_method(method: &TraitItemFn) -> syn::Result<HostMethod> {
    let span = method.sig.ident.span();

    if method.sig.asyncness.is_some() {
        return Err(syn::Error::new(
            span,
            "host functions are synchronous at the FFI boundary; a host implementation that \
             needs to await must bridge to its own runtime internally (e.g. \
             `tokio::runtime::Handle::block_on`) — see the host_interface threading contract",
        ));
    }
    if !method.sig.generics.params.is_empty() {
        return Err(syn::Error::new(
            span,
            "host interface methods cannot be generic",
        ));
    }
    match method.sig.inputs.first() {
        Some(FnArg::Receiver(r)) if r.mutability.is_none() && r.reference.is_some() => {}
        _ => {
            return Err(syn::Error::new(
                span,
                "host interface methods must take `&self` (the host implementation is shared \
                 across plugin threads)",
            ))
        }
    }
    for attr in &method.attrs {
        for unsupported in ["optional", "wire", "method_meta"] {
            if attr.path().is_ident(unsupported) {
                return Err(syn::Error::new(
                    attr.span(),
                    format!("`#[{unsupported}]` is not supported on host interface methods"),
                ));
            }
        }
    }

    let mut arg_names = Vec::new();
    let mut arg_types = Vec::new();
    for arg in &method.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            if matches!(pat_type.ty.as_ref(), Type::Reference(_)) {
                return Err(syn::Error::new(
                    pat_type.ty.span(),
                    "host interface arguments must be owned types (String, Vec<u8>, …), not \
                     references — they are serialized across the FFI boundary",
                ));
            }
            if is_stream_marker(&pat_type.ty) {
                return Err(syn::Error::new(
                    pat_type.ty.span(),
                    "streaming host functions are not supported — host calls are \
                     request/response only",
                ));
            }
            arg_types.push((*pat_type.ty).clone());
            arg_names.push(match pat_type.pat.as_ref() {
                Pat::Ident(p) => p.ident.clone(),
                _ => Ident::new("_arg", pat_type.span()),
            });
        }
    }

    let return_type = match &method.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some((**ty).clone()),
    };
    if let Some(rt) = &return_type {
        if is_stream_marker(rt) {
            return Err(syn::Error::new(
                rt.span(),
                "streaming host functions are not supported — host calls are \
                 request/response only",
            ));
        }
    }

    // A fallible host function must use `PluginError` as its error type: it is
    // the one typed error currency both sides of the wire agree on. Detected
    // by final path segment so `fidius::PluginError` and a re-export both work.
    let result_ok = match return_type.as_ref().and_then(result_types) {
        Some((ok, err)) => {
            let is_plugin_error = last_segment(err).map(|s| s.ident == "PluginError") == Some(true);
            if !is_plugin_error {
                return Err(syn::Error::new(
                    err.span(),
                    "fallible host functions must return `Result<T, fidius::PluginError>` — \
                     encode domain errors in PluginError's code/message/details so they cross \
                     the boundary as typed, inspectable errors",
                ));
            }
            Some(ok.clone())
        }
        None => None,
    };

    let arg_type_strs: Vec<String> = arg_types
        .iter()
        .map(|t| quote::ToTokens::to_token_stream(t).to_string())
        .collect();
    let ret_str = match &return_type {
        Some(t) => quote::ToTokens::to_token_stream(t).to_string(),
        None => String::new(),
    };
    let signature_string = fidius_core::hash::signature_string(
        &method.sig.ident.to_string(),
        &arg_type_strs,
        &ret_str,
        false,
        false,
        false,
    );

    Ok(HostMethod {
        name: method.sig.ident.clone(),
        arg_names,
        arg_types,
        return_type,
        result_ok,
        signature_string,
    })
}

/// Generate all code for a `#[host_interface]` invocation.
pub fn generate_host_interface(
    attrs: &HostInterfaceAttrs,
    item: &ItemTrait,
) -> syn::Result<TokenStream> {
    let trait_name = &item.ident;
    let crate_path = &attrs.crate_path;

    if !item.generics.params.is_empty() {
        return Err(syn::Error::new(
            item.generics.span(),
            "host interface traits cannot be generic",
        ));
    }
    // The table is shared across plugin threads: require the trait itself to
    // promise thread-safety, exactly like `#[plugin_interface]` traits do.
    let has_send = item
        .supertraits
        .iter()
        .any(|b| matches!(b, syn::TypeParamBound::Trait(t) if t.path.is_ident("Send")));
    let has_sync = item
        .supertraits
        .iter()
        .any(|b| matches!(b, syn::TypeParamBound::Trait(t) if t.path.is_ident("Sync")));
    if !has_send || !has_sync {
        return Err(syn::Error::new(
            item.ident.span(),
            "host interface traits must declare `Send + Sync` supertraits — plugin threads \
             call host functions concurrently",
        ));
    }

    let mut methods = Vec::new();
    for trait_item in &item.items {
        if let TraitItem::Fn(m) = trait_item {
            methods.push(parse_method(m)?);
        }
    }
    if methods.is_empty() {
        return Err(syn::Error::new(
            item.ident.span(),
            "host interface traits must declare at least one method",
        ));
    }

    let sigs: Vec<&str> = methods
        .iter()
        .map(|m| m.signature_string.as_str())
        .collect();
    let hash_value = fidius_core::hash::interface_hash(&sigs);
    let version_val = attrs.version;
    let fn_count = methods.len() as u32;

    let companion_mod = format_ident!("__fidius_host_{}", trait_name);
    let hash_name = format_ident!("{}_HOST_INTERFACE_HASH", trait_name);
    let version_name = format_ident!("{}_HOST_INTERFACE_VERSION", trait_name);
    let count_name = format_ident!("{}_HOST_FN_COUNT", trait_name);
    let client_name = format_ident!("{}Client", trait_name);
    let binding_name = format_ident!("{}Binding", trait_name);
    let trait_name_str = trait_name.to_string();

    // Per-method dispatch index constants.
    let index_consts: Vec<TokenStream> = methods
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let const_name = format_ident!("HOST_FN_{}", m.name.to_string().to_uppercase());
            let i = i as u32;
            let doc = format!("Dispatch index for `{}`.", m.name);
            quote! {
                #[doc = #doc]
                pub const #const_name: u32 = #i;
            }
        })
        .collect();

    // ── Host-side dispatch arms ─────────────────────────────────────────────
    let dispatch_arms: Vec<TokenStream> = methods
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let i = i as u32;
            let mname = &m.name;
            let arg_names = &m.arg_names;
            let arg_types = &m.arg_types;

            let output_handling = if m.result_ok.is_some() {
                quote! {
                    match __output {
                        ::core::result::Result::Ok(val) => {
                            match #crate_path::wire::serialize(&val) {
                                Ok(v) => (v, #crate_path::status::STATUS_OK),
                                Err(_) => return #crate_path::status::STATUS_SERIALIZATION_ERROR,
                            }
                        }
                        ::core::result::Result::Err(err) => {
                            match #crate_path::wire::serialize(&err) {
                                Ok(v) => (v, #crate_path::status::STATUS_PLUGIN_ERROR),
                                Err(_) => return #crate_path::status::STATUS_SERIALIZATION_ERROR,
                            }
                        }
                    }
                }
            } else {
                quote! {
                    match #crate_path::wire::serialize(&__output) {
                        Ok(v) => (v, #crate_path::status::STATUS_OK),
                        Err(_) => return #crate_path::status::STATUS_SERIALIZATION_ERROR,
                    }
                }
            };

            quote! {
                #i => {
                    let (#(#arg_names,)*) = match #crate_path::wire::deserialize::<(#(#arg_types,)*)>(__in_slice) {
                        Ok(v) => v,
                        Err(_) => return #crate_path::status::STATUS_SERIALIZATION_ERROR,
                    };
                    let __output = __host.#mname(#(#arg_names),*);
                    let (__bytes, __status) = #output_handling;
                    let __boxed: ::std::boxed::Box<[u8]> = __bytes.into_boxed_slice();
                    let __len = __boxed.len();
                    let __ptr = ::std::boxed::Box::into_raw(__boxed) as *mut u8;
                    unsafe { *out_ptr = __ptr; *out_len = __len as u32; }
                    __status
                }
            }
        })
        .collect();

    // ── Plugin-side client methods ──────────────────────────────────────────
    let client_methods: Vec<TokenStream> = methods
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let i = i as u32;
            let mname = &m.name;
            let arg_names = &m.arg_names;
            let arg_types = &m.arg_types;
            let ok_ty = match (&m.result_ok, &m.return_type) {
                (Some(ok), _) => quote! { #ok },
                (None, Some(rt)) => quote! { #rt },
                (None, None) => quote! { () },
            };
            let err_doc = if m.result_ok.is_some() {
                "A host-raised `PluginError` arrives as `HostCallError::Host`; "
            } else {
                ""
            };
            let doc = format!(
                "Call the host's `{mname}`. {err_doc}transport-level failures \
                 (unbound interface, host panic, serialization) arrive as the other \
                 `HostCallError` variants."
            );
            quote! {
                #[doc = #doc]
                pub fn #mname(
                    &self,
                    #(#arg_names: &#arg_types,)*
                ) -> ::std::result::Result<#ok_ty, #crate_path::host_ffi::HostCallError> {
                    let __input = #crate_path::wire::serialize(&(#(#arg_names,)*))
                        .map_err(|e| #crate_path::host_ffi::HostCallError::Serialization(e.to_string()))?;
                    let __out = #crate_path::host_ffi::call_host_fn(self.table, #i, &__input)?;
                    #crate_path::wire::deserialize(&__out)
                        .map_err(|e| #crate_path::host_ffi::HostCallError::Deserialization(e.to_string()))
                }
            }
        })
        .collect();

    // ── Plugin-side client methods, wasm variant ────────────────────────────
    // Same signatures; dispatch goes through the `fidius:host-call` import,
    // carrying the identity triple so the host gates every call.
    let wasm_client_methods: Vec<TokenStream> = methods
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let i = i as u32;
            let mname = &m.name;
            let arg_names = &m.arg_names;
            let arg_types = &m.arg_types;
            let ok_ty = match (&m.result_ok, &m.return_type) {
                (Some(ok), _) => quote! { #ok },
                (None, Some(rt)) => quote! { #rt },
                (None, None) => quote! { () },
            };
            let doc = format!(
                "Call the host's `{mname}` through the `fidius:host-call` import. \
                 The host gates the call on this interface's version + signature \
                 hash before dispatching."
            );
            quote! {
                #[doc = #doc]
                pub fn #mname(
                    &self,
                    #(#arg_names: &#arg_types,)*
                ) -> ::std::result::Result<#ok_ty, #crate_path::host_ffi::HostCallError> {
                    let __input = #crate_path::wire::serialize(&(#(#arg_names,)*))
                        .map_err(|e| #crate_path::host_ffi::HostCallError::Serialization(e.to_string()))?;
                    let __out = #crate_path::host_call::call(
                        #trait_name_str,
                        #companion_mod::#version_name,
                        #companion_mod::#hash_name,
                        #i,
                        &__input,
                    )?;
                    #crate_path::wire::deserialize(&__out)
                        .map_err(|e| #crate_path::host_ffi::HostCallError::Deserialization(e.to_string()))
                }
            }
        })
        .collect();

    let cleaned_trait = strip_helper_attrs(item);

    let client_doc = format!(
        "Typed plugin-side client for the `{trait_name_str}` host interface.\n\n\
         Obtain with [`{client_name}::bound()`] from inside plugin code after the host \
         has installed its function table (at plugin load/bind time). Calls are \
         synchronous; see the `#[fidius::host_interface]` threading contract for what a \
         host implementation may assume about the calling thread."
    );
    let binding_doc = format!(
        "Host-side binding for the `{trait_name_str}` host interface.\n\n\
         Wraps a host implementation (`Arc<dyn {trait_name_str}>`) in a C-ABI \
         function table and installs it into a plugin. Each successful bind \
         intentionally leaks one table and one `Arc` clone (a few dozen bytes, \
         once per loaded library) so in-flight plugin calls can never observe a \
         dangling table."
    );

    Ok(quote! {
        #cleaned_trait

        /// Generated companion module for the host interface (plugin → host
        /// callback channel): identity constants, the plugin-side table cell +
        /// bind shim + import descriptor, and the host-side dispatch machinery.
        #[allow(non_snake_case, non_upper_case_globals, dead_code)]
        pub mod #companion_mod {
            use super::*;

            /// FNV-1a hash of this host interface's method signatures.
            pub const #hash_name: u64 = #hash_value;
            /// Declared `#[host_interface(version = N)]`.
            pub const #version_name: u32 = #version_val;
            /// Number of host functions in the table.
            pub const #count_name: u32 = #fn_count;
            #(#index_consts)*

            const __FIDIUS_HOST_IFACE_NAME: &::std::ffi::CStr = unsafe {
                ::std::ffi::CStr::from_bytes_with_nul_unchecked(
                    concat!(#trait_name_str, "\0").as_bytes(),
                )
            };

            // ── Plugin side ────────────────────────────────────────────────

            /// The once-bindable cell holding the host's function table.
            #[cfg(not(target_family = "wasm"))]
            pub static __FIDIUS_HOST_TABLE: #crate_path::host_ffi::HostTableCell =
                #crate_path::host_ffi::HostTableCell::new();

            /// Bind shim the host installs its table through. Re-validates
            /// ABI/hash/version/fn_count defensively before storing — a
            /// mismatched table can never become callable.
            #[cfg(not(target_family = "wasm"))]
            pub unsafe extern "C" fn __fidius_host_bind(
                table: *const #crate_path::host_ffi::HostFunctionTable,
            ) -> i32 {
                let result = ::std::panic::catch_unwind(|| {
                    if table.is_null() {
                        return #crate_path::host_ffi::BIND_ERR_NULL;
                    }
                    let t = unsafe { &*table };
                    if t.abi_version != #crate_path::descriptor::ABI_VERSION {
                        return #crate_path::host_ffi::BIND_ERR_ABI;
                    }
                    if t.interface_hash != #hash_name {
                        return #crate_path::host_ffi::BIND_ERR_HASH_MISMATCH;
                    }
                    if t.interface_version != #version_name {
                        return #crate_path::host_ffi::BIND_ERR_VERSION_MISMATCH;
                    }
                    if t.fn_count != #count_name {
                        return #crate_path::host_ffi::BIND_ERR_FN_COUNT;
                    }
                    // SAFETY: validated above; the table has process lifetime
                    // per the HostFunctionTable contract.
                    unsafe { __FIDIUS_HOST_TABLE.bind(table) }
                });
                match result {
                    ::core::result::Result::Ok(s) => s,
                    ::core::result::Result::Err(_) => #crate_path::host_ffi::BIND_ERR_PANIC,
                }
            }

            /// This plugin's declaration that it can consume the interface.
            #[cfg(not(target_family = "wasm"))]
            pub static HOST_IMPORT_DESCRIPTOR: #crate_path::host_ffi::HostImportDescriptor =
                #crate_path::host_ffi::HostImportDescriptor {
                    descriptor_size: ::std::mem::size_of::<
                        #crate_path::host_ffi::HostImportDescriptor,
                    >() as u32,
                    abi_version: #crate_path::descriptor::ABI_VERSION,
                    interface_name: __FIDIUS_HOST_IFACE_NAME.as_ptr(),
                    interface_hash: #hash_name,
                    interface_version: #version_name,
                    bind: __fidius_host_bind,
                };

            // Advertise the import through the plugin's (optional)
            // `fidius_get_host_imports` export.
            #[cfg(not(target_family = "wasm"))]
            #crate_path::inventory::submit! {
                #crate_path::host_registry::HostImportEntry {
                    descriptor: &HOST_IMPORT_DESCRIPTOR,
                }
            }

            // ── Host side ──────────────────────────────────────────────────

            #[cfg(not(target_family = "wasm"))]
            unsafe extern "C" fn __fidius_host_dispatch(
                ctx: *mut ::core::ffi::c_void,
                index: u32,
                in_ptr: *const u8,
                in_len: u32,
                out_ptr: *mut *mut u8,
                out_len: *mut u32,
            ) -> i32 {
                let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    let __depth = #crate_path::host_ffi::CallbackDepthGuard::new();
                    let __host: &::std::sync::Arc<dyn #trait_name> =
                        unsafe { &*(ctx as *const ::std::sync::Arc<dyn #trait_name>) };
                    let __in_slice =
                        unsafe { ::std::slice::from_raw_parts(in_ptr, in_len as usize) };
                    match index {
                        #(#dispatch_arms)*
                        _ => #crate_path::status::STATUS_INVALID_INDEX,
                    }
                }));
                match result {
                    ::core::result::Result::Ok(status) => status,
                    ::core::result::Result::Err(panic_payload) => {
                        let msg = panic_payload
                            .downcast_ref::<&str>()
                            .map(|s| s.to_string())
                            .or_else(|| panic_payload.downcast_ref::<String>().cloned())
                            .unwrap_or_else(|| "unknown panic".to_string());
                        if let Ok(msg_bytes) = #crate_path::wire::serialize(&msg) {
                            let boxed: ::std::boxed::Box<[u8]> = msg_bytes.into_boxed_slice();
                            let len = boxed.len();
                            let ptr = ::std::boxed::Box::into_raw(boxed) as *mut u8;
                            unsafe { *out_ptr = ptr; *out_len = len as u32; }
                        }
                        #crate_path::status::STATUS_PANIC
                    }
                }
            }

            #[cfg(not(target_family = "wasm"))]
            unsafe extern "C" fn __fidius_host_free(ptr: *mut u8, len: usize) {
                if !ptr.is_null() && len > 0 {
                    // Reconstruct the Box<[u8]>: dispatch always allocates
                    // output via into_boxed_slice, so cap == len.
                    unsafe {
                        let slice = ::std::slice::from_raw_parts_mut(ptr, len);
                        drop(::std::boxed::Box::from_raw(slice as *mut [u8]));
                    }
                }
            }

            /// Build a process-lifetime `HostFunctionTable` wrapping `host`.
            /// The table and the boxed `Arc` are intentionally leaked.
            #[cfg(not(target_family = "wasm"))]
            pub fn __fidius_build_host_table(
                host: ::std::sync::Arc<dyn #trait_name>,
            ) -> *const #crate_path::host_ffi::HostFunctionTable {
                let ctx = ::std::boxed::Box::into_raw(::std::boxed::Box::new(host))
                    as *mut ::core::ffi::c_void;
                ::std::boxed::Box::into_raw(::std::boxed::Box::new(
                    #crate_path::host_ffi::HostFunctionTable {
                        table_size: ::std::mem::size_of::<
                            #crate_path::host_ffi::HostFunctionTable,
                        >() as u32,
                        abi_version: #crate_path::descriptor::ABI_VERSION,
                        interface_name: __FIDIUS_HOST_IFACE_NAME.as_ptr(),
                        interface_hash: #hash_name,
                        interface_version: #version_name,
                        fn_count: #count_name,
                        ctx,
                        dispatch: __fidius_host_dispatch,
                        free_buffer: __fidius_host_free,
                    },
                ))
            }
        }

        #[doc = #client_doc]
        #[cfg(not(target_family = "wasm"))]
        pub struct #client_name {
            table: &'static #crate_path::host_ffi::HostFunctionTable,
        }

        #[cfg(not(target_family = "wasm"))]
        impl #client_name {
            /// The bound host functions, or `HostCallError::NotBound` if the
            /// host application never installed a table for this interface.
            pub fn bound() -> ::std::result::Result<Self, #crate_path::host_ffi::HostCallError> {
                match #companion_mod::__FIDIUS_HOST_TABLE.get() {
                    ::core::option::Option::Some(table) => Ok(Self { table }),
                    ::core::option::Option::None => {
                        Err(#crate_path::host_ffi::HostCallError::NotBound {
                            interface: #trait_name_str,
                        })
                    }
                }
            }

            /// Whether the host has bound this interface.
            pub fn is_bound() -> bool {
                #companion_mod::__FIDIUS_HOST_TABLE.get().is_some()
            }

            #(#client_methods)*
        }

        #[doc = #binding_doc]
        #[cfg(not(target_family = "wasm"))]
        pub struct #binding_name;

        #[cfg(not(target_family = "wasm"))]
        impl #binding_name {
            /// The interface's FNV-1a signature hash (what a plugin must have
            /// been built against for a bind to succeed).
            pub const INTERFACE_HASH: u64 = #companion_mod::#hash_name;
            /// The interface's declared version.
            pub const INTERFACE_VERSION: u32 = #companion_mod::#version_name;
            /// The interface (trait) name plugins import it under.
            pub const INTERFACE_NAME: &'static str = #trait_name_str;

            /// Build a process-lifetime function table wrapping `host` (leaks
            /// one table + `Arc` clone). Prefer [`Self::bind`] /
            /// [`Self::bind_in_process`], which validate and install it.
            pub fn table(
                host: ::std::sync::Arc<dyn #trait_name>,
            ) -> *const #crate_path::host_ffi::HostFunctionTable {
                #companion_mod::__fidius_build_host_table(host)
            }

            /// Bind `host` for a plugin linked into the current process (an
            /// in-process `#[plugin_impl]`, e.g. in tests). The dynamic-load
            /// path is [`Self::bind`].
            pub fn bind_in_process(
                host: ::std::sync::Arc<dyn #trait_name>,
            ) -> ::std::result::Result<(), #crate_path::host_ffi::HostBindError> {
                let table = Self::table(host);
                // SAFETY: `table` is freshly built with process lifetime.
                let status = unsafe { #companion_mod::__fidius_host_bind(table) };
                #crate_path::host_ffi::bind_status_to_result(#trait_name_str, status)
            }

            /// Bind `host` into a dynamically loaded plugin library, gating on
            /// the host-interface version and signature hash the plugin was
            /// built against.
            ///
            /// Returns `Ok(true)` when the table was installed, `Ok(false)`
            /// when the plugin does not import this interface (older plugin,
            /// or one that simply doesn't use host functions) — callers that
            /// *require* the plugin to consume the interface should treat
            /// `false` as an error. A version or hash mismatch fails loudly
            /// with `LoadError::HostInterfaceVersionMismatch` /
            /// `LoadError::HostInterfaceHashMismatch` and installs nothing,
            /// so a mismatched surface can never mis-dispatch.
            #[cfg(feature = "host")]
            pub fn bind(
                library: &#crate_path::LoadedLibrary,
                host: ::std::sync::Arc<dyn #trait_name>,
            ) -> ::std::result::Result<bool, #crate_path::LoadError> {
                #crate_path::host_import::bind_host_interface(
                    &library.library,
                    #trait_name_str,
                    #companion_mod::#hash_name,
                    #companion_mod::#version_name,
                    || Self::table(host),
                )
            }

            /// Like [`Self::bind`], for a `LoadedPlugin` (the value
            /// `PluginHost::load` returns). Binding is per-**library**: if a
            /// dylib contains several plugins they share the one table.
            #[cfg(feature = "host")]
            pub fn bind_plugin(
                plugin: &#crate_path::LoadedPlugin,
                host: ::std::sync::Arc<dyn #trait_name>,
            ) -> ::std::result::Result<bool, #crate_path::LoadError> {
                #crate_path::host_import::bind_host_interface(
                    &plugin.library,
                    #trait_name_str,
                    #companion_mod::#hash_name,
                    #companion_mod::#version_name,
                    || Self::table(host),
                )
            }

            /// Bind `host` into a **WASM-backed** `PluginHandle` — the wasm
            /// variant of [`Self::bind`]. The guest dispatches host functions
            /// through the `fidius:host-call` import, which gates **every**
            /// call on this interface's version + signature hash (the wasm
            /// counterpart of the dylib bind-time gate): a plugin built
            /// against a different revision gets a typed
            /// `HostCallError::VersionMismatch` / `HashMismatch` on its first
            /// call, never a bincode mis-dispatch. Once-only per handle.
            ///
            /// Requires the interface crate's `host` and `wasm` features
            /// (forwarding to `fidius/host` + `fidius/wasm`).
            #[cfg(all(feature = "host", feature = "wasm"))]
            pub fn bind_wasm(
                handle: &#crate_path::PluginHandle,
                host: ::std::sync::Arc<dyn #trait_name>,
            ) -> ::std::result::Result<(), #crate_path::LoadError> {
                // SAFETY: `table` builds a fresh, intentionally-leaked
                // (process-lifetime) table — exactly the bind contract.
                unsafe { handle.bind_wasm_host_table(Self::table(host)) }
            }
        }

        #[doc = #client_doc]
        #[cfg(target_family = "wasm")]
        pub struct #client_name {
            _priv: (),
        }

        #[cfg(target_family = "wasm")]
        impl #client_name {
            /// The host functions, if the host bound a matching table.
            /// Probes the `fidius:host-call` import: an unbound interface
            /// returns `HostCallError::NotBound`; a host providing a
            /// different revision returns the typed
            /// `VersionMismatch`/`HashMismatch` error.
            pub fn bound() -> ::std::result::Result<Self, #crate_path::host_ffi::HostCallError> {
                #crate_path::host_call::probe(
                    #trait_name_str,
                    #companion_mod::#version_name,
                    #companion_mod::#hash_name,
                )?;
                Ok(Self { _priv: () })
            }

            /// Whether the host has bound a matching table for this interface.
            pub fn is_bound() -> bool {
                #crate_path::host_call::probe(
                    #trait_name_str,
                    #companion_mod::#version_name,
                    #companion_mod::#hash_name,
                )
                .is_ok()
            }

            #(#wasm_client_methods)*
        }
    })
}

/// Strip fidius helper attributes so the emitted trait compiles bare.
fn strip_helper_attrs(item: &ItemTrait) -> ItemTrait {
    let mut cleaned = item.clone();
    for trait_item in &mut cleaned.items {
        if let TraitItem::Fn(method) = trait_item {
            method
                .attrs
                .retain(|a| !a.path().is_ident("optional") && !a.path().is_ident("wire"));
        }
    }
    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn attrs(version: u32) -> HostInterfaceAttrs {
        HostInterfaceAttrs {
            version,
            crate_path: syn::parse_str("fidius").unwrap(),
        }
    }

    fn parse_trait(tokens: proc_macro2::TokenStream) -> ItemTrait {
        syn::parse2(tokens).expect("trait parses")
    }

    #[test]
    fn generates_for_a_simple_host_trait() {
        let item = parse_trait(quote! {
            pub trait TestHost: Send + Sync {
                fn release_slot(&self, id: String) -> Result<(), PluginError>;
                fn get_value(&self, key: String) -> String;
            }
        });
        let out = generate_host_interface(&attrs(1), &item).expect("generates");
        let s = out.to_string();
        assert!(s.contains("TestHostClient"));
        assert!(s.contains("TestHostBinding"));
        assert!(s.contains("TestHost_HOST_INTERFACE_HASH"));
        assert!(s.contains("HOST_FN_RELEASE_SLOT"));
    }

    #[test]
    fn rejects_async_methods() {
        let item = parse_trait(quote! {
            pub trait H: Send + Sync {
                async fn f(&self, x: u32) -> u32;
            }
        });
        let err = generate_host_interface(&attrs(1), &item).unwrap_err();
        assert!(err.to_string().contains("synchronous"));
    }

    #[test]
    fn rejects_missing_send_sync() {
        let item = parse_trait(quote! {
            pub trait H {
                fn f(&self, x: u32) -> u32;
            }
        });
        let err = generate_host_interface(&attrs(1), &item).unwrap_err();
        assert!(err.to_string().contains("Send + Sync"));
    }

    #[test]
    fn rejects_non_plugin_error_result() {
        let item = parse_trait(quote! {
            pub trait H: Send + Sync {
                fn f(&self, x: u32) -> Result<u32, MyError>;
            }
        });
        let err = generate_host_interface(&attrs(1), &item).unwrap_err();
        assert!(err.to_string().contains("PluginError"));
    }

    #[test]
    fn rejects_reference_arguments() {
        let item = parse_trait(quote! {
            pub trait H: Send + Sync {
                fn f(&self, x: &str) -> u32;
            }
        });
        let err = generate_host_interface(&attrs(1), &item).unwrap_err();
        assert!(err.to_string().contains("owned types"));
    }

    #[test]
    fn rejects_stream_marker() {
        let item = parse_trait(quote! {
            pub trait H: Send + Sync {
                fn f(&self, x: u32) -> fidius::Stream<u64>;
            }
        });
        let err = generate_host_interface(&attrs(1), &item).unwrap_err();
        assert!(err.to_string().contains("request/response"));
    }

    #[test]
    fn rejects_mut_self() {
        let item = parse_trait(quote! {
            pub trait H: Send + Sync {
                fn f(&mut self, x: u32) -> u32;
            }
        });
        let err = generate_host_interface(&attrs(1), &item).unwrap_err();
        assert!(err.to_string().contains("&self"));
    }

    #[test]
    fn hash_is_signature_sensitive() {
        let a = parse_trait(quote! {
            pub trait H: Send + Sync { fn f(&self, x: u32) -> u32; }
        });
        let b = parse_trait(quote! {
            pub trait H: Send + Sync { fn f(&self, x: u64) -> u32; }
        });
        let hash = |item: &ItemTrait| {
            let mut sigs = Vec::new();
            for ti in &item.items {
                if let TraitItem::Fn(m) = ti {
                    sigs.push(parse_method(m).unwrap().signature_string);
                }
            }
            let refs: Vec<&str> = sigs.iter().map(|s| s.as_str()).collect();
            fidius_core::hash::interface_hash(&refs)
        };
        assert_ne!(hash(&a), hash(&b));
    }

    #[test]
    fn attrs_parse_version_and_crate() {
        let a: HostInterfaceAttrs = syn::parse_str("version = 3").unwrap();
        assert_eq!(a.version, 3);
        assert_eq!(a.crate_path.segments.last().unwrap().ident, "fidius");

        let b: HostInterfaceAttrs =
            syn::parse_str(r#"version = 1, crate = "my_crate::fidius""#).unwrap();
        assert_eq!(b.crate_path.segments.len(), 2);
    }
}
