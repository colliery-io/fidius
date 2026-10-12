---
title: "fidius-macro::ir"
description: "Intermediate representation for parsed plugin interface traits."
editUrl: false
sidebar:
  order: 49
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Intermediate representation for parsed plugin interface traits.

Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.

## Structs

### <span id="struct.InterfaceAttrs"></span><code style="font-size: 0.85em">struct InterfaceAttrs</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

**Derives:** `Debug`, `Clone`

Parsed attributes from `#[plugin_interface(version = N, buffer = Strategy)]`.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `version` | `u32` |  |
| `buffer_strategy` | `BufferStrategyAttr` |  |
| `crate_path` | `Path` | The path to the fidius crate. Defaults to `fidius` when not specified. Set via `crate = "my_crate::fidius"` in the attribute. |

### <span id="struct.MetaKvAttr"></span><code style="font-size: 0.85em">struct MetaKvAttr</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

**Derives:** `Debug`, `Clone`

A static metadata key/value pair parsed from a `#[method_meta(...)]` or `#[trait_meta(...)]` attribute. Both values are string literals.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `key` | `String` |  |
| `value` | `String` |  |

### <span id="struct.InterfaceIR"></span><code style="font-size: 0.85em">struct InterfaceIR</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

**Derives:** `Debug`

Full IR for a parsed interface trait.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `trait_name` | `Ident` |  |
| `attrs` | `InterfaceAttrs` |  |
| `methods` | `Vec<MethodIR>` |  |
| `trait_metas` | `Vec<MetaKvAttr>` | Trait-level metadata from `#[trait_meta(...)]` attributes on the trait. |
| `original_trait` | `ItemTrait` | The original trait item, for re-emission. |

### <span id="struct.MethodIR"></span><code style="font-size: 0.85em">struct MethodIR</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

**Derives:** `Debug`

IR for a single trait method.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `name` | `Ident` |  |
| `arg_types` | `Vec<Type>` | Argument types (excluding `self`). |
| `arg_names` | `Vec<Ident>` | Argument names (excluding `self`). |
| `return_type` | `Option<Type>` | Return type (the inner type, not `ReturnType`). |
| `is_async` | `bool` | Whether the method is `async fn`. |
| `optional_since` | `Option<u32>` | If `#[optional(since = N)]`, the version it was added. |
| `signature_string` | `String` | Canonical signature string for interface hashing. Format: `"name:arg_type_1,arg_type_2->return_type"`, with a trailing `!raw` marker for methods opted into raw wire mode so the interface hash diverges between raw and bincode-typed versions. |
| `method_metas` | `Vec<MetaKvAttr>` | Metadata from `#[method_meta("k", "v")]` attributes. Preserves declaration order. |
| `wire_raw` | `bool` | Whether this method is opted into raw (byte-passthrough) wire mode via `#[wire(raw)]`. When true, the macro skips bincode on the success path — the single `Vec<u8>` argument crosses the FFI boundary as raw bytes, and the `Vec<u8>` return value is handed to the host unchanged. Error-path payloads (for `Result<Vec<u8>, E>` returns) continue to go through bincode. |
| `streaming` | `bool` | Whether this is a server-streaming method — its return type is `fidius::Stream<T>` (FIDIUS-I-0026, D4). When true, `Self::stream_item_type` holds the per-item type `T`, the signature string carries a `!stream` marker (so it hashes distinctly from a unary `-> T`), and the host-side client (ST.3) returns a `ChunkStream` instead of a `Result<T, _>`. |
| `stream_item_type` | `Option<Type>` | The per-item type `T` for a `streaming` method (the `T` in `fidius::Stream<T>`). `None` for non-streaming methods. |
| `client_stream_item` | `Option<Type>` | The per-item type `T` for a **client-streaming** method — a `Stream<T>` in argument position (FIDIUS-I-0030). The signature string carries a `<stream` marker so it hashes distinctly. `None` for non-client-streaming methods. (Codegen is wired per backend in CS2.2–CS2.4.) |

#### Methods

##### <span id="MethodIR.is_required"></span><code>is_​required</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn is_required(&self) -> bool
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

### <span id="enum.BufferStrategyAttr"></span><code style="font-size: 0.85em">enum BufferStrategyAttr</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

Discriminants match `fidius_core::descriptor::BufferStrategyKind` — values `1` (PluginAllocated) and `2` (Arena). `0` is reserved for the removed `CallerAllocated` strategy.

#### Variants

- **`PluginAllocated`**
- **`Arena`**

## Functions

### <span id="fn.parse_interface"></span><code style="font-size: 0.85em">fn parse_​interface</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn parse_interface(
    attrs: InterfaceAttrs,
    item: &ItemTrait,
) -> syn::Result<InterfaceIR>
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

        // Server-streaming (FIDIUS-I-0026, D4): a `-> fidius::Stream<T>` return.
        let stream_item = return_type.as_ref().and_then(stream_item_type);
        let streaming = stream_item.is_some();
        // Client-streaming (FIDIUS-I-0030 / ADR-0007): a `Stream<T>` in ARGUMENT
        // position. Recognized in the IR + folded into the interface hash here; the
        // per-backend pull channel is wired in CS2.2–CS2.4, so for now a clear
        // "not yet wired" error is returned below (keeps the compile-fail guard).
        let client_stream_item: Option<Type> = arg_types.iter().find_map(stream_item_type);

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

        let signature_string = build_signature_string(
            method,
            wire_raw,
            stream_item.as_ref(),
            client_stream_item.is_some(),
        );

        // Client-streaming (FIDIUS-I-0030): recognized + hashed in CS2.1; the
        // cdylib backend is wired in CS2.2. The WASM adapter + the typed Client
        // still reject/skip it (CS2.3/CS2.5), so non-cdylib paths stay guarded.

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
            client_stream_item,
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
