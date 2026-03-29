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
Format: `"name:arg_type_1,arg_type_2->return_type"` |

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


#### Variants

- **`CallerAllocated`**
- **`PluginAllocated`**
- **`Arena`**



## Functions

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



### `fidius-macro::ir::build_signature_string`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn build_signature_string (method : & TraitItemFn) -> String
```

Build the canonical signature string for a method.

<details>
<summary>Source</summary>

```rust
fn build_signature_string(method: &TraitItemFn) -> String {
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

    let ret = match &method.sig.output {
        ReturnType::Default => String::new(),
        ReturnType::Type(_, ty) => ty.to_token_stream().to_string(),
    };

    format!("{}:{}->{}", name, arg_types.join(","), ret)
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
        let signature_string = build_signature_string(method);
        let arg_types = extract_arg_types(method);
        let arg_names = extract_arg_names(method);
        let return_type = extract_return_type(method);

        methods.push(MethodIR {
            name: method.sig.ident.clone(),
            arg_types,
            arg_names,
            return_type,
            is_async,
            optional_since,
            signature_string,
        });
    }

    Ok(InterfaceIR {
        trait_name,
        attrs,
        methods,
        original_trait: item.clone(),
    })
}
```

</details>



