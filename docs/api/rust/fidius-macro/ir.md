# fidius-macro::ir <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Intermediate representation for parsed plugin interface traits.

Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.

## Structs

### `fidius-macro::ir::InterfaceAttrs`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`, `Clone`

Parsed attributes from `#[plugin_interface(version = N, buffer = Strategy)]`.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `version` | `u32` |  |
| `buffer_strategy` | `BufferStrategyAttr` |  |
| `crate_path` | `Path` | The path to the fidius crate. Defaults to `fidius` when not specified.
Set via `crate = "my_crate::fidius"` in the attribute. |



### `fidius-macro::ir::MetaKvAttr`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`, `Clone`

A static metadata key/value pair parsed from a `#[method_meta(...)]` or `#[trait_meta(...)]` attribute. Both values are string literals.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `key` | `String` |  |
| `value` | `String` |  |



### `fidius-macro::ir::InterfaceIR`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`

Full IR for a parsed interface trait.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `trait_name` | `Ident` |  |
| `attrs` | `InterfaceAttrs` |  |
| `methods` | `Vec < MethodIR >` |  |
| `trait_metas` | `Vec < MetaKvAttr >` | Trait-level metadata from `#[trait_meta(...)]` attributes on the trait. |
| `original_trait` | `ItemTrait` | The original trait item, for re-emission. |



### `fidius-macro::ir::MethodIR`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`

IR for a single trait method.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `name` | `Ident` |  |
| `arg_types` | `Vec < Type >` | Argument types (excluding `self`). |
| `arg_names` | `Vec < Ident >` | Argument names (excluding `self`). |
| `return_type` | `Option < Type >` | Return type (the inner type, not `ReturnType`). |
| `is_async` | `bool` | Whether the method is `async fn`. |
| `optional_since` | `Option < u32 >` | If `#[optional(since = N)]`, the version it was added. |
| `signature_string` | `String` | Canonical signature string for interface hashing.
Format: `"name:arg_type_1,arg_type_2->return_type"`, with a trailing
`!raw` marker for methods opted into raw wire mode so the interface
hash diverges between raw and bincode-typed versions. |
| `method_metas` | `Vec < MetaKvAttr >` | Metadata from `#[method_meta("k", "v")]` attributes. Preserves declaration order. |
| `wire_raw` | `bool` | Whether this method is opted into raw (byte-passthrough) wire mode
via `#[wire(raw)]`. When true, the macro skips bincode on the
success path — the single `Vec<u8>` argument crosses the FFI
boundary as raw bytes, and the `Vec<u8>` return value is handed to
the host unchanged. Error-path payloads (for `Result<Vec<u8>, E>`
returns) continue to go through bincode. |
| `streaming` | `bool` | Whether this is a server-streaming method — its return type is
`fidius::Stream<T>` (FIDIUS-I-0026, D4). When true, [`Self::stream_item_type`]
holds the per-item type `T`, the signature string carries a `!stream`
marker (so it hashes distinctly from a unary `-> T`), and the host-side
client (ST.3) returns a `ChunkStream` instead of a `Result<T, _>`. |
| `stream_item_type` | `Option < Type >` | The per-item type `T` for a `streaming` method (the `T` in
`fidius::Stream<T>`). `None` for non-streaming methods. |

#### Methods

##### `is_required` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn is_required (& self) -> bool
```

Whether this is a required (non-optional) method.

<details>
<summary>Source</summary>

```rust
    pub fn is_required(&self) -> bool {
        self.optional_since.is_none()
    }
```

</details>





## Enums

### `fidius-macro::ir::BufferStrategyAttr` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Discriminants match `fidius_core::descriptor::BufferStrategyKind` — values `1` (PluginAllocated) and `2` (Arena). `0` is reserved for the removed `CallerAllocated` strategy.

#### Variants

- **`PluginAllocated`**
- **`Arena`**



## Functions

### `fidius-macro::ir::parse_meta_attrs`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn parse_meta_attrs (attrs : & [Attribute] , ident : & str) -> syn :: Result < Vec < MetaKvAttr > >
```

Parse all `#[method_meta("k", "v")]` or `#[trait_meta("k", "v")]` attributes with the given name from an attribute list into a `Vec<MetaKvAttr>`. Validates string-literal only, non-empty keys, no duplicate keys, and rejects keys in the reserved `fidius.*` namespace.

