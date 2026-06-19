# fidius-cli::python_stub <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Python stub generator: turn a `#[plugin_interface]` trait into a `.py` file the plugin author imports. Gives them type-hinted signatures and the `__interface_hash__` constant the host expects at load time.

The stub computes its hash via `fidius_core::hash::interface_hash` over
signature strings produced by `fidius_core::hash::signature_string` —
the same call path the proc macro uses, so the two are guaranteed to
agree byte-for-byte.

## Structs

### `fidius-cli::python_stub::MethodSpec`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


One method extracted from a trait, ready for stub emission.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `name` | `String` |  |
| `arg_types` | `Vec < String >` | Argument types as raw `to_token_stream().to_string()` — matches the
proc macro's signature-string format exactly. |
| `arg_names_with_py_types` | `Vec < (String , String) >` | Argument names paired with their parsed Python type hint. |
| `return_type_string` | `String` | Return type stringified the same way as `arg_types` (empty string for
methods returning `()`). |
| `return_py_type` | `String` | Python type hint for the return value. |
| `wire_raw` | `bool` | `#[wire(raw)]` opt-in. Forces argument and return to bytes regardless
of the underlying Rust signature. |
| `docs` | `Vec < String >` | Doc comment lines (without leading `///`). |



## Functions

### `fidius-cli::python_stub::generate_stub`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn generate_stub (interface_src : & Path , requested_trait : Option < & str >) -> Result < String >
```

Generate the contents of a Python stub file for the named trait found in `interface_src`. If `requested_trait` is `None`, exactly one `#[plugin_interface]` trait must be present.

<details>
<summary>Source</summary>

```rust
pub fn generate_stub(interface_src: &Path, requested_trait: Option<&str>) -> Result<String> {
    let source = std::fs::read_to_string(interface_src)?;
    let parsed: File = syn::parse_file(&source)
        .map_err(|e| format!("failed to parse {}: {e}", interface_src.display()))?;

    let traits: Vec<&ItemTrait> = parsed
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Trait(t) if has_plugin_interface_attr(t) => Some(t),
            _ => None,
        })
        .collect();

    let target = pick_trait(&traits, requested_trait, interface_src)?;
    let methods = extract_methods(target)?;

    Ok(render_python_stub(&target.ident.to_string(), &methods))
}
```

</details>



### `fidius-cli::python_stub::write_stub`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn write_stub (interface_src : & Path , out_path : & Path , requested_trait : Option < & str >) -> Result
```

Write the stub for the named trait to `out_path`. Special-cases `-` for stdout.

<details>
<summary>Source</summary>

```rust
pub fn write_stub(interface_src: &Path, out_path: &Path, requested_trait: Option<&str>) -> Result {
    let stub = generate_stub(interface_src, requested_trait)?;
    if out_path.as_os_str() == "-" {
        print!("{stub}");
    } else {
        let len = stub.len();
        std::fs::write(out_path, stub)?;
        println!(
            "Wrote Python stub for trait to {} ({} bytes)",
            out_path.display(),
            len
        );
    }
    Ok(())
}
```

</details>



### `fidius-cli::python_stub::has_plugin_interface_attr`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn has_plugin_interface_attr (item : & ItemTrait) -> bool
```

<details>
<summary>Source</summary>

```rust
fn has_plugin_interface_attr(item: &ItemTrait) -> bool {
    item.attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .map(|s| s.ident == "plugin_interface")
            .unwrap_or(false)
    })
}
```

</details>



