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

//! Intermediate representation for parsed plugin interface traits.
//!
//! Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.

use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, FnArg, Ident, ItemTrait, LitInt, LitStr, Pat, Path, ReturnType, Token, TraitItem,
    TraitItemFn, Type,
};

/// Parsed attributes from `#[plugin_interface(version = N, buffer = Strategy)]`.
#[derive(Debug, Clone)]
pub struct InterfaceAttrs {
    pub version: u32,
    pub buffer_strategy: BufferStrategyAttr,
    /// The path to the fidius crate. Defaults to `fidius` when not specified.
    /// Set via `crate = "my_crate::fidius"` in the attribute.
    pub crate_path: Path,
}

/// Discriminants match `fidius_core::descriptor::BufferStrategyKind` — values
/// `1` (PluginAllocated) and `2` (Arena). `0` is reserved for the removed
/// `CallerAllocated` strategy.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferStrategyAttr {
    PluginAllocated = 1,
    Arena = 2,
}

impl Parse for InterfaceAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut version = None;
        let mut buffer = None;
        let mut crate_path = None;

        while !input.is_empty() {
            // `crate` is a keyword, so we need to handle it specially
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
                "buffer" => {
                    let strategy: Ident = input.parse()?;
                    buffer = Some(match strategy.to_string().as_str() {
                        "PluginAllocated" => BufferStrategyAttr::PluginAllocated,
                        "Arena" => BufferStrategyAttr::Arena,
                        "CallerAllocated" => {
                            return Err(syn::Error::new(
                                strategy.span(),
                                "`CallerAllocated` buffer strategy was removed in fidius 0.1.0; use `PluginAllocated` or `Arena`",
                            ))
                        }
                        _ => {
                            return Err(syn::Error::new(
                                strategy.span(),
                                "expected PluginAllocated or Arena",
                            ))
                        }
                    });
                }
                "crate" => {
                    let lit: LitStr = input.parse()?;
                    let path: Path = lit.parse()?;
                    crate_path = Some(path);
                }
                other => {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!(
                            "unknown attribute `{other}`, expected `version`, `buffer`, or `crate`"
                        ),
                    ))
                }
            }

            if !input.is_empty() {
                let _comma: Token![,] = input.parse()?;
            }
        }

        // Default crate path to `fidius`
        let crate_path = crate_path.unwrap_or_else(|| syn::parse_str::<Path>("fidius").unwrap());

        Ok(InterfaceAttrs {
            version: version
                .ok_or_else(|| syn::Error::new(Span::call_site(), "missing `version` attribute"))?,
            buffer_strategy: buffer
                .ok_or_else(|| syn::Error::new(Span::call_site(), "missing `buffer` attribute"))?,
            crate_path,
        })
    }
}

/// A static metadata key/value pair parsed from a `#[method_meta(...)]`
/// or `#[trait_meta(...)]` attribute. Both values are string literals.
#[derive(Debug, Clone)]
pub struct MetaKvAttr {
    pub key: String,
    pub value: String,
}

/// Full IR for a parsed interface trait.
#[derive(Debug)]
pub struct InterfaceIR {
    pub trait_name: Ident,
    pub attrs: InterfaceAttrs,
    pub methods: Vec<MethodIR>,
    /// Trait-level metadata from `#[trait_meta(...)]` attributes on the trait.
    pub trait_metas: Vec<MetaKvAttr>,
    /// The original trait item, for re-emission.
    pub original_trait: ItemTrait,
}

/// IR for a single trait method.
#[derive(Debug)]
#[allow(dead_code)]
pub struct MethodIR {
    pub name: Ident,
    /// Argument types (excluding `self`).
    pub arg_types: Vec<Type>,
    /// Argument names (excluding `self`).
    pub arg_names: Vec<Ident>,
    /// Return type (the inner type, not `ReturnType`).
    pub return_type: Option<Type>,
    /// Whether the method is `async fn`.
    pub is_async: bool,
    /// If `#[optional(since = N)]`, the version it was added.
    pub optional_since: Option<u32>,
    /// Canonical signature string for interface hashing.
    /// Format: `"name:arg_type_1,arg_type_2->return_type"`, with a trailing
    /// `!raw` marker for methods opted into raw wire mode so the interface
    /// hash diverges between raw and bincode-typed versions.
    pub signature_string: String,
    /// Metadata from `#[method_meta("k", "v")]` attributes. Preserves declaration order.
    pub method_metas: Vec<MetaKvAttr>,
    /// Whether this method is opted into raw (byte-passthrough) wire mode
    /// via `#[wire(raw)]`. When true, the macro skips bincode on the
    /// success path — the single `Vec<u8>` argument crosses the FFI
    /// boundary as raw bytes, and the `Vec<u8>` return value is handed to
    /// the host unchanged. Error-path payloads (for `Result<Vec<u8>, E>`
    /// returns) continue to go through bincode.
    pub wire_raw: bool,
}

impl MethodIR {
    /// Whether this is a required (non-optional) method.
    pub fn is_required(&self) -> bool {
        self.optional_since.is_none()
    }
}

/// Parse all `#[method_meta("k", "v")]` or `#[trait_meta("k", "v")]`
/// attributes with the given name from an attribute list into a `Vec<MetaKvAttr>`.
/// Validates string-literal only, non-empty keys, no duplicate keys, and
/// rejects keys in the reserved `fidius.*` namespace.
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

/// Parse an `#[optional(since = N)]` attribute, if present.
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

