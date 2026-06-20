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

use crate::ir::BufferStrategyAttr;

/// Info about an impl method, extracted from the impl block.
struct MethodInfo<'a> {
    name: &'a Ident,
    is_async: bool,
    returns_result: bool,
    /// Argument types (excluding `self`).
    arg_types: Vec<&'a Type>,
    /// Argument names (excluding `self`).
    arg_names: Vec<Ident>,
    /// Whether the impl method carries `#[wire(raw)]`. Must match the
    /// interface trait's declaration; mismatch surfaces at plugin-load time
    /// as an interface-hash mismatch (the trait's `#[wire(raw)]` flips a
    /// `!raw` marker in the signature string the hash is computed over).
    wire_raw: bool,
    /// The method's return type, if any (`None` for `-> ()`). Used by the
    /// WASM component WIT generator (FIDIUS-T-0106).
    ret_type: Option<&'a Type>,
    /// For a server-streaming method (`-> fidius::Stream<T>`): the item type `T`
    /// (FIDIUS-I-0026). When `Some`, the WASM adapter emits a `resource`
    /// instead of a value-returning func, and the cdylib path is disabled.
    stream_item: Option<&'a Type>,
    /// For a **client-streaming** method — a `Stream<T>` in *argument* position
    /// (FIDIUS-I-0030): the item type `T`. When `Some`, the cdylib shim takes the
    /// host's producer handle and builds the `Stream<T>` the method consumes.
    client_stream_item: Option<&'a Type>,
}

/// Detect a `#[wire(raw)]` attribute on an impl-side method. Mirrors the
/// trait-side parser in `ir.rs` — an explicit `raw` keyword is the only
/// supported form.
fn impl_method_is_raw(attrs: &[syn::Attribute]) -> syn::Result<bool> {
    for attr in attrs {
        if !attr.path().is_ident("wire") {
            continue;
        }
        let mut raw = false;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("raw") {
                raw = true;
                Ok(())
            } else {
                Err(meta.error("expected `raw` — only `#[wire(raw)]` is supported"))
            }
        })?;
        return Ok(raw);
    }
    Ok(false)
}

