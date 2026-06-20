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
    enum_to_wit, return_to_wit_with, struct_to_record, to_kebab_case, wit_type_with, WitMethod,
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

/// Generate WIT + conversions from a crate's source string (`lib.rs`). Inline
/// modules (`mod m { .. }`) are walked; external `mod m;` files cannot be read
/// from a bare string — use [`generate_from_path`] (the `build.rs` helper does).
pub fn generate(src: &str) -> Result<Generated, String> {
    let file = syn::parse_file(src).map_err(|e| format!("parse error: {e}"))?;
    let mut acc = Collected::default();
    collect(&file.items, &[], None, &mut acc)?;
    assemble(acc)
}

/// Like [`generate`], but reads `lib_rs` and follows external `mod m;` files
/// (resolving `m.rs` / `m/mod.rs`), so `#[derive(WitType)]` types and the
/// `#[plugin_interface]` trait may live in submodules.
pub fn generate_from_path(lib_rs: &std::path::Path) -> Result<Generated, String> {
    let src = std::fs::read_to_string(lib_rs)
        .map_err(|e| format!("reading {}: {e}", lib_rs.display()))?;
    let file = syn::parse_file(&src).map_err(|e| format!("parse {}: {e}", lib_rs.display()))?;
    let dir = lib_rs.parent().unwrap_or_else(|| std::path::Path::new("."));
    let mut acc = Collected::default();
    collect(&file.items, &[], Some(dir), &mut acc)?;
    assemble(acc)
}

/// `#[derive(WitType)]` types (tagged with their Rust module path) + the
/// `#[plugin_interface]` trait, gathered across the module tree.
#[derive(Default)]
struct Collected {
    structs: Vec<(Vec<String>, syn::ItemStruct)>,
    enums: Vec<(Vec<String>, syn::ItemEnum)>,
    the_trait: Option<syn::ItemTrait>,
}