<details>
<summary>Source</summary>

```rust
fn parse_meta_attrs(attrs: &[Attribute], ident: &str) -> syn::Result<Vec<MetaKvAttr>> {
    let mut out = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident(ident) {
            continue;
        }
        // Parse two comma-separated string literals: #[attr("key", "value")]
        let (key_lit, value_lit): (LitStr, LitStr) =
            attr.parse_args_with(|input: syn::parse::ParseStream| {
                let k: LitStr = input.parse()?;
                let _: Token![,] = input.parse()?;
                let v: LitStr = input.parse()?;
                Ok((k, v))
            })?;
        let key = key_lit.value();
        let value = value_lit.value();

        if key.is_empty() {
            return Err(syn::Error::new(
                key_lit.span(),
                format!("#[{ident}(key, value)] key must not be empty"),
            ));
        }
        if key.trim() != key {
            return Err(syn::Error::new(
                key_lit.span(),
                format!("#[{ident}(key, value)] key must not have leading or trailing whitespace"),
            ));
        }
        if key.starts_with("fidius.") {
            return Err(syn::Error::new(
                key_lit.span(),
                format!("the `fidius.*` key namespace is reserved for framework use; got `{key}`"),
            ));
        }
        if out.iter().any(|existing: &MetaKvAttr| existing.key == key) {
            return Err(syn::Error::new(
                key_lit.span(),
                format!("duplicate #[{ident}] key `{key}`"),
            ));
        }
        out.push(MetaKvAttr { key, value });
    }
    Ok(out)
}
```

</details>



### `fidius-macro::ir::parse_optional_attr`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn parse_optional_attr (attrs : & [Attribute]) -> syn :: Result < Option < u32 > >
```

Parse an `#[optional(since = N)]` attribute, if present.

<details>
<summary>Source</summary>

```rust
fn parse_optional_attr(attrs: &[Attribute]) -> syn::Result<Option<u32>> {
    for attr in attrs {
        if attr.path().is_ident("optional") {
            let mut since = None;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("since") {
                    let _eq: Token![=] = meta.input.parse()?;
                    let lit: LitInt = meta.input.parse()?;
                    since = Some(lit.base10_parse::<u32>()?);
                    Ok(())
                } else {
                    Err(meta.error("expected `since`"))
                }
            })?;
            return Ok(since);
        }
    }
    Ok(None)
}
```

</details>



### `fidius-macro::ir::parse_wire_attr`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn parse_wire_attr (attrs : & [Attribute]) -> syn :: Result < bool >
```

Parse a `#[wire(raw)]` attribute, if present. Returns `true` when raw wire mode is opted in, `false` otherwise. Any other `wire(...)` form is a compile-time error.

<details>
<summary>Source</summary>

```rust
fn parse_wire_attr(attrs: &[Attribute]) -> syn::Result<bool> {
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
```

</details>



### `fidius-macro::ir::is_vec_u8`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn is_vec_u8 (ty : & Type) -> bool
```

Return `true` if the given type is `Vec<u8>`.

<details>
<summary>Source</summary>

```rust
fn is_vec_u8(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };
    let Some(last) = type_path.path.segments.last() else {
        return false;
    };
    if last.ident != "Vec" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return false;
    };
    if args.args.len() != 1 {
        return false;
    }
    let Some(syn::GenericArgument::Type(inner)) = args.args.first() else {
        return false;
    };
    let Type::Path(inner_path) = inner else {
        return false;
    };
    inner_path
        .path
        .get_ident()
        .map(|id| id == "u8")
        .unwrap_or(false)
}
```

</details>



### `fidius-macro::ir::result_ok_type`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn result_ok_type (ty : & Type) -> Option < & Type >
```

Extract the first type parameter of `Result<_, _>`, if `ty` is a Result.

<details>
<summary>Source</summary>

```rust
fn result_ok_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let last = type_path.path.segments.last()?;
    if last.ident != "Result" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    let first = args.args.first()?;
    match first {
        syn::GenericArgument::Type(t) => Some(t),
        _ => None,
    }
}
```

</details>



### `fidius-macro::ir::validate_raw_method_signature`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn validate_raw_method_signature (method : & TraitItemFn , arg_types : & [Type] , return_type : Option < & Type > ,) -> syn :: Result < () >
```

Validate that a method flagged `#[wire(raw)]` has a supported signature: exactly one `Vec<u8>` argument, and returns either `Vec<u8>` or `Result<Vec<u8>, _>`. Returns a helpful error otherwise.