/// kebab-case → PascalCase, for deriving the wit-bindgen resource type name from
/// a method name: `tick` → resource `tick-stream` → `TickStream` (trait
/// `GuestTickStream`). (FIDIUS-I-0026.)
fn kebab_to_pascal(s: &str) -> String {
    s.split('-')
        .map(|seg| {
            let mut c = seg.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect()
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

/// Arguments to `#[plugin_impl(TraitName)]`, `#[plugin_impl(TraitName, crate = "...")]`,
/// or `#[plugin_impl(TraitName, buffer = Arena)]`.
pub struct PluginImplAttrs {
    pub trait_name: Ident,
    /// The path to the fidius crate. Defaults to `fidius` when not specified.
    pub crate_path: Path,
    /// Must match the interface's `buffer` attribute. Defaults to
    /// `PluginAllocated`. Mismatches produce a vtable fn-pointer type error
    /// at compile time (the emitted shim's signature won't match the
    /// generated vtable's field type).
    pub buffer_strategy: BufferStrategyAttr,
    /// FIDIUS-A-0006: the config type `C` for a configured plugin
    /// (`#[plugin_impl(Trait, config = C)]`). When set, `construct` deserializes
    /// `C` and calls the impl's `fn configure(cfg: C) -> Self`. None = zero-config
    /// (the singleton, constructed from unit).
    pub config: Option<Path>,
}

impl Parse for PluginImplAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let trait_name: Ident = input.parse()?;
        let mut crate_path = None;
        let mut buffer_strategy = None;
        let mut config = None;

        while !input.is_empty() {
            let _comma: Token![,] = input.parse()?;
            if input.peek(Token![crate]) {
                let _kw: Token![crate] = input.parse()?;
                let _eq: Token![=] = input.parse()?;
                let lit: LitStr = input.parse()?;
                let path: Path = lit.parse()?;
                crate_path = Some(path);
            } else {
                let key: Ident = input.parse()?;
                let _eq: Token![=] = input.parse()?;
                match key.to_string().as_str() {
                    "buffer" => {
                        let value: Ident = input.parse()?;
                        buffer_strategy = Some(match value.to_string().as_str() {
                            "PluginAllocated" => BufferStrategyAttr::PluginAllocated,
                            "Arena" => BufferStrategyAttr::Arena,
                            _ => {
                                return Err(syn::Error::new(
                                    value.span(),
                                    "expected PluginAllocated or Arena",
                                ))
                            }
                        });
                    }
                    "config" => {
                        config = Some(input.parse::<Path>()?);
                    }
                    other => {
                        return Err(syn::Error::new(
                            key.span(),
                            format!("unknown plugin_impl attribute `{other}`"),
                        ));
                    }
                }
            }
        }

        let crate_path = crate_path.unwrap_or_else(|| syn::parse_str::<Path>("fidius").unwrap());
        let buffer_strategy = buffer_strategy.unwrap_or(BufferStrategyAttr::PluginAllocated);

        Ok(PluginImplAttrs {
            trait_name,
            crate_path,
            buffer_strategy,
            config,
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
    let mut impl_methods: Vec<MethodInfo> = Vec::new();
    for impl_item in &item.items {
        if let ImplItem::Fn(method) = impl_item {
            let ret_type: Option<&Type> = match &method.sig.output {
                ReturnType::Type(_, ty) => Some(ty.as_ref()),
                ReturnType::Default => None,
            };
            // Server-streaming (`-> fidius::Stream<T>`): the WASM adapter emits a
            // resource; the cdylib path is disabled for this plugin (handled in
            // the codegen below). (FIDIUS-I-0026.)
            let stream_item = ret_type.and_then(crate::wit::stream_item_type);
            let returns_result = ret_type.map(is_result_type).unwrap_or(false);
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
            // Client-streaming (FIDIUS-I-0030): a `Stream<T>` in argument position.
            let client_stream_item = arg_types
                .iter()
                .find_map(|t| crate::wit::stream_item_type(t));
            let wire_raw = impl_method_is_raw(&method.attrs)?;
            impl_methods.push(MethodInfo {
                name: &method.sig.ident,
                is_async: method.sig.asyncness.is_some(),
                returns_result,
                arg_types,
                arg_names,
                wire_raw,
                ret_type,
                stream_item,
                client_stream_item,
            });
        }
    }

    let method_names: Vec<&Ident> = impl_methods.iter().map(|m| m.name).collect();
    let _has_async = impl_methods.iter().any(|m| m.is_async);

    let crate_path = &attrs.crate_path;
    let buffer_strategy = attrs.buffer_strategy;

    // Server-streaming methods (`-> fidius::Stream<T>`, FIDIUS-I-0026) require the
    // PluginAllocated buffer strategy on the cdylib path (the iterator-handle ABI
    // and per-item Box<[u8]> allocation are PluginAllocated-shaped; Arena
    // streaming is out of scope). Fail at macro time — this is a structural
    // misconfiguration, not target-dependent.
    let any_streaming = impl_methods
        .iter()
        .any(|m| m.stream_item.is_some() || m.client_stream_item.is_some());
    if any_streaming && matches!(buffer_strategy, BufferStrategyAttr::Arena) {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "server-streaming methods (`-> fidius::Stream<T>`) require the PluginAllocated \
             buffer strategy; the Arena strategy does not support streaming.",
        ));
    }

    // Strip `#[wire(...)]` helper attrs from the re-emitted impl block so the
    // Rust compiler doesn't reject them as unknown attributes.
    let mut item_emit = item.clone();
    for impl_item in &mut item_emit.items {
        if let ImplItem::Fn(method) = impl_item {
            method.attrs.retain(|a| !a.path().is_ident("wire"));
        }
    }

    // Static singleton — now used ONLY by the wasm adapter (FIDIUS-A-0006: the
    // cdylib path constructs instances via the descriptor's `construct`).
    let instance_name = format_ident!("__FIDIUS_INSTANCE_{}", impl_ident);
    // FIDIUS-A-0006 / CI.3: a configured wasm plugin holds its instance in a
    // `OnceLock` set by the `fidius-configure` export (bound once); zero-config
    // keeps the unit singleton.
    let instance = if attrs.config.is_some() {
        quote! {
            #[cfg(target_family = "wasm")]
            static #instance_name: ::std::sync::OnceLock<#impl_type> = ::std::sync::OnceLock::new();
        }
    } else {
        quote! {
            #[cfg(target_family = "wasm")]
            static #instance_name: #impl_type = #impl_type;
        }
    };

    // WASM Component Model auto-export (FIDIUS-T-0106). Emitted under
    // `#[cfg(target_family = "wasm")]`; handles both unary and (FIDIUS-I-0026)
    // server-streaming methods. cdylib/Python builds cfg this out entirely.
    let wasm_adapter = generate_wasm_adapter(
        trait_name,
        &instance_name,
        &impl_methods,
        attrs.config.as_ref(),
        crate_path,
        impl_type,
    );

    // The cdylib FFI machinery — shims (unary or, for streaming methods, the
    // iterator-handle init/next/drop), vtable, descriptor, registration. All
    // gated `#[cfg(not(target_family = "wasm"))]` inside; a wasm build uses the
    // adapter above instead.
    let cdylib = {
        let shims = generate_shims(&impl_ident, &impl_methods, crate_path, buffer_strategy);
        let free_fn_name = format_ident!("__fidius_free_buffer_{}", impl_ident);
        let free_buffer = match buffer_strategy {
            BufferStrategyAttr::PluginAllocated => quote! {
                #[cfg(not(target_family = "wasm"))]
                unsafe extern "C" fn #free_fn_name(ptr: *mut u8, len: usize) {
                    if !ptr.is_null() && len > 0 {
                        // Reconstruct the Box<[u8]> from its raw parts. Safe because the
                        // shim emitted by generate_shims always allocates output as a
                        // Box<[u8]> (cap == len by construction — no mismatch possible).
                        // Streaming item buffers use the same contract.
                        unsafe {
                            let slice = std::slice::from_raw_parts_mut(ptr, len);
                            drop(Box::from_raw(slice as *mut [u8]));
                        }
                    }
                }
            },
            BufferStrategyAttr::Arena => quote! {},
        };
        let vtable = generate_vtable_static(trait_name, &impl_ident, &method_names);
        let descriptor = generate_descriptor(
            trait_name,
            &impl_ident,
            &method_names,
            crate_path,
            buffer_strategy,
            attrs.config.as_ref(),
        );
        let registration = generate_inventory_registration(&impl_ident, crate_path);
        quote! {
            #shims
            #free_buffer
            #vtable
            #descriptor
            #registration
        }
    };

    Ok(quote! {
        #item_emit
        #instance
        #cdylib
        #wasm_adapter
    })
}

/// Generate the WASM component auto-export adapter for `#[plugin_impl]`.
///
/// Emits, under `#[cfg(target_family = "wasm")]`, a `wit_bindgen::generate!`
/// invocation (inline WIT generated from the method signatures) plus a `Guest`
/// impl that forwards to the plugin's static instance, and `export!`. If any
/// method has a reference argument or a type outside the supported set, it
/// instead emits a wasm-gated `compile_error!` (a clear failure on wasm builds;
/// a no-op on cdylib/Python builds where the whole adapter is cfg'd out).
fn generate_wasm_adapter(
    trait_name: &Ident,
    instance_name: &Ident,
    methods: &[MethodInfo],
    config: Option<&Path>,
    crate_path: &Path,
    impl_type: &Type,
) -> TokenStream {
    use crate::wit::{
        conv_expr, render_wit, result_ok_type, return_to_wit, return_to_wit_with, rust_type_to_wit,
        to_kebab_case, wit_type_with, WitMethod,
    };
    use std::collections::BTreeSet;

    let iface_kebab = to_kebab_case(&trait_name.to_string());
    let iface_snake = iface_kebab.replace('-', "_");
    let pkg_seg = format_ident!("{}", iface_snake);
    let world = format!("{iface_kebab}-plugin");

    // FIDIUS-A-0006 / CI.3: the WIT always declares `fidius-configure`, so the
    // Guest always implements it — a no-op for a zero-config plugin, or
    // deserialize-and-set the OnceLock instance for a configured one. Methods
    // dispatch on the static singleton (zero-config) or the configured instance.
    let dispatch_self = if config.is_some() {
        quote! { super::#instance_name.get().expect("fidius: plugin method called before configure()") }
    } else {
        quote! { super::#instance_name }
    };
    let configure_item = if let Some(cfg_ty) = config {
        quote! {
            fn fidius_configure(config: ::std::vec::Vec<u8>) {
                let cfg: #cfg_ty = #crate_path::wire::deserialize(&config)
                    .expect("fidius: configure failed to deserialize config");
                let _ = super::#instance_name.set(<#impl_type>::configure(cfg));
            }
        }
    } else {
        quote! { fn fidius_configure(_config: ::std::vec::Vec<u8>) {} }
    };
    // The interface-hash const lives in the `#[plugin_interface]` companion
    // module (`__fidius_<Trait>`), a sibling of the impl — reference it there.
    let companion = format_ident!("__fidius_{}", trait_name);
    let hash_const = format_ident!("{}_INTERFACE_HASH", trait_name);
    let module_ident = format_ident!("__fidius_wasm_{}", instance_name);

    // Collect candidate user types (non-primitive path idents in signatures;
    // `#[derive(WitType)]` records/variants). Reject reference args (owned only,
    // v1). Then validate every type maps to WIT — a structurally-unsupported
    // type emits a wasm-gated compile_error rather than a silently-broken export.
    let mut known: BTreeSet<String> = BTreeSet::new();
    for m in methods {
        for ty in &m.arg_types {
            if matches!(ty, Type::Reference(_)) {
                return wasm_unsupported(
                    m.name,
                    "reference arguments are not supported — take owned types (String, Vec<u8>, …)",
                );
            }
            // Client-streaming: the `Stream<T>` arg isn't a WIT type — classify its
            // ITEM type `T` (CS2.3), like a server-streaming return.
            if let Some(item) = crate::wit::stream_item_type(ty) {
                if m.client_stream_item.is_some() {
                    collect_user_idents(item, &mut known);
                    continue;
                }
            }
            collect_user_idents(ty, &mut known);
        }
        // For a streaming method, classify the stream *item* type, not the
        // `Stream<T>` wrapper (which isn't a WIT type). (FIDIUS-I-0026.)
        if let Some(item) = m.stream_item {
            collect_user_idents(item, &mut known);
        } else if m.returns_result {
            if let Some(ok) = m.ret_type.and_then(result_ok_type) {
                collect_user_idents(ok, &mut known);
            }
        } else if let Some(rt) = m.ret_type {
            collect_user_idents(rt, &mut known);
        }
    }
    for m in methods {
        for ty in &m.arg_types {
            // Client-streaming: validate the `Stream<T>` arg's item type, not the
            // (non-WIT) `Stream<T>` wrapper.
            if let Some(item) = crate::wit::stream_item_type(ty) {
                if m.client_stream_item.is_some() {
                    if let Err(e) = wit_type_with(item, &known) {
                        return wasm_unsupported(m.name, &e);
                    }
                    continue;
                }
            }
            if let Err(e) = wit_type_with(ty, &known) {
                return wasm_unsupported(m.name, &e);
            }
        }
        if let Some(item) = m.stream_item {
            if let Err(e) = wit_type_with(item, &known) {
                return wasm_unsupported(m.name, &e);
            }
        } else if let Err(e) = return_to_wit_with(m.ret_type, &known) {
            return wasm_unsupported(m.name, &e);
        }
    }

    let has_user = !known.is_empty();

    if !has_user {
        // ── Primitives-only: self-contained inline WIT (no build.rs needed). ──
        let mut wit_methods = Vec::new();
        for m in methods {
            let mut params = Vec::new();
            for (name, ty) in m.arg_names.iter().zip(&m.arg_types) {
                // Client-streaming (FIDIUS-I-0030 CS2.3): the `Stream<T>` argument
                // is pulled via the `fidius:stream-pull` import, not a WIT param.
                if m.client_stream_item.is_some() && crate::wit::stream_item_type(ty).is_some() {
                    continue;
                }
                let wt = rust_type_to_wit(ty).expect("validated above");
                params.push((to_kebab_case(&name.to_string()), wt));
            }
            let (ret, stream_item) = if let Some(item) = m.stream_item {
                (None, Some(rust_type_to_wit(item).expect("validated above")))
            } else {
                (return_to_wit(m.ret_type).expect("validated above"), None)
            };
            wit_methods.push(WitMethod {
                name: to_kebab_case(&m.name.to_string()),
                params,
                ret,
                stream_item,
            });
        }
        let wit_doc = render_wit(&iface_kebab, &wit_methods);

        // Items inside `impl Guest` (fns + streaming `type` decls), plus
        // module-level resource defs (struct + `Guest<M>Stream` impl) for each
        // streaming method.
        let mut guest_items: Vec<TokenStream> = Vec::new();
        let mut resource_defs: Vec<TokenStream> = Vec::new();
        for m in methods {
            let mname = m.name;
            let arg_names = &m.arg_names;
            let arg_types = &m.arg_types;

            if let (Some(out_item), Some(in_item)) = (m.stream_item, m.client_stream_item) {
                // Bidirectional (FIDIUS-I-0032 / ADR-0010): input via the
                // `fidius:stream-pull` import (CS2.3), output as an exported resource
                // (WS). The method takes the NON-stream args as WIT params, builds the
                // input `Stream<In>` from `WasmHostStream`, calls the user method to get
                // a lazy `Stream<Out>`, and returns it as the output resource. Pulling the
                // resource re-enters the import — the synchronous lazy-pull composition.
                // Checked BEFORE the server-only branch (bidi sets `stream_item` too).
                let res_pascal =
                    kebab_to_pascal(&format!("{}-stream", to_kebab_case(&mname.to_string())));
                let res_ident = format_ident!("{}", res_pascal);
                let guest_trait = format_ident!("Guest{}", res_pascal);
                let state_ident = format_ident!("__Fidius{}", res_pascal);
                let stream_ty = m
                    .ret_type
                    .expect("a bidirectional method returns Stream<Out>");
                let stream_idx = arg_types
                    .iter()
                    .position(|t| crate::wit::stream_item_type(t).is_some())
                    .expect("a bidirectional method has a `Stream<In>` argument");
                let stream_arg_name = &arg_names[stream_idx];
                let ns_names: Vec<&Ident> = arg_names
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != stream_idx)
                    .map(|(_, n)| n)
                    .collect();
                let ns_types: Vec<&Type> = arg_types
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != stream_idx)
                    .map(|(_, t)| *t)
                    .collect();
                resource_defs.push(quote! {
                    struct #state_ident {
                        stream: ::core::cell::RefCell<#stream_ty>,
                    }
                    impl exports::fidius::#pkg_seg::#pkg_seg::#guest_trait for #state_ident {
                        fn next(&self) -> ::core::result::Result<
                            ::core::option::Option<#out_item>,
                            exports::fidius::#pkg_seg::#pkg_seg::PluginError,
                        > {
                            ::core::result::Result::Ok(self.stream.borrow_mut().next_item())
                        }
                    }
                });
                guest_items.push(quote! {
                    type #res_ident = #state_ident;
                    fn #mname(#(#ns_names: #ns_types),*)
                        -> exports::fidius::#pkg_seg::#pkg_seg::#res_ident {
                        // Input `Stream` from the host's `fidius:stream-pull` import.
                        let #stream_arg_name = #crate_path::stream_marker::Stream::from_iter(
                            #crate_path::client_stream::WasmHostStream::<#in_item>::new()
                        );
                        let __s = #dispatch_self.#mname(#(#arg_names),*);
                        exports::fidius::#pkg_seg::#pkg_seg::#res_ident::new(
                            #state_ident { stream: ::core::cell::RefCell::new(__s) }
                        )
                    }
                });
            } else if let Some(item) = m.stream_item {
                // Server-streaming → a wit-bindgen exported resource. Resource
                // `<m>-stream` → Pascal `<M>Stream`, trait `Guest<M>Stream`.
                let res_pascal =
                    kebab_to_pascal(&format!("{}-stream", to_kebab_case(&mname.to_string())));
                let res_ident = format_ident!("{}", res_pascal);
                let guest_trait = format_ident!("Guest{}", res_pascal);
                let state_ident = format_ident!("__Fidius{}", res_pascal);
                let stream_ty = m.ret_type.expect("streaming method has a return type");

                resource_defs.push(quote! {
                    struct #state_ident {
                        stream: ::core::cell::RefCell<#stream_ty>,
                    }
                    impl exports::fidius::#pkg_seg::#pkg_seg::#guest_trait for #state_ident {
                        fn next(&self) -> ::core::result::Result<
                            ::core::option::Option<#item>,
                            exports::fidius::#pkg_seg::#pkg_seg::PluginError,
                        > {
                            ::core::result::Result::Ok(self.stream.borrow_mut().next_item())
                        }
                    }
                });
                guest_items.push(quote! {
                    type #res_ident = #state_ident;
                    fn #mname(#(#arg_names: #arg_types),*)
                        -> exports::fidius::#pkg_seg::#pkg_seg::#res_ident {
                        let __s = #dispatch_self.#mname(#(#arg_names),*);
                        exports::fidius::#pkg_seg::#pkg_seg::#res_ident::new(
                            #state_ident { stream: ::core::cell::RefCell::new(__s) }
                        )
                    }
                });
            } else if let Some(cs_item) = m.client_stream_item {
                // Client-streaming (FIDIUS-I-0030 CS2.3): the guest method takes the
                // NON-stream args as WIT params; the `Stream<T>` arg is built from
                // the `fidius:stream-pull` import (`WasmHostStream`), then the user
                // method is called with all args in declaration order.
                let stream_idx = arg_types
                    .iter()
                    .position(|t| crate::wit::stream_item_type(t).is_some())
                    .expect("client-streaming method has a `Stream<T>` argument");
                let stream_arg_name = &arg_names[stream_idx];
                let ns_names: Vec<&Ident> = arg_names
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != stream_idx)
                    .map(|(_, n)| n)
                    .collect();
                let ns_types: Vec<&Type> = arg_types
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != stream_idx)
                    .map(|(_, t)| *t)
                    .collect();
                let build_stream = quote! {
                    let #stream_arg_name = #crate_path::stream_marker::Stream::from_iter(
                        #crate_path::client_stream::WasmHostStream::<#cs_item>::new()
                    );
                };
                let call = quote! { #dispatch_self.#mname(#(#arg_names),*) };
                if m.returns_result {
                    let ok = match m.ret_type.and_then(result_ok_type) {
                        Some(t) => quote! { #t },
                        None => quote! { () },
                    };
                    guest_items.push(quote! {
                        fn #mname(#(#ns_names: #ns_types),*)
                            -> Result<#ok, exports::fidius::#pkg_seg::#pkg_seg::PluginError> {
                            #build_stream
                            #call.map_err(|__e| exports::fidius::#pkg_seg::#pkg_seg::PluginError {
                                code: __e.code, message: __e.message, details: __e.details,
                            })
                        }
                    });
                } else {
                    match m.ret_type {
                        Some(rt) => guest_items.push(quote! {
                            fn #mname(#(#ns_names: #ns_types),*) -> #rt { #build_stream #call }
                        }),
                        None => guest_items.push(quote! {
                            fn #mname(#(#ns_names: #ns_types),*) { #build_stream #call }
                        }),
                    }
                }
            } else {
                let call = quote! { #dispatch_self.#mname(#(#arg_names),*) };
                if m.returns_result {
                    let ok = match m.ret_type.and_then(result_ok_type) {
                        Some(t) => quote! { #t },
                        None => quote! { () },
                    };
                    guest_items.push(quote! {
                        fn #mname(#(#arg_names: #arg_types),*)
                            -> Result<#ok, exports::fidius::#pkg_seg::#pkg_seg::PluginError> {
                            #call.map_err(|__e| exports::fidius::#pkg_seg::#pkg_seg::PluginError {
                                code: __e.code, message: __e.message, details: __e.details,
                            })
                        }
                    });
                } else {
                    match m.ret_type {
                        Some(rt) => guest_items.push(
                            quote! { fn #mname(#(#arg_names: #arg_types),*) -> #rt { #call } },
                        ),
                        None => guest_items
                            .push(quote! { fn #mname(#(#arg_names: #arg_types),*) { #call } }),
                    }
                }
            }
        }
        return quote! {
            #[cfg(target_family = "wasm")]
            #[allow(warnings, clippy::all)]
            mod #module_ident {
                use super::*;
                ::wit_bindgen::generate!({ inline: #wit_doc, world: #world });
                #(#resource_defs)*
                struct __FidiusComponent;
                impl exports::fidius::#pkg_seg::#pkg_seg::Guest for __FidiusComponent {
                    #(#guest_items)*
                    fn fidius_interface_hash() -> u64 { super::#companion::#hash_const }
                    #configure_item
                }
                export!(__FidiusComponent);
            }
        };
    }

    // ── User types present: consume the build.rs-generated wit/ + conversions. ──
    // Client-streaming with `#[derive(WitType)]` user types is a follow-on; the
    // primitives-only branch above wires it (CS2.3).
    if let Some(m) = methods.iter().find(|m| m.client_stream_item.is_some()) {
        return wasm_unsupported(
            m.name,
            "client-streaming alongside #[derive(WitType)] user types is not yet supported on \
             WASM; use a primitive/String stream item in a user-type-free interface",
        );
    }
    // The Guest uses wit-bindgen's generated types; we convert at the boundary
    // via the generated `From` impls (`conv_expr` is identity for primitive-only
    // fields and `.into()`/map for user types).
    let mut guest_methods: Vec<TokenStream> = Vec::new();
    let mut resource_defs: Vec<TokenStream> = Vec::new();
    for m in methods {
        let mname = m.name;
        let arg_sig: Vec<TokenStream> = m
            .arg_names
            .iter()
            .zip(&m.arg_types)
            .map(|(n, t)| {
                let gt = gen_type(t, &known, &pkg_seg);
                quote! { #n: #gt }
            })
            .collect();
        let call_args: Vec<syn::Expr> = m
            .arg_names
            .iter()
            .zip(&m.arg_types)
            .map(|(n, t)| {
                let s = conv_expr(&n.to_string(), t, &known);
                syn::parse_str::<syn::Expr>(&s).expect("conv expr parses")
            })
            .collect();

        // Server-streaming with user types (PC.2): a wit-bindgen exported resource
        // whose `next()` yields the *binding* item type, converting each user item
        // via the generated `From`/`conv_expr`. Mirrors the primitives-only branch.
        if let Some(item) = m.stream_item {
            let res_pascal =
                kebab_to_pascal(&format!("{}-stream", to_kebab_case(&mname.to_string())));
            let res_ident = format_ident!("{}", res_pascal);
            let guest_trait = format_ident!("Guest{}", res_pascal);
            let state_ident = format_ident!("__Fidius{}", res_pascal);
            let stream_ty = m.ret_type.expect("streaming method has a return type");
            let item_binding = gen_type(item, &known, &pkg_seg);
            let item_conv: syn::Expr =
                syn::parse_str(&conv_expr("__v", item, &known)).expect("conv expr parses");
            resource_defs.push(quote! {
                struct #state_ident {
                    stream: ::core::cell::RefCell<#stream_ty>,
                }
                impl exports::fidius::#pkg_seg::#pkg_seg::#guest_trait for #state_ident {
                    fn next(&self) -> ::core::result::Result<
                        ::core::option::Option<#item_binding>,
                        exports::fidius::#pkg_seg::#pkg_seg::PluginError,
                    > {
                        ::core::result::Result::Ok(
                            self.stream.borrow_mut().next_item().map(|__v| #item_conv),
                        )
                    }
                }
            });
            guest_methods.push(quote! {
                type #res_ident = #state_ident;
                fn #mname(#(#arg_sig),*)
                    -> exports::fidius::#pkg_seg::#pkg_seg::#res_ident {
                    let __s = #dispatch_self.#mname(#(#call_args),*);
                    exports::fidius::#pkg_seg::#pkg_seg::#res_ident::new(
                        #state_ident { stream: ::core::cell::RefCell::new(__s) },
                    )
                }
            });
            continue;
        }

        let call = quote! { #dispatch_self.#mname(#(#call_args),*) };
        if m.returns_result {
            let ok = m.ret_type.and_then(result_ok_type);
            let gen_ok = match ok {
                Some(t) => gen_type(t, &known, &pkg_seg),
                None => quote! { () },
            };
            let ok_map = match ok {
                Some(t) => {
                    let e: syn::Expr =
                        syn::parse_str(&conv_expr("__v", t, &known)).expect("conv expr");
                    quote! { .map(|__v| #e) }
                }
                None => quote! {},
            };
            guest_methods.push(quote! {
                fn #mname(#(#arg_sig),*)
                    -> Result<#gen_ok, exports::fidius::#pkg_seg::#pkg_seg::PluginError> {
                    #call #ok_map .map_err(|__e| exports::fidius::#pkg_seg::#pkg_seg::PluginError {
                        code: __e.code, message: __e.message, details: __e.details,
                    })
                }
            });
        } else {
            match m.ret_type {
                Some(rt) => {
                    let gen_ret = gen_type(rt, &known, &pkg_seg);
                    let e: syn::Expr =
                        syn::parse_str(&conv_expr("__r", rt, &known)).expect("conv expr");
                    guest_methods.push(
                        quote! { fn #mname(#(#arg_sig),*) -> #gen_ret { let __r = #call; #e } },
                    );
                }
                None => guest_methods.push(quote! { fn #mname(#(#arg_sig),*) { #call } }),
            }
        }
    }

    quote! {
        #[cfg(target_family = "wasm")]
        #[allow(warnings, clippy::all)]
        mod #module_ident {
            use super::*;
            // wit/ + the conversions are (re)generated from source by the crate's
            // build.rs (`fidius_build::emit_wit()`), since a proc-macro can't see
            // external type definitions.
            ::wit_bindgen::generate!({ path: "wit", world: #world });
            include!(concat!(env!("OUT_DIR"), "/fidius_wit_conversions.rs"));
            #(#resource_defs)*
            struct __FidiusComponent;
            impl exports::fidius::#pkg_seg::#pkg_seg::Guest for __FidiusComponent {
                #(#guest_methods)*
                fn fidius_interface_hash() -> u64 { super::#companion::#hash_const }
                #configure_item
            }
            export!(__FidiusComponent);
        }
    }
}

/// Collect candidate user-type idents (non-primitive path leaves) from a type,
/// descending through `Vec`/`Option`/`Box` and `Result`'s ok type.
fn collect_user_idents(ty: &Type, out: &mut std::collections::BTreeSet<String>) {
    match ty {
        Type::Reference(r) => collect_user_idents(&r.elem, out),
        Type::Slice(s) => collect_user_idents(&s.elem, out),
        Type::Path(p) => {
            if let Some(seg) = p.path.segments.last() {
                let id = seg.ident.to_string();
                let prim = matches!(
                    id.as_str(),
                    "bool"
                        | "i8"
                        | "i16"
                        | "i32"
                        | "i64"
                        | "u8"
                        | "u16"
                        | "u32"
                        | "u64"
                        | "f32"
                        | "f64"
                        | "char"
                        | "String"
                        | "str"
                        | "PluginError"
                );
                match id.as_str() {
                    "Vec" | "Option" | "Box" => {
                        if let Some(inner) = wasm_first_generic(seg) {
                            collect_user_idents(inner, out);
                        }
                    }
                    "Result" => {
                        if let Some(ok) = wasm_first_generic(seg) {
                            collect_user_idents(ok, out);
                        }
                    }
                    _ if !prim => {
                        out.insert(id);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

/// The wit-bindgen-generated type for an author type: identity for types holding
/// no user type (wit-bindgen uses the same Rust type), else the generated path
/// `exports::fidius::<iface>::<iface>::<T>`, recursing through `Vec`/`Option`.
fn gen_type(ty: &Type, known: &std::collections::BTreeSet<String>, pkg_seg: &Ident) -> TokenStream {
    // Maps bind as `Vec<(K, V)>` — the wit-bindgen lowering of `list<tuple<k, v>>`.
    // Handle before the user-type short-circuit (a `HashMap<String, u32>` has no
    // `#[derive(WitType)]` inside but still needs its binding type).
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            if matches!(seg.ident.to_string().as_str(), "HashMap" | "BTreeMap") {
                if let Some((k, v)) = wasm_two_generics(seg) {
                    let gk = gen_type(k, known, pkg_seg);
                    let gv = gen_type(v, known, pkg_seg);
                    return quote! { ::std::vec::Vec<(#gk, #gv)> };
                }
            }
        }
    }
    if !crate::wit::contains_user_type(ty, known) {
        return quote! { #ty };
    }
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            let id = seg.ident.to_string();
            if id == "Vec" {
                if let Some(inner) = wasm_first_generic(seg) {
                    let g = gen_type(inner, known, pkg_seg);
                    return quote! { ::std::vec::Vec<#g> };
                }
            }
            if id == "Option" {
                if let Some(inner) = wasm_first_generic(seg) {
                    let g = gen_type(inner, known, pkg_seg);
                    return quote! { ::core::option::Option<#g> };
                }
            }
            if known.contains(&id) {
                let tid = &seg.ident;
                return quote! { exports::fidius::#pkg_seg::#pkg_seg::#tid };
            }
        }
    }
    quote! { #ty }
}

fn wasm_first_generic(seg: &syn::PathSegment) -> Option<&Type> {
    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
        for a in &ab.args {
            if let syn::GenericArgument::Type(t) = a {
                return Some(t);
            }
        }
    }
    None
}

fn wasm_two_generics(seg: &syn::PathSegment) -> Option<(&Type, &Type)> {
    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
        let types: Vec<&Type> = ab
            .args
            .iter()
            .filter_map(|a| match a {
                syn::GenericArgument::Type(t) => Some(t),
                _ => None,
            })
            .collect();
        if types.len() >= 2 {
            return Some((types[0], types[1]));
        }
    }
    None
}

/// Emit a `#[cfg(target_family = "wasm")]`-gated `compile_error!` for a method
/// the WASM auto-export can't handle. On native (cdylib/Python) builds the cfg
/// is false, so this is a no-op there; on a wasm build it fails the compile with
/// a clear message instead of silently producing a component that exports nothing.
fn wasm_unsupported(method: &Ident, reason: &str) -> TokenStream {
    let msg = format!(
        "fidius WASM auto-export: method `{method}` cannot be exported to a component — {reason}. \
         Supported types: bool, i8..i64, u8..u64, f32/f64, char, String, Vec<T>, Option<T>, \
         and Result<T, PluginError>. (User struct/enum support via #[derive(WitType)] is planned.)"
    );
    quote! {
        #[cfg(target_family = "wasm")]
        ::core::compile_error!(#msg);
    }
}

/// Generate extern "C" shim functions for each method. Shim signatures and
/// bodies vary by buffer strategy — see the two emit paths below.
fn generate_shims(
    impl_ident: &Ident,
    methods: &[MethodInfo],
    crate_path: &Path,
    buffer_strategy: BufferStrategyAttr,
) -> TokenStream {
    let shim_fns: Vec<TokenStream> = methods
        .iter()
        .map(|method| {
            let method_name = method.name;
            let shim_name = format_ident!("__fidius_shim_{}_{}", impl_ident, method_name);

            let arg_types = &method.arg_types;
            let arg_names = &method.arg_names;

            // Raw mode: skip bincode on the input. The single arg is a
            // Vec<u8> built directly from the FFI buffer (one alloc + memcpy).
            // Typed mode: deserialize input as a tuple of all argument types.
            let deserialize_args = if method.wire_raw {
                let arg_name = method
                    .arg_names
                    .first()
                    .cloned()
                    .unwrap_or_else(|| format_ident!("_arg"));
                quote! {
                    let #arg_name: ::std::vec::Vec<u8> = in_slice.to_vec();
                }
            } else {
                quote! {
                    let (#(#arg_names,)*) = match #crate_path::wire::deserialize::<(#(#arg_types,)*)>(in_slice) {
                        Ok(v) => v,
                        Err(_) => return #crate_path::status::STATUS_SERIALIZATION_ERROR,
                    };
                }
            };

            // Bidirectional (FIDIUS-I-0032 / ADR-0010): `Stream<In>` argument +
            // `Stream<Out>` return. The vtable slot is a `ClientStreamFn` (it takes the
            // host's input producer handle); the shim builds the input `Stream` from that
            // handle (client-streaming, CS2.2), calls the method to get a lazy
            // `Stream<Out>`, and returns an OUTPUT stream handle (server-streaming, CS.1).
            // The host drives `output.next()`, which re-enters `input.next()` — the
            // synchronous lazy-pull composition. Checked BEFORE the server-only branch
            // (a bidi method has `stream_item` set too).
            if let (Some(out_item), Some(in_item)) =
                (method.stream_item, method.client_stream_item)
            {
                let out_stream_ty = method
                    .ret_type
                    .expect("a bidirectional method returns `-> fidius::Stream<Out>`");
                let stream_idx = method
                    .arg_types
                    .iter()
                    .position(|t| crate::wit::stream_item_type(t).is_some())
                    .expect("a bidirectional method has a `Stream<In>` argument");
                let stream_arg_name = &method.arg_names[stream_idx];
                let non_stream_names: Vec<&Ident> = method
                    .arg_names
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != stream_idx)
                    .map(|(_, n)| n)
                    .collect();
                let non_stream_types: Vec<&Type> = method
                    .arg_types
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != stream_idx)
                    .map(|(_, t)| *t)
                    .collect();
                let next_name = format_ident!("__fidius_bnext_{}_{}", impl_ident, method_name);
                let drop_name = format_ident!("__fidius_bdrop_{}_{}", impl_ident, method_name);
                // The OUTPUT stream's per-item state (identical to server-streaming).
                let state_ty = quote! { #crate_path::stream_ffi::StreamState<#out_item> };
                return quote! {
                    // Output-stream `next` (arena-style, server-streaming shape).
                    #[cfg(not(target_family = "wasm"))]
                    unsafe extern "C" fn #next_name(
                        handle: *mut #crate_path::stream_ffi::FidiusStreamHandle,
                        buf_ptr: *mut u8,
                        buf_cap: u32,
                        out_len: *mut u32,
                    ) -> i32 {
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let __state = unsafe { &mut *((*handle).state as *mut #state_ty) };
                            let __buf = unsafe {
                                ::core::slice::from_raw_parts_mut(buf_ptr, buf_cap as usize)
                            };
                            match __state.next_into(__buf) {
                                #crate_path::stream_ffi::NextStatus::Item(__n) => {
                                    unsafe { *out_len = __n as u32; }
                                    #crate_path::status::STATUS_OK
                                }
                                #crate_path::stream_ffi::NextStatus::End => {
                                    #crate_path::status::STATUS_STREAM_END
                                }
                                #crate_path::stream_ffi::NextStatus::TooSmall(__need) => {
                                    unsafe { *out_len = __need as u32; }
                                    #crate_path::status::STATUS_BUFFER_TOO_SMALL
                                }
                                #crate_path::stream_ffi::NextStatus::SerErr => {
                                    #crate_path::status::STATUS_SERIALIZATION_ERROR
                                }
                            }
                        }));
                        match result {
                            ::core::result::Result::Ok(s) => s,
                            ::core::result::Result::Err(_) => #crate_path::status::STATUS_PANIC,
                        }
                    }

                    #[cfg(not(target_family = "wasm"))]
                    unsafe extern "C" fn #drop_name(
                        handle: *mut #crate_path::stream_ffi::FidiusStreamHandle,
                    ) {
                        if handle.is_null() { return; }
                        unsafe {
                            let __h = Box::from_raw(handle);
                            drop(Box::from_raw(__h.state as *mut #state_ty));
                        }
                    }

                    // Init shim: `ClientStreamFn` shape — takes the input producer handle,
                    // returns the output stream handle.
                    #[cfg(not(target_family = "wasm"))]
                    unsafe extern "C" fn #shim_name(
                        instance: *mut ::core::ffi::c_void,
                        handle: *mut #crate_path::stream_ffi::FidiusStreamHandle,
                        in_ptr: *const u8,
                        in_len: u32,
                        out_ptr: *mut *mut u8,
                        out_len: *mut u32,
                    ) -> i32 {
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let in_slice = unsafe { std::slice::from_raw_parts(in_ptr, in_len as usize) };
                            let (#(#non_stream_names,)*) = match #crate_path::wire::deserialize::<(#(#non_stream_types,)*)>(in_slice) {
                                Ok(v) => v,
                                Err(_) => return #crate_path::status::STATUS_SERIALIZATION_ERROR,
                            };
                            // Input `Stream` from the host producer handle (CS2.2).
                            let #stream_arg_name = #crate_path::stream_marker::Stream::from_iter(
                                unsafe { #crate_path::stream_ffi::HostStream::<#in_item>::from_handle(handle) }
                            );
                            // Lazy `Stream<Out>` — pulls input on demand when driven.
                            let __stream: #out_stream_ty = (unsafe { &*(instance as *const #impl_ident) }).#method_name(#(#arg_names),*);
                            let __state: Box<#state_ty> = Box::new(
                                #crate_path::stream_ffi::StreamState::new(__stream),
                            );
                            let __state = Box::into_raw(__state) as *mut ::core::ffi::c_void;
                            let __handle = Box::into_raw(Box::new(
                                #crate_path::stream_ffi::FidiusStreamHandle {
                                    next: #next_name,
                                    drop_fn: #drop_name,
                                    state: __state,
                                },
                            ));
                            unsafe { *out_ptr = __handle as *mut u8; *out_len = 0; }
                            #crate_path::status::STATUS_OK
                        }));
                        match result {
                            ::core::result::Result::Ok(s) => s,
                            ::core::result::Result::Err(_) => #crate_path::status::STATUS_PANIC,
                        }
                    }
                };
            }

            // Server-streaming (FIDIUS-I-0026 CS.1): the vtable slot holds an
            // `init` shim that returns a `FidiusStreamHandle` (via the standard
            // out_ptr); generated `next`/`drop_fn` drive and free it. Items cross
            // as self-describing JSON(`Value`). Requires PluginAllocated (enforced
            // in `generate_plugin_impl`).
            if let Some(item_ty) = method.stream_item {
                let stream_ty = method
                    .ret_type
                    .expect("a streaming method has a `-> fidius::Stream<T>` return type");
                let next_name = format_ident!("__fidius_snext_{}_{}", impl_ident, method_name);
                let drop_name = format_ident!("__fidius_sdrop_{}_{}", impl_ident, method_name);
                // Per-stream state = `StreamState<ItemTy>` (producer + the bincode
                // of the current item, retained across BUFFER_TOO_SMALL retries).
                let state_ty = quote! { #crate_path::stream_ffi::StreamState<#item_ty> };
                return quote! {
                    // Arena-style next (FIDIUS-T-0138): the host passes a reusable
                    // buffer; we write the bincode item into it. No per-item alloc,
                    // no `free_buffer` crossing.
                    #[cfg(not(target_family = "wasm"))]
                    unsafe extern "C" fn #next_name(
                        handle: *mut #crate_path::stream_ffi::FidiusStreamHandle,
                        buf_ptr: *mut u8,
                        buf_cap: u32,
                        out_len: *mut u32,
                    ) -> i32 {
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let __state = unsafe { &mut *((*handle).state as *mut #state_ty) };
                            let __buf = unsafe {
                                ::core::slice::from_raw_parts_mut(buf_ptr, buf_cap as usize)
                            };
                            match __state.next_into(__buf) {
                                #crate_path::stream_ffi::NextStatus::Item(__n) => {
                                    unsafe { *out_len = __n as u32; }
                                    #crate_path::status::STATUS_OK
                                }
                                #crate_path::stream_ffi::NextStatus::End => {
                                    #crate_path::status::STATUS_STREAM_END
                                }
                                #crate_path::stream_ffi::NextStatus::TooSmall(__need) => {
                                    unsafe { *out_len = __need as u32; }
                                    #crate_path::status::STATUS_BUFFER_TOO_SMALL
                                }
                                #crate_path::stream_ffi::NextStatus::SerErr => {
                                    #crate_path::status::STATUS_SERIALIZATION_ERROR
                                }
                            }
                        }));
                        match result {
                            ::core::result::Result::Ok(s) => s,
                            ::core::result::Result::Err(_) => #crate_path::status::STATUS_PANIC,
                        }
                    }

                    #[cfg(not(target_family = "wasm"))]
                    unsafe extern "C" fn #drop_name(
                        handle: *mut #crate_path::stream_ffi::FidiusStreamHandle,
                    ) {
                        if handle.is_null() { return; }
                        // Reclaim the handle box + the producer it owns (guest
                        // frees its own allocations — never the host).
                        unsafe {
                            let __h = Box::from_raw(handle);
                            drop(Box::from_raw(__h.state as *mut #state_ty));
                        }
                    }

                    #[cfg(not(target_family = "wasm"))]
                    unsafe extern "C" fn #shim_name(
                        instance: *mut ::core::ffi::c_void,
                        in_ptr: *const u8,
                        in_len: u32,
                        out_ptr: *mut *mut u8,
                        out_len: *mut u32,
                    ) -> i32 {
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let in_slice = unsafe { std::slice::from_raw_parts(in_ptr, in_len as usize) };
                            #deserialize_args
                            let __stream: #stream_ty = (unsafe { &*(instance as *const #impl_ident) }).#method_name(#(#arg_names),*);
                            let __state: Box<#state_ty> = Box::new(
                                #crate_path::stream_ffi::StreamState::new(__stream),
                            );
                            let __state = Box::into_raw(__state) as *mut ::core::ffi::c_void;
                            let __handle = Box::into_raw(Box::new(
                                #crate_path::stream_ffi::FidiusStreamHandle {
                                    next: #next_name,
                                    drop_fn: #drop_name,
                                    state: __state,
                                },
                            ));
                            unsafe { *out_ptr = __handle as *mut u8; *out_len = 0; }
                            #crate_path::status::STATUS_OK
                        }));
                        match result {
                            ::core::result::Result::Ok(s) => s,
                            ::core::result::Result::Err(_) => #crate_path::status::STATUS_PANIC,
                        }
                    }
                };
            }

            // The method call — dispatches on the instance pointer the host
            // passed (FIDIUS-A-0006), not a static singleton. Either sync or
            // async via block_on.
            let method_call = if method.is_async {
                quote! {
                    #crate_path::async_runtime::FIDIUS_RUNTIME.block_on(
                        (unsafe { &*(instance as *const #impl_ident) }).#method_name(#(#arg_names),*)
                    )
                }
            } else {
                quote! { (unsafe { &*(instance as *const #impl_ident) }).#method_name(#(#arg_names),*) }
            };

            // Generate the output handling based on whether the method returns Result.
            // Raw mode bypasses bincode on the success payload (the Vec<u8> *is*
            // the wire payload). Error path still bincode-encodes for typed errors.
            let output_handling = if method.wire_raw {
                if method.returns_result {
                    quote! {
                        match output {
                            Ok(val) => (val, #crate_path::status::STATUS_OK),
                            Err(err) => {
                                match #crate_path::wire::serialize(&err) {
                                    Ok(v) => (v, #crate_path::status::STATUS_PLUGIN_ERROR),
                                    Err(_) => return #crate_path::status::STATUS_SERIALIZATION_ERROR,
                                }
                            }
                        }
                    }
                } else {
                    quote! { (output, #crate_path::status::STATUS_OK) }
                }
            } else if method.returns_result {
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

            // Client-streaming (FIDIUS-I-0030 CS2.2): the vtable slot is a
            // `ClientStreamFn` — it also takes the host's producer handle, from
            // which we build the `Stream<T>` the method consumes. The non-stream
            // args still cross as a bincode tuple; the result returns via the
            // PluginAllocated out-buffer (Arena is rejected in generate_plugin_impl).
            if let Some(item_ty) = method.client_stream_item {
                let stream_idx = method
                    .arg_types
                    .iter()
                    .position(|t| crate::wit::stream_item_type(t).is_some())
                    .expect("client-streaming method has a `Stream<T>` argument");
                let stream_arg_name = &method.arg_names[stream_idx];
                let non_stream_names: Vec<&Ident> = method
                    .arg_names
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != stream_idx)
                    .map(|(_, n)| n)
                    .collect();
                let non_stream_types: Vec<&Type> = method
                    .arg_types
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != stream_idx)
                    .map(|(_, t)| *t)
                    .collect();
                return quote! {
                    #[cfg(not(target_family = "wasm"))]
                    unsafe extern "C" fn #shim_name(
                        instance: *mut ::core::ffi::c_void,
                        handle: *mut #crate_path::stream_ffi::FidiusStreamHandle,
                        in_ptr: *const u8,
                        in_len: u32,
                        out_ptr: *mut *mut u8,
                        out_len: *mut u32,
                    ) -> i32 {
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let in_slice = unsafe { std::slice::from_raw_parts(in_ptr, in_len as usize) };
                            let (#(#non_stream_names,)*) = match #crate_path::wire::deserialize::<(#(#non_stream_types,)*)>(in_slice) {
                                Ok(v) => v,
                                Err(_) => return #crate_path::status::STATUS_SERIALIZATION_ERROR,
                            };
                            // Build the `Stream<T>` from the host producer handle.
                            let #stream_arg_name = #crate_path::stream_marker::Stream::from_iter(
                                unsafe { #crate_path::stream_ffi::HostStream::<#item_ty>::from_handle(handle) }
                            );
                            let output = #method_call;
                            let (output_bytes, status) = #output_handling;
                            let boxed: Box<[u8]> = output_bytes.into_boxed_slice();
                            let len = boxed.len();
                            let ptr = Box::into_raw(boxed) as *mut u8;
                            unsafe { *out_ptr = ptr; *out_len = len as u32; }
                            status
                        }));
                        match result {
                            ::core::result::Result::Ok(status) => status,
                            ::core::result::Result::Err(_) => #crate_path::status::STATUS_PANIC,
                        }
                    }
                };
            }

            match buffer_strategy {
                BufferStrategyAttr::Arena => quote! {
                    #[cfg(not(target_family = "wasm"))]
                    unsafe extern "C" fn #shim_name(
                        instance: *mut ::core::ffi::c_void,
                        in_ptr: *const u8,
                        in_len: u32,
                        arena_ptr: *mut u8,
                        arena_cap: u32,
                        out_offset: *mut u32,
                        out_len: *mut u32,
                    ) -> i32 {
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let in_slice = unsafe { std::slice::from_raw_parts(in_ptr, in_len as usize) };
                            #deserialize_args

                            let output = #method_call;

                            let (output_bytes, status) = #output_handling;

                            // Arena strategy: write into host-provided buffer.
                            // If too small, return BUFFER_TOO_SMALL with needed size.
                            if output_bytes.len() > arena_cap as usize {
                                unsafe {
                                    *out_len = output_bytes.len() as u32;
                                }
                                return #crate_path::status::STATUS_BUFFER_TOO_SMALL;
                            }
                            let arena = unsafe {
                                ::std::slice::from_raw_parts_mut(arena_ptr, arena_cap as usize)
                            };
                            arena[..output_bytes.len()].copy_from_slice(&output_bytes);
                            unsafe {
                                *out_offset = 0;
                                *out_len = output_bytes.len() as u32;
                            }
                            status
                        }));

                        match result {
                            Ok(status) => status,
                            Err(_panic_payload) => {
                                // Arena strategy cannot reliably transmit panic
                                // messages (the arena may be too small and we
                                // can't re-request from here). Return STATUS_PANIC
                                // with out_len = 0; host reports an opaque panic.
                                unsafe {
                                    *out_offset = 0;
                                    *out_len = 0;
                                }
                                #crate_path::status::STATUS_PANIC
                            }
                        }
                    }
                },
                BufferStrategyAttr::PluginAllocated => quote! {
                #[cfg(not(target_family = "wasm"))]
                unsafe extern "C" fn #shim_name(
                    instance: *mut ::core::ffi::c_void,
                    in_ptr: *const u8,
                    in_len: u32,
                    out_ptr: *mut *mut u8,
                    out_len: *mut u32,
                ) -> i32 {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let in_slice = unsafe { std::slice::from_raw_parts(in_ptr, in_len as usize) };
                        #deserialize_args

                        let output = #method_call;

                        let (output_bytes, status) = #output_handling;

                        // Hand ownership to the host via Box<[u8]>. cap == len
                        // by construction, so free_buffer's reconstruction is
                        // always well-defined.
                        let boxed: Box<[u8]> = output_bytes.into_boxed_slice();
                        let len = boxed.len();
                        let ptr = Box::into_raw(boxed) as *mut u8;
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
                                let boxed: Box<[u8]> = msg_bytes.into_boxed_slice();
                                let len = boxed.len();
                                let ptr = Box::into_raw(boxed) as *mut u8;
                                unsafe {
                                    *out_ptr = ptr;
                                    *out_len = len as u32;
                                }
                            }
                            #crate_path::status::STATUS_PANIC
                        }
                    }
                }
                },
            }
        })
        .collect();

    // cdylib FFI shims are only meaningful for native dynamic loading. Each
    // emitted shim carries its own `#[cfg(not(target_family = "wasm"))]` (a
    // streaming method emits three fns — init/next/drop — so the gate can't live
    // on the group), so a wasm component build (which exports via the WIT
    // adapter, not these extern "C" shims) doesn't compile them.
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
        #[cfg(not(target_family = "wasm"))]
        static #vtable_name: #companion::#vtable_type = #companion::#constructor(#(#shim_args),*);
    }
}