/// Recursively gather items, descending into inline `mod m { .. }` and (when
/// `dir` is `Some`) external `mod m;` files (`m.rs` / `m/mod.rs`).
fn collect(
    items: &[Item],
    mod_path: &[String],
    dir: Option<&std::path::Path>,
    acc: &mut Collected,
) -> Result<(), String> {
    for item in items {
        match item {
            Item::Struct(s) if has_derive(&s.attrs, "WitType") => {
                acc.structs.push((mod_path.to_vec(), s.clone()));
            }
            Item::Enum(e) if has_derive(&e.attrs, "WitType") => {
                acc.enums.push((mod_path.to_vec(), e.clone()));
            }
            Item::Trait(t) if has_attr(&t.attrs, "plugin_interface") => {
                if acc.the_trait.is_some() {
                    return Err("multiple #[plugin_interface] traits found".into());
                }
                acc.the_trait = Some(t.clone());
            }
            Item::Mod(m) => {
                let mut child = mod_path.to_vec();
                child.push(m.ident.to_string());
                if let Some((_, items)) = &m.content {
                    let sub = dir.map(|d| d.join(m.ident.to_string()));
                    collect(items, &child, sub.as_deref(), acc)?;
                } else if let Some(d) = dir {
                    let name = m.ident.to_string();
                    let candidates = [d.join(format!("{name}.rs")), d.join(&name).join("mod.rs")];
                    let file = candidates.iter().find(|p| p.exists()).ok_or_else(|| {
                        format!(
                            "cannot find module file for `mod {name};` near {}",
                            d.display()
                        )
                    })?;
                    let src = std::fs::read_to_string(file)
                        .map_err(|e| format!("reading {}: {e}", file.display()))?;
                    let parsed = syn::parse_file(&src)
                        .map_err(|e| format!("parse {}: {e}", file.display()))?;
                    collect(&parsed.items, &child, Some(&d.join(&name)), acc)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Build the `.wit` + conversions from the collected items.
fn assemble(acc: Collected) -> Result<Generated, String> {
    let the_trait = acc
        .the_trait
        .ok_or("no #[plugin_interface] trait found in source")?;
    let interface_name = the_trait.ident.to_string();
    let iface_kebab = to_kebab_case(&interface_name);

    let mut user_types: Vec<String> = Vec::new();
    user_types.extend(acc.structs.iter().map(|(_, s)| s.ident.to_string()));
    user_types.extend(acc.enums.iter().map(|(_, e)| e.ident.to_string()));
    let known: BTreeSet<String> = user_types.iter().cloned().collect();

    // Type defs: records (struct records + synthetic struct-variant payload
    // records) before variants, so forward references resolve cleanly.
    let mut type_defs: Vec<String> = Vec::new();
    let mut variant_defs: Vec<String> = Vec::new();
    for (_, s) in &acc.structs {
        type_defs.push(struct_to_record(s, &known)?);
    }
    for (_, e) in &acc.enums {
        let (synthetic, variant) = enum_to_wit(e, &known)?;
        type_defs.extend(synthetic);
        variant_defs.push(variant);
    }
    type_defs.extend(variant_defs);

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
        // Server-streaming (`-> fidius::Stream<T>`): the func renders as a
        // resource-returning export, not a value return (FIDIUS-I-0026).
        let stream_item = ret_ty.and_then(crate::stream_item_type);
        let (ret, stream_item) = match stream_item {
            Some(item_ty) => {
                let item_wit = wit_type_with(item_ty, &known)
                    .map_err(|e| format!("method `{}` stream item: {e}", f.sig.ident))?;
                (None, Some(item_wit))
            }
            None => {
                let ret = return_to_wit_with(ret_ty, &known)
                    .map_err(|e| format!("method `{}` return: {e}", f.sig.ident))?;
                (ret, None)
            }
        };
        methods.push(WitMethod {
            name: to_kebab_case(&f.sig.ident.to_string()),
            params,
            ret,
            stream_item,
        });
    }

    let wit = crate::render_wit_full(&iface_kebab, &type_defs, &methods);
    let conversions = render_conversions(&iface_kebab, &acc.structs, &acc.enums, &known);

    Ok(Generated {
        interface_name,
        iface_kebab,
        user_types,
        wit,
        conversions,
    })
}

/// `crate::<mod::path>::<Name>` — the author-side path for a type at `mod_path`.
fn author_path(mod_path: &[String], name: &str) -> String {
    if mod_path.is_empty() {
        format!("crate::{name}")
    } else {
        format!("crate::{}::{name}", mod_path.join("::"))
    }
}

/// Render `From` impls (both directions) between each user type and its
/// wit-bindgen-generated mirror. Emitted into the adapter module, where the
/// generated types live (flat) at `exports::fidius::<iface>::<iface>::<Type>`
/// and the author types at `crate::<mod::path>::<Type>`.
fn render_conversions(
    iface_kebab: &str,
    structs: &[(Vec<String>, syn::ItemStruct)],
    enums: &[(Vec<String>, syn::ItemEnum)],
    known: &BTreeSet<String>,
) -> String {
    if structs.is_empty() && enums.is_empty() {
        return String::new();
    }
    let snake = iface_kebab.replace('-', "_");
    let gen_path = format!("exports::fidius::{snake}::{snake}");
    let mut out = String::new();
    out.push_str("// Generated by fidius-wit: author <-> wit-bindgen conversions.\n");

    for (path, s) in structs {
        let name = s.ident.to_string();
        let g = format!("{gen_path}::{name}");
        let a = author_path(path, &name);
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

    for (path, e) in enums {
        let ename = e.ident.to_string();
        let g = format!("{gen_path}::{ename}");
        let a = author_path(path, &ename);

        let mut g2a = Vec::new(); // generated -> author
        let mut a2g = Vec::new(); // author -> generated
        for v in &e.variants {
            let case = v.ident.to_string();
            match &v.fields {
                syn::Fields::Unit => {
                    g2a.push(format!("{g}::{case} => {a}::{case}"));
                    a2g.push(format!("{a}::{case} => {g}::{case}"));
                }
                syn::Fields::Unnamed(u) if u.unnamed.len() == 1 => {
                    let ty = &u.unnamed[0].ty;
                    g2a.push(format!(
                        "{g}::{case}(x) => {a}::{case}({})",
                        conv_expr("x", ty, known)
                    ));
                    a2g.push(format!(
                        "{a}::{case}(x) => {g}::{case}({})",
                        conv_expr("x", ty, known)
                    ));
                }
                syn::Fields::Named(f) => {
                    // The gen side wraps the case payload in a synthesized record
                    // (`<Enum><Case>`, e.g. `ShapeRect`); the author side is a
                    // struct variant with inline fields.
                    let gen_rec = format!("{gen_path}::{ename}{case}");
                    let fnames: Vec<String> = f
                        .named
                        .iter()
                        .map(|fl| fl.ident.as_ref().unwrap().to_string())
                        .collect();
                    let g2a_inits = fnames
                        .iter()
                        .zip(f.named.iter())
                        .map(|(n, fl)| {
                            format!("{n}: {}", conv_expr(&format!("__r.{n}"), &fl.ty, known))
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    g2a.push(format!("{g}::{case}(__r) => {a}::{case} {{ {g2a_inits} }}"));

                    let binds = fnames.join(", ");
                    let a2g_inits = fnames
                        .iter()
                        .zip(f.named.iter())
                        .map(|(n, fl)| format!("{n}: {}", conv_expr(n, &fl.ty, known)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    a2g.push(format!(
                        "{a}::{case} {{ {binds} }} => {g}::{case}({gen_rec} {{ {a2g_inits} }})"
                    ));
                }
                // Multi-field tuple cases are rejected by `enum_to_wit`.
                syn::Fields::Unnamed(_) => unreachable!("rejected by enum_to_wit"),
            }
        }
        out.push_str(&format!(
            "impl ::core::convert::From<{g}> for {a} {{ fn from(v: {g}) -> Self {{ match v {{ {} }} }} }}\n",
            g2a.join(", ")
        ));
        out.push_str(&format!(
            "impl ::core::convert::From<{a}> for {g} {{ fn from(v: {a}) -> Self {{ match v {{ {} }} }} }}\n",
            a2g.join(", ")
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
    // Maps cross as `list<tuple<k, v>>` — a `Vec<(K, V)>` binding. Convert with
    // `collect()` (bidirectional via type inference: `Vec`↔`HashMap`/`BTreeMap`).
    // Handle before the user-type short-circuit so a `HashMap<String, u32>` (no
    // `#[derive(WitType)]` inside) still converts to/from its binding.
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            if matches!(seg.ident.to_string().as_str(), "HashMap" | "BTreeMap") {
                if let Some((k, v)) = two_generics(seg) {
                    let kc = conv_expr("k", k, known);
                    let vc = conv_expr("v", v, known);
                    if kc == "k" && vc == "v" {
                        return format!("{access}.into_iter().collect()");
                    }
                    return format!("{access}.into_iter().map(|(k, v)| ({kc}, {vc})).collect()");
                }
            }
        }
    }
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
            if matches!(ident.as_str(), "HashMap" | "BTreeMap") {
                if let Some((k, v)) = two_generics(seg) {
                    return contains_user_type(k, known) || contains_user_type(v, known);
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

fn two_generics(seg: &syn::PathSegment) -> Option<(&Type, &Type)> {
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
        // `Box<T>` has no WIT projection (maps/tuples are now supported, so use a
        // type that genuinely isn't).
        let src = r#"
            #[plugin_interface(version = 1)]
            pub trait T { fn f(&self, x: Box<String>) -> u32; }
        "#;
        assert!(generate(src).is_err());
    }
}