/// Parse a `#[wire(raw)]` attribute, if present. Returns `true` when raw wire
/// mode is opted in, `false` otherwise. Any other `wire(...)` form is a
/// compile-time error.
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

/// Return `true` if the given type is `Vec<u8>`.
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

/// Extract the first type parameter of `Result<_, _>`, if `ty` is a Result.
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

/// Validate that a method flagged `#[wire(raw)]` has a supported signature:
/// exactly one `Vec<u8>` argument, and returns either `Vec<u8>` or
/// `Result<Vec<u8>, _>`. Returns a helpful error otherwise.
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

/// Build the canonical signature string for a method.
///
/// Delegates the format to `fidius_core::hash::signature_string` so the
/// proc macro and any other tooling (e.g. `fidius python-stub`) share one
/// source of truth. The `!raw` marker for `#[wire(raw)]` methods is part
/// of that shared format.
fn build_signature_string(method: &TraitItemFn, wire_raw: bool) -> String {
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

    fidius_core::hash::signature_string(&name, &arg_types, &ret, wire_raw)
}

/// Extract argument names from a method signature (excluding `self`).
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

/// Extract argument types from a method signature (excluding `self`).
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

/// Extract the return type (unwrapped from `-> Type`).
fn extract_return_type(method: &TraitItemFn) -> Option<Type> {
    match &method.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some((**ty).clone()),
    }
}

/// Parse an `ItemTrait` into an `InterfaceIR`.
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

        if wire_raw {
            if is_async {
                return Err(syn::Error::new(
                    method.sig.ident.span(),
                    "#[wire(raw)] is not supported on async methods in this release",
                ));
            }
            validate_raw_method_signature(method, &arg_types, return_type.as_ref())?;
        }

        let signature_string = build_signature_string(method, wire_raw);

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

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn parse_test_trait(tokens: proc_macro2::TokenStream) -> InterfaceIR {
        let item: ItemTrait = syn::parse2(tokens).expect("failed to parse trait");
        let attrs = InterfaceAttrs {
            version: 1,
            buffer_strategy: BufferStrategyAttr::PluginAllocated,
            crate_path: syn::parse_str("fidius").unwrap(),
        };
        parse_interface(attrs, &item).expect("failed to parse interface")
    }

    #[test]
    fn basic_trait_parsing() {
        let ir = parse_test_trait(quote! {
            pub trait Greeter: Send + Sync {
                fn greet(&self, name: String) -> String;
            }
        });

        assert_eq!(ir.trait_name, "Greeter");
        assert_eq!(ir.methods.len(), 1);

        let m = &ir.methods[0];
        assert_eq!(m.name, "greet");
        assert!(!m.is_async);
        assert!(m.is_required());
        assert_eq!(m.arg_types.len(), 1);
        assert!(m.return_type.is_some());
        assert!(m.signature_string.starts_with("greet:"));
    }

    #[test]
    fn optional_method_parsing() {
        let ir = parse_test_trait(quote! {
            pub trait Filter: Send + Sync {
                fn process(&self, data: Vec<u8>) -> Vec<u8>;

                #[optional(since = 2)]
                fn process_v2(&self, data: Vec<u8>, opts: String) -> Vec<u8>;
            }
        });

        assert_eq!(ir.methods.len(), 2);
        assert!(ir.methods[0].is_required());
        assert_eq!(ir.methods[1].optional_since, Some(2));
    }

    #[test]
    fn async_method_detection() {
        let ir = parse_test_trait(quote! {
            pub trait AsyncProcessor: Send + Sync {
                async fn process(&self, input: String) -> String;
                fn sync_method(&self) -> u32;
            }
        });

        assert!(ir.methods[0].is_async);
        assert!(!ir.methods[1].is_async);
    }

    #[test]
    fn rejects_mut_self() {
        let item: ItemTrait = syn::parse2(quote! {
            pub trait Bad: Send + Sync {
                fn mutate(&mut self);
            }
        })
        .unwrap();

        let attrs = InterfaceAttrs {
            version: 1,
            buffer_strategy: BufferStrategyAttr::PluginAllocated,
            crate_path: syn::parse_str("fidius").unwrap(),
        };
        let result = parse_interface(attrs, &item);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("stateless"));
    }

    #[test]
    fn signature_string_format() {
        let ir = parse_test_trait(quote! {
            pub trait Example: Send + Sync {
                fn foo(&self, a: u32, b: String) -> bool;
            }
        });

        let sig = &ir.methods[0].signature_string;
        assert!(sig.starts_with("foo:"));
        assert!(sig.contains("->"));
    }

    #[test]
    fn interface_attrs_parsing() {
        let attrs: InterfaceAttrs = syn::parse_str("version = 3, buffer = Arena").unwrap();
        assert_eq!(attrs.version, 3);
        assert_eq!(attrs.buffer_strategy, BufferStrategyAttr::Arena);
        // Default crate path
        assert_eq!(attrs.crate_path.segments.last().unwrap().ident, "fidius");
    }

    #[test]
    fn interface_attrs_with_crate_path() {
        let attrs: InterfaceAttrs =
            syn::parse_str(r#"version = 1, buffer = PluginAllocated, crate = "my_crate::fidius""#)
                .unwrap();
        assert_eq!(attrs.version, 1);
        assert_eq!(attrs.buffer_strategy, BufferStrategyAttr::PluginAllocated);
        let segments: Vec<String> = attrs
            .crate_path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        assert_eq!(segments, vec!["my_crate", "fidius"]);
    }
}
