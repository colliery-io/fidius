# fidius-macro::impl_macro <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Code generation for `#[plugin_impl(TraitName)]`.

Generates: the original impl, extern "C" FFI shims, a static instance,
a populated vtable static, a PluginDescriptor static, and for single-plugin
dylibs, the FIDIUS_PLUGIN_REGISTRY.

## Structs

### `fidius-macro::impl_macro::MethodInfo`<'a>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


Info about an impl method, extracted from the impl block.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `name` | `& 'a Ident` |  |
| `is_async` | `bool` |  |
| `returns_result` | `bool` |  |
| `arg_types` | `Vec < & 'a Type >` | Argument types (excluding `self`). |
| `arg_names` | `Vec < Ident >` | Argument names (excluding `self`). |



### `fidius-macro::impl_macro::PluginImplAttrs`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Arguments to `#[plugin_impl(TraitName)]`, `#[plugin_impl(TraitName, crate = "...")]`, or `#[plugin_impl(TraitName, buffer = Arena)]`.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `trait_name` | `Ident` |  |
| `crate_path` | `Path` | The path to the fidius crate. Defaults to `fidius` when not specified. |
| `buffer_strategy` | `BufferStrategyAttr` | Must match the interface's `buffer` attribute. Defaults to
`PluginAllocated`. Mismatches produce a vtable fn-pointer type error
at compile time (the emitted shim's signature won't match the
generated vtable's field type). |



## Functions

### `fidius-macro::impl_macro::is_result_type`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn is_result_type (ty : & Type) -> bool
```

Check if a return type looks like `Result<T, ...>`.

<details>
<summary>Source</summary>

```rust
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
```

</details>



### `fidius-macro::impl_macro::generate_plugin_impl`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn generate_plugin_impl (attrs : & PluginImplAttrs , item : & ItemImpl) -> syn :: Result < TokenStream >
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
    let buffer_strategy = attrs.buffer_strategy;

    // Generate shim functions (signature and body vary by buffer strategy)
    let shims = generate_shims(&impl_ident, &impl_methods, crate_path, buffer_strategy);

    // Generate static instance
    let instance_name = format_ident!("__FIDIUS_INSTANCE_{}", impl_ident);
    let instance = quote! {
        static #instance_name: #impl_type = #impl_type;
    };

    // Generate vtable static
    let vtable = generate_vtable_static(trait_name, &impl_ident, &method_names);

    // free_buffer is only needed for PluginAllocated — Arena doesn't allocate
    // output, it writes into the host-provided arena, so nothing to free.
    let free_fn_name = format_ident!("__fidius_free_buffer_{}", impl_ident);
    let free_buffer = match buffer_strategy {
        BufferStrategyAttr::PluginAllocated => quote! {
            unsafe extern "C" fn #free_fn_name(ptr: *mut u8, len: usize) {
                if !ptr.is_null() && len > 0 {
                    // Reconstruct the Box<[u8]> from its raw parts. Safe because the
                    // shim emitted by generate_shims always allocates output as a
                    // Box<[u8]> (cap == len by construction — no mismatch possible).
                    unsafe {
                        let slice = std::slice::from_raw_parts_mut(ptr, len);
                        drop(Box::from_raw(slice as *mut [u8]));
                    }
                }
            }
        },
        BufferStrategyAttr::Arena => quote! {},
    };

    // Generate descriptor (free_buffer field is None for Arena)
    let descriptor = generate_descriptor(
        trait_name,
        &impl_ident,
        &method_names,
        crate_path,
        buffer_strategy,
    );

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
```

</details>



### `fidius-macro::impl_macro::generate_shims`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn generate_shims (impl_ident : & Ident , methods : & [MethodInfo] , crate_path : & Path , buffer_strategy : BufferStrategyAttr ,) -> TokenStream
```

Generate extern "C" shim functions for each method. Shim signatures and bodies vary by buffer strategy — see the two emit paths below.

<details>
<summary>Source</summary>

```rust
fn generate_shims(
    impl_ident: &Ident,
    methods: &[MethodInfo],
    crate_path: &Path,
    buffer_strategy: BufferStrategyAttr,
) -> TokenStream {
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

            match buffer_strategy {
                BufferStrategyAttr::Arena => quote! {
                    unsafe extern "C" fn #shim_name(
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

    quote! { #(#shim_fns)* }
}
```

</details>



### `fidius-macro::impl_macro::generate_vtable_static`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn generate_vtable_static (trait_name : & Ident , impl_ident : & Ident , methods : & [& Ident] ,) -> TokenStream
```

Generate the static vtable with function pointers.

Uses the `new_{trait}` constructor generated by `#[plugin_interface]`,
which knows which fields are optional (Option<fn>) vs required (fn).

<details>
<summary>Source</summary>

```rust
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
```

</details>



### `fidius-macro::impl_macro::generate_descriptor`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn generate_descriptor (trait_name : & Ident , impl_ident : & Ident , methods : & [& Ident] , crate_path : & Path , buffer_strategy : BufferStrategyAttr ,) -> TokenStream
```

Generate the PluginDescriptor static.

<details>
<summary>Source</summary>

```rust
fn generate_descriptor(
    trait_name: &Ident,
    impl_ident: &Ident,
    methods: &[&Ident],
    crate_path: &Path,
    buffer_strategy: BufferStrategyAttr,
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

    // free_buffer is only meaningful for PluginAllocated. Arena strategy
    // doesn't allocate output buffers — nothing to free.
    let free_buffer_expr = match buffer_strategy {
        BufferStrategyAttr::PluginAllocated => quote! { Some(#free_fn_name) },
        BufferStrategyAttr::Arena => quote! { None },
    };

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
                #free_buffer_expr,
                #method_count,
            )
        };
    }
}
```

</details>



### `fidius-macro::impl_macro::generate_inventory_registration`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn generate_inventory_registration (impl_ident : & Ident , crate_path : & Path) -> TokenStream
```

Register the descriptor via inventory for multi-plugin support.

<details>
<summary>Source</summary>

```rust
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
```

</details>



