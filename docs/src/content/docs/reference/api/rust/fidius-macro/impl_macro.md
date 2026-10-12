---
title: "fidius-macro::impl_macro"
description: "Code generation for `#[plugin_impl(TraitName)]`."
editUrl: false
sidebar:
  order: 47
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Code generation for `#[plugin_impl(TraitName)]`.

Generates: the original impl, extern "C" FFI shims, a static instance,
a populated vtable static, a PluginDescriptor static, and for single-plugin
dylibs, the FIDIUS_PLUGIN_REGISTRY.

## Structs

### <span id="struct.PluginImplAttrs"></span><code style="font-size: 0.85em">struct PluginImplAttrs</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

Arguments to `#[plugin_impl(TraitName)]`, `#[plugin_impl(TraitName, crate = "...")]`, or `#[plugin_impl(TraitName, buffer = Arena)]`.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `trait_name` | `Ident` |  |
| `crate_path` | `Path` | The path to the fidius crate. Defaults to `fidius` when not specified. |
| `buffer_strategy` | `BufferStrategyAttr` | Must match the interface's `buffer` attribute. Defaults to `PluginAllocated`. Mismatches produce a vtable fn-pointer type error at compile time (the emitted shim's signature won't match the generated vtable's field type). |
| `config` | `Option<Path>` | FIDIUS-A-0006: the config type `C` for a configured plugin (`#[plugin_impl(Trait, config = C)]`). When set, `construct` deserializes `C` and calls the impl's `fn configure(cfg: C) -> Self`. None = zero-config (the singleton, constructed from unit). |

## Functions

### <span id="fn.generate_plugin_impl"></span><code style="font-size: 0.85em">fn generate_​plugin_​impl</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn generate_plugin_impl(
    attrs: &PluginImplAttrs,
    item: &ItemImpl,
) -> syn::Result<TokenStream>
```

Generate all code for a `#[plugin_impl(TraitName)]` invocation.

<details>
<summary>Source</summary>

```rust
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
```

</details>