<details>
<summary>Source</summary>

```rust
fn validate_raw_method_signature(
    method: &TraitItemFn,
    arg_types: &[Type],
    return_type: Option<&Type>,
) -> syn::Result<()> {
    let span = method.sig.ident.span();
    if arg_types.len() != 1 {
        return Err(syn::Error::new(
            span,
            "#[wire(raw)] methods must take exactly one argument of type `Vec<u8>` (excluding `&self`)",
        ));
    }
    if !is_vec_u8(&arg_types[0]) {
        return Err(syn::Error::new(
            arg_types[0].span(),
            "#[wire(raw)] argument must be `Vec<u8>`",
        ));
    }
    let Some(ret) = return_type else {
        return Err(syn::Error::new(
            span,
            "#[wire(raw)] methods must return `Vec<u8>` or `Result<Vec<u8>, E>`",
        ));
    };
    // Return is either Vec<u8> directly, or Result<Vec<u8>, _>.
    if is_vec_u8(ret) {
        return Ok(());
    }
    if let Some(ok) = result_ok_type(ret) {
        if is_vec_u8(ok) {
            return Ok(());
        }
    }
    Err(syn::Error::new(
        ret.span(),
        "#[wire(raw)] methods must return `Vec<u8>` or `Result<Vec<u8>, E>`",
    ))
}
```

</details>



### `fidius-macro::ir::stream_item_type`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn stream_item_type (ty : & Type) -> Option < Type >
```

Return the per-item type `T` if `ty` is a `Stream<T>` (i.e. its final path segment is `Stream` with exactly one angle-bracketed type argument). Matches `fidius::Stream<T>`, `crate::fidius::Stream<T>`, or a bare `Stream<T>` — the detection keys on the segment name, since the marker is written explicitly (FIDIUS-I-0026, D4). Returns `None` for any other type.

<details>
<summary>Source</summary>

```rust
fn stream_item_type(ty: &Type) -> Option<Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let last = type_path.path.segments.last()?;
    if last.ident != "Stream" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    if args.args.len() != 1 {
        return None;
    }
    match args.args.first()? {
        syn::GenericArgument::Type(t) => Some(t.clone()),
        _ => None,
    }
}
```

</details>



### `fidius-macro::ir::build_signature_string`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn build_signature_string (method : & TraitItemFn , wire_raw : bool , stream_item : Option < & Type > ,) -> String
```

Build the canonical signature string for a method.

Delegates the format to `fidius_core::hash::signature_string` so the
proc macro and any other tooling (e.g. `fidius python-stub`) share one
source of truth. The `!raw` (raw-wire) and `!stream` (server-streaming)
markers are part of that shared format.
For a streaming method (`-> fidius::Stream<T>`) the canonical return type is
the per-item type `T` (passed in `stream_item`), plus the `!stream` marker —
so `read:->Row!stream` hashes distinctly from a unary `read:->Row`.

<details>
<summary>Source</summary>

```rust
fn build_signature_string(
    method: &TraitItemFn,
    wire_raw: bool,
    stream_item: Option<&Type>,
) -> String {
    let name = method.sig.ident.to_string();

    let arg_types: Vec<String> = method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => Some(pat_type.ty.to_token_stream().to_string()),
        })
        .collect();

    let ret = match stream_item {
        // Streaming: the canonical return is the per-item type `T`.
        Some(item) => item.to_token_stream().to_string(),
        None => match &method.sig.output {
            ReturnType::Default => String::new(),
            ReturnType::Type(_, ty) => ty.to_token_stream().to_string(),
        },
    };

    fidius_core::hash::signature_string(&name, &arg_types, &ret, wire_raw, stream_item.is_some())
}
```

</details>



### `fidius-macro::ir::extract_arg_names`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn extract_arg_names (method : & TraitItemFn) -> Vec < Ident >
```

Extract argument names from a method signature (excluding `self`).

<details>
<summary>Source</summary>

```rust
fn extract_arg_names(method: &TraitItemFn) -> Vec<Ident> {
    method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                    Some(pat_ident.ident.clone())
                } else {
                    // Fallback for patterns like `_`
                    Some(Ident::new("_arg", pat_type.span()))
                }
            }
        })
        .collect()
}
```

</details>



### `fidius-macro::ir::extract_arg_types`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn extract_arg_types (method : & TraitItemFn) -> Vec < Type >
```

