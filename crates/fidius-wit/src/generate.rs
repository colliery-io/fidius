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

//! Source-parsing WIT generator (FIDIUS-I-0023).
//!
//! Parses a plugin crate's Rust source, finds the `#[plugin_interface]` trait and
//! every `#[derive(WitType)]` `struct`/`enum`, and produces (a) a complete `.wit`
//! document (records/variants + funcs) and (b) the Rust source for
//! generated↔author `From` conversions the wasm adapter includes. A proc-macro
//! can't see external type definitions, so this runs from a `build.rs` helper and
//! the `fidius wit` CLI, which read the source files.

use std::collections::BTreeSet;

use syn::{FnArg, Item, Pat, ReturnType, TraitItem, Type};

use crate::{
    enum_to_variant, return_to_wit_with, struct_to_record, to_kebab_case, wit_type_with, WitMethod,
};

/// The product of generating from a plugin crate's source.
pub struct Generated {
    /// The interface's Rust trait name (e.g. `Greeter`).
    pub interface_name: String,
    /// kebab-case interface name (the WIT package + interface name).
    pub iface_kebab: String,
    /// Rust idents of the `#[derive(WitType)]` user types found.
    pub user_types: Vec<String>,
    /// The complete `.wit` document.
    pub wit: String,
    /// Rust source for the generated↔author `From` conversions (to be `include!`d
    /// inside the wasm adapter module). Empty when there are no user types.
    pub conversions: String,
}

/// Generate WIT + conversions from a single source string (typically a crate's
/// `lib.rs`). v1 expects the `#[plugin_interface]` trait and all
/// `#[derive(WitType)]` types to be in the parsed file.
pub fn generate(src: &str) -> Result<Generated, String> {
    let file = syn::parse_file(src).map_err(|e| format!("parse error: {e}"))?;

    // Collect WitType structs/enums and locate the interface trait.
    let mut structs: Vec<&syn::ItemStruct> = Vec::new();
    let mut enums: Vec<&syn::ItemEnum> = Vec::new();
    let mut the_trait: Option<&syn::ItemTrait> = None;

    for item in &file.items {
        match item {
            Item::Struct(s) if has_derive(&s.attrs, "WitType") => structs.push(s),
            Item::Enum(e) if has_derive(&e.attrs, "WitType") => enums.push(e),
            Item::Trait(t) if has_attr(&t.attrs, "plugin_interface") => {
                if the_trait.is_some() {
                    return Err("multiple #[plugin_interface] traits in one file".into());
                }
                the_trait = Some(t);
            }
            _ => {}
        }
    }

    let the_trait = the_trait.ok_or("no #[plugin_interface] trait found in source")?;
    let interface_name = the_trait.ident.to_string();
    let iface_kebab = to_kebab_case(&interface_name);

    let mut user_types: Vec<String> = Vec::new();
    user_types.extend(structs.iter().map(|s| s.ident.to_string()));
    user_types.extend(enums.iter().map(|e| e.ident.to_string()));
    let known: BTreeSet<String> = user_types.iter().cloned().collect();

    // Type defs (records then variants) in declaration order.
    let mut type_defs: Vec<String> = Vec::new();
    for s in &structs {
        type_defs.push(struct_to_record(s, &known)?);
    }
    for e in &enums {
        type_defs.push(enum_to_variant(e, &known)?);
    }

    // Methods → WIT funcs.
    let mut methods: Vec<WitMethod> = Vec::new();
    for item in &the_trait.items {
        let TraitItem::Fn(f) = item else { continue };
        let mut params = Vec::new();
        for arg in &f.sig.inputs {
            let FnArg::Typed(pt) = arg else { continue }; // skip &self
            let name = match pt.pat.as_ref() {
                Pat::Ident(id) => to_kebab_case(&id.ident.to_string()),
                _ => "arg".to_string(),
            };
            let wt = wit_type_with(&pt.ty, &known)
                .map_err(|e| format!("method `{}` arg `{name}`: {e}", f.sig.ident))?;
            params.push((name, wt));
        }
        let ret_ty: Option<&Type> = match &f.sig.output {
            ReturnType::Type(_, t) => Some(t.as_ref()),
            ReturnType::Default => None,
        };
        let ret = return_to_wit_with(ret_ty, &known)
            .map_err(|e| format!("method `{}` return: {e}", f.sig.ident))?;
        methods.push(WitMethod {
            name: to_kebab_case(&f.sig.ident.to_string()),
            params,
            ret,
        });
    }

    let wit = crate::render_wit_full(&iface_kebab, &type_defs, &methods);
    let conversions = render_conversions(&iface_kebab, &structs, &enums, &known);

    Ok(Generated {
        interface_name,
        iface_kebab,
        user_types,
        wit,
        conversions,
    })
}