### `fidius-cli::python_stub::pick_trait`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn pick_trait < 'a > (traits : & 'a [& 'a ItemTrait] , requested : Option < & str > , src : & Path ,) -> Result < & 'a ItemTrait >
```

<details>
<summary>Source</summary>

```rust
fn pick_trait<'a>(
    traits: &'a [&'a ItemTrait],
    requested: Option<&str>,
    src: &Path,
) -> Result<&'a ItemTrait> {
    match (traits.len(), requested) {
        (0, _) => Err(format!("no `#[plugin_interface]` trait found in {}", src.display()).into()),
        (1, None) => Ok(traits[0]),
        (_, None) => {
            let names: Vec<String> = traits.iter().map(|t| t.ident.to_string()).collect();
            Err(format!(
                "multiple `#[plugin_interface]` traits in {}: {}. Pass --trait-name to choose.",
                src.display(),
                names.join(", ")
            )
            .into())
        }
        (_, Some(name)) => traits
            .iter()
            .find(|t| t.ident == name)
            .copied()
            .ok_or_else(|| {
                format!(
                    "trait '{name}' not found in {}; available: {}",
                    src.display(),
                    traits
                        .iter()
                        .map(|t| t.ident.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
                .into()
            }),
    }
}
```

</details>



### `fidius-cli::python_stub::extract_methods`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn extract_methods (item : & ItemTrait) -> Result < Vec < MethodSpec > >
```

<details>
<summary>Source</summary>

```rust
fn extract_methods(item: &ItemTrait) -> Result<Vec<MethodSpec>> {
    let mut out = Vec::new();
    for trait_item in &item.items {
        let TraitItem::Fn(method) = trait_item else {
            continue;
        };
        out.push(method_to_spec(method)?);
    }
    Ok(out)
}
```

</details>



### `fidius-cli::python_stub::method_to_spec`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn method_to_spec (method : & TraitItemFn) -> Result < MethodSpec >
```

<details>
<summary>Source</summary>

```rust
fn method_to_spec(method: &TraitItemFn) -> Result<MethodSpec> {
    let wire_raw = method.attrs.iter().any(is_wire_raw_attr);

    let arg_types: Vec<String> = method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat) => Some(token_string(&pat.ty)),
        })
        .collect();

    let arg_names_with_py_types: Vec<(String, String)> = method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat) => {
                let name = match pat.pat.as_ref() {
                    syn::Pat::Ident(p) => p.ident.to_string(),
                    _ => "_arg".to_string(),
                };
                let py = if wire_raw {
                    "bytes".to_string()
                } else {
                    rust_type_to_python(&pat.ty)
                };
                Some((name, py))
            }
        })
        .collect();

    let (return_type_string, return_py_type) = match &method.sig.output {
        ReturnType::Default => (String::new(), "None".to_string()),
        ReturnType::Type(_, ty) => {
            let s = token_string(ty);
            let py = if wire_raw {
                // For Result<Vec<u8>, _> we still surface bytes — the error
                // path is bincode-encoded on the host side.
                "bytes".to_string()
            } else {
                rust_type_to_python(ty)
            };
            (s, py)
        }
    };

    let docs = method.attrs.iter().filter_map(extract_doc_line).collect();

    Ok(MethodSpec {
        name: method.sig.ident.to_string(),
        arg_types,
        arg_names_with_py_types,
        return_type_string,
        return_py_type,
        wire_raw,
        docs,
    })
}
```

</details>



### `fidius-cli::python_stub::is_wire_raw_attr`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn is_wire_raw_attr (attr : & syn :: Attribute) -> bool
```

<details>
<summary>Source</summary>

```rust
fn is_wire_raw_attr(attr: &syn::Attribute) -> bool {
    if !attr.path().is_ident("wire") {
        return false;
    }
    let mut raw = false;
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("raw") {
            raw = true;
        }
        Ok(())
    });
    raw
}
```

</details>



### `fidius-cli::python_stub::token_string`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn token_string < T : ToTokens > (t : & T) -> String
```

<details>
<summary>Source</summary>

```rust
fn token_string<T: ToTokens>(t: &T) -> String {
    t.to_token_stream().to_string()
}
```

</details>



### `fidius-cli::python_stub::extract_doc_line`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn extract_doc_line (attr : & syn :: Attribute) -> Option < String >
```

<details>
<summary>Source</summary>

```rust
fn extract_doc_line(attr: &syn::Attribute) -> Option<String> {
    if !attr.path().is_ident("doc") {
        return None;
    }
    if let syn::Meta::NameValue(nv) = &attr.meta {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &nv.value
        {
            return Some(s.value().trim().to_string());
        }
    }
    None
}
```

</details>