Extract argument types from a method signature (excluding `self`).

<details>
<summary>Source</summary>

```rust
fn extract_arg_types(method: &TraitItemFn) -> Vec<Type> {
    method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => Some((*pat_type.ty).clone()),
        })
        .collect()
}
```

</details>



### `fidius-macro::ir::extract_return_type`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn extract_return_type (method : & TraitItemFn) -> Option < Type >
```

Extract the return type (unwrapped from `-> Type`).

<details>
<summary>Source</summary>

```rust
fn extract_return_type(method: &TraitItemFn) -> Option<Type> {
    match &method.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some((**ty).clone()),
    }
}
```

</details>



### `fidius-macro::ir::parse_interface`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn parse_interface (attrs : InterfaceAttrs , item : & ItemTrait) -> syn :: Result < InterfaceIR >
```

Parse an `ItemTrait` into an `InterfaceIR`.

<details>
<summary>Source</summary>

```rust
pub fn parse_interface(attrs: InterfaceAttrs, item: &ItemTrait) -> syn::Result<InterfaceIR> {
    let trait_name = item.ident.clone();
    let mut methods = Vec::new();
    let mut optional_count = 0u32;

    for trait_item in &item.items {
        let TraitItem::Fn(method) = trait_item else {
            continue;
        };

        // Validate: must take &self, not &mut self
        if let Some(FnArg::Receiver(receiver)) = method.sig.inputs.first() {
            if receiver.mutability.is_some() {
                return Err(syn::Error::new(
                    receiver.span(),
                    "fidius plugins are stateless: methods must take `&self`, not `&mut self`",
                ));
            }
        }

        let optional_since = parse_optional_attr(&method.attrs)?;
        if optional_since.is_some() {
            optional_count += 1;
            if optional_count > 64 {
                return Err(syn::Error::new(
                    method.sig.ident.span(),
                    "fidius supports at most 64 optional methods per interface (u64 capability bitfield)",
                ));
            }
        }

        let is_async = method.sig.asyncness.is_some();
        let wire_raw = parse_wire_attr(&method.attrs)?;
        let arg_types = extract_arg_types(method);
        let arg_names = extract_arg_names(method);
        let return_type = extract_return_type(method);
        let method_metas = parse_meta_attrs(&method.attrs, "method_meta")?;

        // Server-streaming detection (FIDIUS-I-0026, D4): a method whose return
        // type is `fidius::Stream<T>`. Argument-position `Stream<T>`
        // (client-streaming / bidirectional) is rejected — v1 is server-streaming
        // only.
        for at in &arg_types {
            if stream_item_type(at).is_some() {
                return Err(syn::Error::new(
                    at.span(),
                    "fidius v1 supports server-streaming only: `Stream<T>` is not allowed in \
                     argument position (client-streaming and bidirectional are deferred)",
                ));
            }
        }
        let stream_item = return_type.as_ref().and_then(stream_item_type);
        let streaming = stream_item.is_some();

        if wire_raw {
            if is_async {
                return Err(syn::Error::new(
                    method.sig.ident.span(),
                    "#[wire(raw)] is not supported on async methods in this release",
                ));
            }
            // A `#[wire(raw)]` method must return `Vec<u8>`/`Result<Vec<u8>,_>`,
            // which is never a `Stream<T>`; validate_raw_method_signature
            // enforces that, so raw + streaming can't co-occur.
            validate_raw_method_signature(method, &arg_types, return_type.as_ref())?;
        }

        let signature_string = build_signature_string(method, wire_raw, stream_item.as_ref());

        methods.push(MethodIR {
            name: method.sig.ident.clone(),
            arg_types,
            arg_names,
            return_type,
            is_async,
            optional_since,
            signature_string,
            method_metas,
            wire_raw,
            streaming,
            stream_item_type: stream_item,
        });
    }

    let trait_metas = parse_meta_attrs(&item.attrs, "trait_meta")?;

    Ok(InterfaceIR {
        trait_name,
        attrs,
        methods,
        trait_metas,
        original_trait: item.clone(),
    })
}
```

</details>