/// Render `From` impls (both directions) between each user type and its
/// wit-bindgen-generated mirror. Emitted into the adapter module, where the
/// generated types live at `exports::fidius::<iface>::<iface>::<Type>` and the
/// author types at `crate::<Type>` (v1: WitType types live at the crate root).
fn render_conversions(
    iface_kebab: &str,
    structs: &[&syn::ItemStruct],
    enums: &[&syn::ItemEnum],
    known: &BTreeSet<String>,
) -> String {
    if structs.is_empty() && enums.is_empty() {
        return String::new();
    }
    let snake = iface_kebab.replace('-', "_");
    let gen_path = format!("exports::fidius::{snake}::{snake}");
    let mut out = String::new();
    out.push_str("// Generated by fidius-wit: author <-> wit-bindgen conversions.\n");

    for s in structs {
        let name = s.ident.to_string();
        let g = format!("{gen_path}::{name}");
        let a = format!("crate::{name}");
        let fields: Vec<String> = match &s.fields {
            syn::Fields::Named(f) => f
                .named
                .iter()
                .map(|fl| fl.ident.as_ref().unwrap().to_string())
                .collect(),
            _ => Vec::new(),
        };
        let field_types: Vec<&Type> = match &s.fields {
            syn::Fields::Named(f) => f.named.iter().map(|fl| &fl.ty).collect(),
            _ => Vec::new(),
        };
        // generated -> author
        let to_author: Vec<String> = fields
            .iter()
            .zip(&field_types)
            .map(|(f, ty)| format!("{f}: {}", conv_expr(&format!("v.{f}"), ty, known)))
            .collect();
        out.push_str(&format!(
            "impl ::core::convert::From<{g}> for {a} {{ fn from(v: {g}) -> Self {{ {a} {{ {} }} }} }}\n",
            to_author.join(", ")
        ));
        // author -> generated
        let to_gen: Vec<String> = fields
            .iter()
            .zip(&field_types)
            .map(|(f, ty)| format!("{f}: {}", conv_expr(&format!("v.{f}"), ty, known)))
            .collect();
        out.push_str(&format!(
            "impl ::core::convert::From<{a}> for {g} {{ fn from(v: {a}) -> Self {{ {g} {{ {} }} }} }}\n",
            to_gen.join(", ")
        ));
    }

    for e in enums {
        let name = e.ident.to_string();
        let g = format!("{gen_path}::{name}");
        let a = format!("crate::{name}");
        let arms = |from: &str, to: &str| -> String {
            e.variants
                .iter()
                .map(|v| {
                    let case = v.ident.to_string();
                    match &v.fields {
                        syn::Fields::Unit => {
                            format!("{from}::{case} => {to}::{case}")
                        }
                        syn::Fields::Unnamed(u) if u.unnamed.len() == 1 => {
                            let conv = conv_expr("x", &u.unnamed[0].ty, known);
                            format!("{from}::{case}(x) => {to}::{case}({conv})")
                        }
                        _ => format!("{from}::{case} {{ .. }} => unreachable!()"),
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        out.push_str(&format!(
            "impl ::core::convert::From<{g}> for {a} {{ fn from(v: {g}) -> Self {{ match v {{ {} }} }} }}\n",
            arms(&g, &a)
        ));
        out.push_str(&format!(
            "impl ::core::convert::From<{a}> for {g} {{ fn from(v: {a}) -> Self {{ match v {{ {} }} }} }}\n",
            arms(&a, &g)
        ));
    }
    out
}

/// Conversion expression for a field/payload `access` of type `ty`. Identity
/// (move) when the type holds no user type (generated and author types are then
/// identical); otherwise `.into()` (user type), or a `map`/`into_iter().map()`
/// recursing through `Option`/`Vec`. Symmetric — works generated→author and
/// author→generated, since `From` is generated both ways. Public so the macro's
/// adapter reuses the exact same boundary conversions.
pub fn conv_expr(access: &str, ty: &Type, known: &BTreeSet<String>) -> String {
    if !contains_user_type(ty, known) {
        return access.to_string();
    }
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            let ident = seg.ident.to_string();
            if let Some(inner) = single_generic(seg) {
                match ident.as_str() {
                    "Vec" => {
                        return format!(
                            "{access}.into_iter().map(|w| {}).collect()",
                            conv_expr("w", inner, known)
                        );
                    }
                    "Option" => {
                        return format!("{access}.map(|w| {})", conv_expr("w", inner, known));
                    }
                    _ => {}
                }
            }
            if known.contains(&ident) {
                return format!("{access}.into()");
            }
        }
    }
    access.to_string()
}

/// Whether `ty` is, or contains (through `Vec`/`Option`/`Box`), a user type in
/// `known`. Public so the macro can classify args/returns.
pub fn contains_user_type(ty: &Type, known: &BTreeSet<String>) -> bool {
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            let ident = seg.ident.to_string();
            if known.contains(&ident) {
                return true;
            }
            if matches!(ident.as_str(), "Vec" | "Option" | "Box") {
                if let Some(inner) = single_generic(seg) {
                    return contains_user_type(inner, known);
                }
            }
        }
    }
    false
}