### `fidius-cli::python_stub::rust_type_to_python`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn rust_type_to_python (ty : & Type) -> String
```

Map a Rust type to its Python type-hint counterpart. Unknown types fall back to `Any` with a TODO comment surfaced in the rendered stub.

<details>
<summary>Source</summary>

```rust
fn rust_type_to_python(ty: &Type) -> String {
    match ty {
        Type::Path(tp) => {
            let segs = &tp.path.segments;
            if segs.is_empty() {
                return "Any  # TODO: empty type path".to_string();
            }
            let last = segs.last().unwrap();
            let name = last.ident.to_string();

            // Generic helpers: Vec<T>, Option<T>, Result<T, E>
            if let syn::PathArguments::AngleBracketed(args) = &last.arguments {
                let type_args: Vec<&Type> = args
                    .args
                    .iter()
                    .filter_map(|a| match a {
                        syn::GenericArgument::Type(t) => Some(t),
                        _ => None,
                    })
                    .collect();

                match name.as_str() {
                    "Vec" if type_args.len() == 1 => {
                        // Vec<u8> → bytes (special-case; matches raw-wire and Python idiom)
                        if is_u8(type_args[0]) {
                            return "bytes".to_string();
                        }
                        return format!("list[{}]", rust_type_to_python(type_args[0]));
                    }
                    "Option" if type_args.len() == 1 => {
                        return format!("Optional[{}]", rust_type_to_python(type_args[0]));
                    }
                    "Result" if type_args.len() == 2 => {
                        // Python plugins surface `Result<T, E>` as a successful T;
                        // errors are raised via fidius.PluginError.
                        return rust_type_to_python(type_args[0]);
                    }
                    _ => {}
                }
            }

            match name.as_str() {
                "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize" => {
                    "int".to_string()
                }
                "f32" | "f64" => "float".to_string(),
                "bool" => "bool".to_string(),
                "String" => "str".to_string(),
                _ => format!("Any  # TODO: unsupported Rust type `{name}`"),
            }
        }
        Type::Reference(r) => match r.elem.as_ref() {
            Type::Path(p)
                if p.path
                    .segments
                    .last()
                    .map(|s| s.ident == "str")
                    .unwrap_or(false) =>
            {
                "str".to_string()
            }
            Type::Slice(s) if is_u8(&s.elem) => "bytes".to_string(),
            _ => format!("Any  # TODO: unsupported reference `{}`", token_string(ty)),
        },
        Type::Tuple(t) if t.elems.is_empty() => "None".to_string(),
        _ => format!("Any  # TODO: unsupported Rust type `{}`", token_string(ty)),
    }
}
```

</details>



### `fidius-cli::python_stub::is_u8`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn is_u8 (ty : & Type) -> bool
```

<details>
<summary>Source</summary>

```rust
fn is_u8(ty: &Type) -> bool {
    if let Type::Path(p) = ty {
        return p.path.get_ident().map(|i| i == "u8").unwrap_or(false);
    }
    false
}
```

</details>



### `fidius-cli::python_stub::render_python_stub`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn render_python_stub (trait_name : & str , methods : & [MethodSpec]) -> String
```

<details>
<summary>Source</summary>

```rust
fn render_python_stub(trait_name: &str, methods: &[MethodSpec]) -> String {
    let signatures: Vec<String> = methods
        .iter()
        .map(|m| {
            fidius_core::hash::signature_string(
                &m.name,
                &m.arg_types,
                &m.return_type_string,
                m.wire_raw,
                // The Python stub generator does not yet model server-streaming
                // methods (Phase-1 streaming-Python authoring is a later task);
                // non-streaming until then.
                false,
            )
        })
        .collect();
    let sig_refs: Vec<&str> = signatures.iter().map(|s| s.as_str()).collect();
    // Match the macro's `generate_constants` behaviour: only required methods
    // contribute to the hash. The proc macro filters by `is_required()`; we
    // don't yet model `#[optional]` on the stub side because Python plugins
    // typically implement the full interface — if we ever support optional
    // method declarations in stubs we'll filter here too.
    let hash = fidius_core::hash::interface_hash(&sig_refs);

    let any_unsupported = methods.iter().any(|m| {
        m.return_py_type.contains("# TODO")
            || m.arg_names_with_py_types
                .iter()
                .any(|(_, t)| t.contains("# TODO"))
    });

    let mut out = String::new();
    out.push_str("# This file was generated by `fidius python-stub`. DO NOT EDIT BY HAND —\n");
    out.push_str("# regenerate when the underlying Rust interface changes.\n");
    out.push_str(&format!(
        "# Interface: {trait_name}  (interface_hash = {hash:#018x})\n\n"
    ));
    out.push_str("from __future__ import annotations\n\n");
    out.push_str("from typing import Any, Optional\n\n");
    out.push_str("from fidius import method\n\n");
    out.push_str(&format!("__interface_hash__ = {hash:#018x}\n\n"));

    if any_unsupported {
        out.push_str(
            "# Some method signatures contain Rust types that don't have a built-in Python\n\
            # mapping. The stub falls back to `Any` for those — fill in a more specific type\n\
            # if you can, and consider whether the host-side interface should be simplified.\n\n",
        );
    }

    for m in methods {
        for line in &m.docs {
            out.push_str(&format!("# {line}\n"));
        }
        if m.wire_raw {
            out.push_str("# (raw-wire method: bytes in / bytes out, no bincode)\n");
        }
        out.push_str("@method\n");
        out.push_str(&format!("def {}(", m.name));
        let arg_text = m
            .arg_names_with_py_types
            .iter()
            .map(|(n, t)| format!("{n}: {t}"))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&arg_text);
        out.push_str(&format!(") -> {}:\n", m.return_py_type));
        out.push_str("    raise NotImplementedError\n\n");
    }

    out
}
```

</details>



