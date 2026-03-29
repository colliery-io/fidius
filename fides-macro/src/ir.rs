//! Intermediate representation for parsed plugin interface traits.
//!
//! Both `#[plugin_interface]` and `#[plugin_impl]` consume this IR.

use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, FnArg, Ident, ItemTrait, LitInt, Pat, ReturnType, Token, TraitItem, TraitItemFn,
    Type,
};

/// Parsed attributes from `#[plugin_interface(version = N, buffer = Strategy)]`.
#[derive(Debug, Clone)]
pub struct InterfaceAttrs {
    pub version: u32,
    pub buffer_strategy: BufferStrategyAttr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferStrategyAttr {
    CallerAllocated,
    PluginAllocated,
    Arena,
}

impl Parse for InterfaceAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut version = None;
        let mut buffer = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let _eq: Token![=] = input.parse()?;

            match ident.to_string().as_str() {
                "version" => {
                    let lit: LitInt = input.parse()?;
                    version = Some(lit.base10_parse::<u32>()?);
                }
                "buffer" => {
                    let strategy: Ident = input.parse()?;
                    buffer = Some(match strategy.to_string().as_str() {
                        "CallerAllocated" => BufferStrategyAttr::CallerAllocated,
                        "PluginAllocated" => BufferStrategyAttr::PluginAllocated,
                        "Arena" => BufferStrategyAttr::Arena,
                        _ => {
                            return Err(syn::Error::new(
                                strategy.span(),
                                "expected CallerAllocated, PluginAllocated, or Arena",
                            ))
                        }
                    });
                }
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown attribute `{other}`, expected `version` or `buffer`"),
                    ))
                }
            }

            if !input.is_empty() {
                let _comma: Token![,] = input.parse()?;
            }
        }

        Ok(InterfaceAttrs {
            version: version
                .ok_or_else(|| syn::Error::new(Span::call_site(), "missing `version` attribute"))?,
            buffer_strategy: buffer.ok_or_else(|| {
                syn::Error::new(Span::call_site(), "missing `buffer` attribute")
            })?,
        })
    }
}

/// Full IR for a parsed interface trait.
#[derive(Debug)]
pub struct InterfaceIR {
    pub trait_name: Ident,
    pub attrs: InterfaceAttrs,
    pub methods: Vec<MethodIR>,
    /// The original trait item, for re-emission.
    pub original_trait: ItemTrait,
}

/// IR for a single trait method.
#[derive(Debug)]
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
    /// Format: `"name:arg_type_1,arg_type_2->return_type"`
    pub signature_string: String,
}

impl MethodIR {
    /// Whether this is a required (non-optional) method.
    pub fn is_required(&self) -> bool {
        self.optional_since.is_none()
    }
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

/// Build the canonical signature string for a method.
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
                    "fides plugins are stateless: methods must take `&self`, not `&mut self`",
                ));
            }
        }

        let optional_since = parse_optional_attr(&method.attrs)?;
        if optional_since.is_some() {
            optional_count += 1;
            if optional_count > 64 {
                return Err(syn::Error::new(
                    method.sig.ident.span(),
                    "fides supports at most 64 optional methods per interface (u64 capability bitfield)",
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

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn parse_test_trait(tokens: proc_macro2::TokenStream) -> InterfaceIR {
        let item: ItemTrait = syn::parse2(tokens).expect("failed to parse trait");
        let attrs = InterfaceAttrs {
            version: 1,
            buffer_strategy: BufferStrategyAttr::PluginAllocated,
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
        };
        let result = parse_interface(attrs, &item);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("stateless"));
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
    }
}