fn single_generic(seg: &syn::PathSegment) -> Option<&Type> {
    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
        for a in &ab.args {
            if let syn::GenericArgument::Type(t) = a {
                return Some(t);
            }
        }
    }
    None
}

/// Does `attrs` contain `#[<name>(...)]` / `#[<path>::<name>]` (last segment match)?
fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|a| {
        a.path()
            .segments
            .last()
            .map(|s| s.ident == name)
            .unwrap_or(false)
    })
}

/// Does `attrs` contain a `#[derive(... <name> ...)]`?
fn has_derive(attrs: &[syn::Attribute], name: &str) -> bool {
    for a in attrs {
        if !a.path().is_ident("derive") {
            continue;
        }
        let mut found = false;
        let _ = a.parse_nested_meta(|m| {
            if m.path
                .segments
                .last()
                .map(|s| s.ident == name)
                .unwrap_or(false)
            {
                found = true;
            }
            Ok(())
        });
        if found {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = r#"
        #[derive(WitType)]
        pub struct Point { pub x: i32, pub y: i32 }

        #[derive(WitType)]
        pub enum Shape { Circle(u32), Rect(Point), Dot }

        #[plugin_interface(version = 1, crate = "fidius_guest")]
        pub trait Geo: Send + Sync {
            fn midpoint(&self, a: Point, b: Point) -> Point;
            fn classify(&self, pts: Vec<Point>) -> Shape;
            fn name(&self, s: Shape) -> String;
        }
    "#;

    #[test]
    fn generates_wit_with_records_variants_and_funcs() {
        let g = generate(SRC).unwrap();
        assert_eq!(g.interface_name, "Geo");
        assert_eq!(g.iface_kebab, "geo");
        assert!(g.user_types.contains(&"Point".to_string()));
        assert!(g.user_types.contains(&"Shape".to_string()));

        assert!(g.wit.contains("record point {"));
        assert!(g.wit.contains("variant shape {"));
        assert!(g.wit.contains("rect(point),"));
        assert!(g
            .wit
            .contains("midpoint: func(a: point, b: point) -> point;"));
        assert!(g.wit.contains("classify: func(pts: list<point>) -> shape;"));
        assert!(g.wit.contains("name: func(s: shape) -> string;"));
        assert!(g.wit.contains("fidius-interface-hash: func() -> u64;"));
    }

    #[test]
    fn generates_conversions_both_ways() {
        let g = generate(SRC).unwrap();
        let c = &g.conversions;
        // struct record conversions
        assert!(c.contains("From<exports::fidius::geo::geo::Point> for crate::Point"));
        assert!(c.contains("From<crate::Point> for exports::fidius::geo::geo::Point"));
        // enum variant conversions with nested .into() for Rect(Point)
        assert!(c.contains("From<exports::fidius::geo::geo::Shape> for crate::Shape"));
        assert!(
            c.contains("Rect(x) => crate :: Shape :: Rect")
                || c.contains("Rect(x) => crate::Shape::Rect")
        );
        assert!(c.contains(".into()"));
    }

    #[test]
    fn primitive_only_interface_has_no_conversions() {
        let src = r#"
            #[plugin_interface(version = 1)]
            pub trait Greeter { fn greet(&self, name: String) -> String; }
        "#;
        let g = generate(src).unwrap();
        assert!(g.user_types.is_empty());
        assert!(g.conversions.is_empty());
        assert!(g.wit.contains("greet: func(name: string) -> string;"));
    }

    #[test]
    fn unsupported_type_errors() {
        let src = r#"
            #[plugin_interface(version = 1)]
            pub trait T { fn f(&self, x: std::collections::HashMap<String,String>) -> u32; }
        "#;
        assert!(generate(src).is_err());
    }
}