/// Generate the PluginDescriptor static.
fn generate_descriptor(
    trait_name: &Ident,
    impl_ident: &Ident,
    methods: &[&Ident],
    crate_path: &Path,
    buffer_strategy: BufferStrategyAttr,
    config: Option<&Path>,
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
    let construct_name = format_ident!("__fidius_construct_{}", impl_ident);
    let destroy_name = format_ident!("__fidius_destroy_{}", impl_ident);

    let optional_methods_ident = format_ident!("{}_OPTIONAL_METHODS", trait_name);
    let method_strs: Vec<String> = methods.iter().map(|m| m.to_string()).collect();
    let method_count = methods.len() as u32;

    // free_buffer is only meaningful for PluginAllocated. Arena strategy
    // doesn't allocate output buffers — nothing to free.
    let free_buffer_expr = match buffer_strategy {
        BufferStrategyAttr::PluginAllocated => quote! { Some(#free_fn_name) },
        BufferStrategyAttr::Arena => quote! { None },
    };

    // FIDIUS-A-0006 construct body. With `config = C`: deserialize C from the
    // host-supplied bytes and call `Type::configure(cfg)` (null on a bad config).
    // Without: the zero-config unit instance (the singleton).
    let construct_body = match config {
        Some(cfg_ty) => quote! {
            let __slice = if cfg_ptr.is_null() || cfg_len == 0 {
                &[][..]
            } else {
                unsafe { ::core::slice::from_raw_parts(cfg_ptr, cfg_len as usize) }
            };
            let __cfg: #cfg_ty = match #crate_path::wire::deserialize(__slice) {
                ::core::result::Result::Ok(c) => c,
                ::core::result::Result::Err(_) => return ::core::ptr::null_mut(),
            };
            ::std::boxed::Box::into_raw(::std::boxed::Box::new(#impl_ident::configure(__cfg)))
                as *mut ::std::ffi::c_void
        },
        None => quote! {
            let _ = (cfg_ptr, cfg_len);
            ::std::boxed::Box::into_raw(::std::boxed::Box::new(#impl_ident)) as *mut ::std::ffi::c_void
        },
    };

    quote! {
        // FIDIUS-A-0006: construct/destroy a plugin instance. CI.1 builds the
        // zero-config (unit) instance and ignores the config bytes; typed
        // `config = C` deserialization is CI.2. The host passes the returned
        // pointer to every vtable method and frees it via destroy.
        #[cfg(not(target_family = "wasm"))]
        unsafe extern "C" fn #construct_name(
            cfg_ptr: *const u8,
            cfg_len: u32,
        ) -> *mut ::std::ffi::c_void {
            #construct_body
        }

        #[cfg(not(target_family = "wasm"))]
        unsafe extern "C" fn #destroy_name(instance: *mut ::std::ffi::c_void) {
            if instance.is_null() { return; }
            unsafe { drop(::std::boxed::Box::from_raw(instance as *mut #impl_ident)); }
        }

        #[cfg(not(target_family = "wasm"))]
        const #plugin_name_const: &std::ffi::CStr = unsafe {
            std::ffi::CStr::from_bytes_with_nul_unchecked(concat!(#impl_name_str, "\0").as_bytes())
        };

        #[cfg(not(target_family = "wasm"))]
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
                #free_buffer_expr,
                #method_count,
                Some(#construct_name),
                Some(#destroy_name),
            )
        };
    }
}

/// Register the descriptor via inventory for multi-plugin support.
fn generate_inventory_registration(impl_ident: &Ident, crate_path: &Path) -> TokenStream {
    let descriptor_name = format_ident!("__FIDIUS_DESCRIPTOR_{}", impl_ident);

    quote! {
        #[cfg(not(target_family = "wasm"))]
        #crate_path::inventory::submit! {
            #crate_path::registry::DescriptorEntry {
                descriptor: &#descriptor_name,
            }
        }
    }
}
