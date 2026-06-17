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

//! WIT generation for the Fidius WASM backend (FIDIUS-I-0023).
//!
//! Maps the Rust types in a `#[plugin_interface]` to WIT, per
//! `docs/explanation/wasm-component-abi.md`. Primitives, `String`, `Vec<T>`,
//! `Option<T>`, and `Result<T, PluginError>` map directly; user types that
//! carry `#[derive(WitType)]` map to WIT `record`s (structs) and `variant`s
//! (enums), referenced by their kebab-case name.
//!
//! This is a plain library (not a proc-macro), so `fidius-macro`, the `build.rs`
//! helper, and the `fidius wit` CLI can all share one implementation.

use std::collections::BTreeSet;

use syn::{Fields, GenericArgument, ItemEnum, ItemStruct, PathArguments, Type};

mod generate;
pub use generate::{contains_user_type, conv_expr, generate, Generated};

/// Convert a Rust identifier (CamelCase or snake_case) to kebab-case, the WIT
/// naming convention. `BytePipe` → `byte-pipe`, `echo_bytes` → `echo-bytes`.
pub fn to_kebab_case(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch == '_' {
            out.push('-');
        } else if ch.is_uppercase() {
            if i != 0 {
                out.push('-');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

/// Extract the `T` from `Result<T, _>`, if `ty` is a `Result`.
pub fn result_ok_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            if seg.ident == "Result" {
                return first_generic(seg);
            }
        }
    }
    None
}

/// One method projected to WIT (already-mapped strings).
pub struct WitMethod {
    /// kebab-case export name.
    pub name: String,
    /// `(param-name, wit-type)` pairs, in order.
    pub params: Vec<(String, String)>,
    /// WIT return type, or `None` for no return.
    pub ret: Option<String>,
}

/// Map a Rust argument/return type to its WIT spelling, where `known` holds the
/// Rust identifiers of `#[derive(WitType)]` user types (mapped to their
/// kebab-case record/variant name). Returns `Err(msg)` for unsupported types.
pub fn wit_type_with(ty: &Type, known: &BTreeSet<String>) -> Result<String, String> {
    match ty {
        // Peel references: `&str` → string, `&[u8]` → list<u8>, `&T` → wit(T).
        Type::Reference(r) => {
            if let Type::Path(p) = r.elem.as_ref() {
                if path_is(p, "str") {
                    return Ok("string".to_string());
                }
            }
            if let Type::Slice(s) = r.elem.as_ref() {
                let inner = wit_type_with(&s.elem, known)?;
                return Ok(format!("list<{inner}>"));
            }
            wit_type_with(&r.elem, known)
        }
        Type::Path(p) => {
            let seg = p
                .path
                .segments
                .last()
                .ok_or_else(|| "empty type path".to_string())?;
            let ident = seg.ident.to_string();
            match ident.as_str() {
                "bool" => Ok("bool".into()),
                "i8" => Ok("s8".into()),
                "i16" => Ok("s16".into()),
                "i32" => Ok("s32".into()),
                "i64" => Ok("s64".into()),
                "u8" => Ok("u8".into()),
                "u16" => Ok("u16".into()),
                "u32" => Ok("u32".into()),
                "u64" => Ok("u64".into()),
                "f32" => Ok("f32".into()),
                "f64" => Ok("f64".into()),
                "char" => Ok("char".into()),
                "String" => Ok("string".into()),
                "Vec" => {
                    let inner = single_generic(seg, "Vec")?;
                    Ok(format!("list<{}>", wit_type_with(inner, known)?))
                }
                "Option" => {
                    let inner = single_generic(seg, "Option")?;
                    Ok(format!("option<{}>", wit_type_with(inner, known)?))
                }
                // A user type with `#[derive(WitType)]` → its kebab record/variant name.
                other if known.contains(other) => Ok(to_kebab_case(other)),
                other => Err(format!(
                    "type `{other}` is not supported in a WASM fidius interface \
                     (supported: bool, i8..i64, u8..u64, f32/f64, char, String, Vec<T>, \
                     Option<T>, Result<T, PluginError>, and #[derive(WitType)] structs/enums)"
                )),
            }
        }
        Type::Tuple(t) if t.elems.is_empty() => {
            Err("unit `()` is not a valid argument type".to_string())
        }
        _ => Err("unsupported type in a WASM fidius interface".to_string()),
    }
}

/// Primitive/std-only mapping (no user types) — the form `fidius-macro` uses for
/// the descriptor/method tables.
pub fn rust_type_to_wit(ty: &Type) -> Result<String, String> {
    wit_type_with(ty, &BTreeSet::new())
}

/// Map a method's return type to an optional WIT return, with user types in
/// `known`. `Result<T, PluginError>` → `result<T, plugin-error>` (or
/// `result<_, plugin-error>` for `T = ()`); `()`/none → no return.
pub fn return_to_wit_with(
    ret: Option<&Type>,
    known: &BTreeSet<String>,
) -> Result<Option<String>, String> {
    let Some(ty) = ret else { return Ok(None) };
    if is_unit(ty) {
        return Ok(None);
    }
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            if seg.ident == "Result" {
                let ok = first_generic(seg).ok_or_else(|| "Result needs type args".to_string())?;
                let ok_wit = if is_unit(ok) {
                    "_".to_string()
                } else {
                    wit_type_with(ok, known)?
                };
                return Ok(Some(format!("result<{ok_wit}, plugin-error>")));
            }
        }
    }
    Ok(Some(wit_type_with(ty, known)?))
}

/// Primitive/std-only return mapping (no user types).
pub fn return_to_wit(ret: Option<&Type>) -> Result<Option<String>, String> {
    return_to_wit_with(ret, &BTreeSet::new())
}

/// Render a `record <name> { ... }` WIT block from a Rust struct (named fields
/// only). Field types are mapped with `known` so they may reference other user
/// types.
pub fn struct_to_record(item: &ItemStruct, known: &BTreeSet<String>) -> Result<String, String> {
    let name = to_kebab_case(&item.ident.to_string());
    let Fields::Named(fields) = &item.fields else {
        return Err(format!(
            "WitType struct `{}` must have named fields (tuple/unit structs are not supported)",
            item.ident
        ));
    };
    let mut out = format!("    record {name} {{\n");
    for f in &fields.named {
        let fname = to_kebab_case(&f.ident.as_ref().unwrap().to_string());
        let fty = wit_type_with(&f.ty, known)
            .map_err(|e| format!("field `{}` of `{}`: {e}", fname, item.ident))?;
        out.push_str(&format!("        {fname}: {fty},\n"));
    }
    out.push_str("    }\n");
    Ok(out)
}

/// Render a `variant <name> { ... }` WIT block from a Rust enum. Each case is a
/// unit variant (`case`) or a single-field tuple variant (`case(type)`).
pub fn enum_to_variant(item: &ItemEnum, known: &BTreeSet<String>) -> Result<String, String> {
    let name = to_kebab_case(&item.ident.to_string());
    let mut out = format!("    variant {name} {{\n");
    for v in &item.variants {
        let case = to_kebab_case(&v.ident.to_string());
        match &v.fields {
            Fields::Unit => out.push_str(&format!("        {case},\n")),
            Fields::Unnamed(u) if u.unnamed.len() == 1 => {
                let payload = wit_type_with(&u.unnamed[0].ty, known)
                    .map_err(|e| format!("variant `{}::{}`: {e}", item.ident, v.ident))?;
                out.push_str(&format!("        {case}({payload}),\n"));
            }
            _ => {
                return Err(format!(
                    "WitType enum `{}` variant `{}` must be a unit or single-field \
                     variant (WIT `variant` cases carry at most one payload)",
                    item.ident, v.ident
                ));
            }
        }
    }
    out.push_str("    }\n");
    Ok(out)
}

/// Render a complete `.wit` document: package + interface (the `plugin-error`
/// record, any user `type_defs` (records/variants), the funcs, and the
/// `fidius-interface-hash` carrier) + the `<iface>-plugin` world. `type_defs`
/// are pre-rendered (see [`struct_to_record`] / [`enum_to_variant`]).
pub fn render_wit_full(iface_kebab: &str, type_defs: &[String], methods: &[WitMethod]) -> String {
    let mut s = String::new();
    s.push_str(&format!("package fidius:{iface_kebab}@0.1.0;\n\n"));
    s.push_str(&format!("interface {iface_kebab} {{\n"));
    s.push_str("    record plugin-error {\n");
    s.push_str("        code: string,\n");
    s.push_str("        message: string,\n");
    s.push_str("        details: option<string>,\n");
    s.push_str("    }\n");
    for def in type_defs {
        s.push_str(def);
    }
    for m in methods {
        let params = m
            .params
            .iter()
            .map(|(n, t)| format!("{n}: {t}"))
            .collect::<Vec<_>>()
            .join(", ");
        match &m.ret {
            Some(r) => s.push_str(&format!("    {}: func({params}) -> {r};\n", m.name)),
            None => s.push_str(&format!("    {}: func({params});\n", m.name)),
        }
    }
    s.push_str("    fidius-interface-hash: func() -> u64;\n");
    s.push_str("}\n\n");
    s.push_str(&format!(
        "world {iface_kebab}-plugin {{\n    export {iface_kebab};\n}}\n"
    ));
    s
}

/// Convenience: render a WIT document with no user type defs (the primitives-only
/// form `fidius-macro` emits inline today).
pub fn render_wit(iface_kebab: &str, methods: &[WitMethod]) -> String {
    render_wit_full(iface_kebab, &[], methods)
}

// ── helpers ─────────────────────────────────────────────────────────────────

fn is_unit(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(t) if t.elems.is_empty())
}

fn path_is(p: &syn::TypePath, name: &str) -> bool {
    p.path
        .segments
        .last()
        .map(|s| s.ident == name)
        .unwrap_or(false)
}

fn single_generic<'a>(seg: &'a syn::PathSegment, what: &str) -> Result<&'a Type, String> {
    first_generic(seg).ok_or_else(|| format!("`{what}` needs one type argument"))
}

fn first_generic(seg: &syn::PathSegment) -> Option<&Type> {
    if let PathArguments::AngleBracketed(ab) = &seg.arguments {
        for a in &ab.args {
            if let GenericArgument::Type(t) = a {
                return Some(t);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn known(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }
    fn wit(s: &str) -> String {
        rust_type_to_wit(&syn::parse_str::<Type>(s).unwrap()).unwrap()
    }

    #[test]
    fn primitives_strings_containers() {
        assert_eq!(wit("i64"), "s64");
        assert_eq!(wit("u32"), "u32");
        assert_eq!(wit("String"), "string");
        assert_eq!(wit("& str"), "string");
        assert_eq!(wit("Vec<u8>"), "list<u8>");
        assert_eq!(wit("Option<i32>"), "option<s32>");
        assert_eq!(wit("&[u8]"), "list<u8>");
    }

    #[test]
    fn returns() {
        let ret = |s: &str| return_to_wit(Some(&syn::parse_str::<Type>(s).unwrap())).unwrap();
        assert_eq!(ret("()"), None);
        assert_eq!(
            ret("Result<i64, PluginError>"),
            Some("result<s64, plugin-error>".into())
        );
        assert_eq!(
            ret("Result<(), PluginError>"),
            Some("result<_, plugin-error>".into())
        );
    }

    #[test]
    fn user_types_need_the_known_set() {
        let ty: Type = syn::parse_str("Point").unwrap();
        assert!(rust_type_to_wit(&ty).is_err()); // unknown without the set
        let k = known(&["Point"]);
        assert_eq!(wit_type_with(&ty, &k).unwrap(), "point");
        // nested in containers
        let v: Type = syn::parse_str("Vec<Point>").unwrap();
        assert_eq!(wit_type_with(&v, &k).unwrap(), "list<point>");
        let o: Type = syn::parse_str("Option<BytePipe>").unwrap();
        assert_eq!(
            wit_type_with(&o, &known(&["BytePipe"])).unwrap(),
            "option<byte-pipe>"
        );
    }

    #[test]
    fn struct_renders_to_record() {
        let item: ItemStruct = syn::parse_str("struct Point { x: i32, y_pos: u64 }").unwrap();
        let rec = struct_to_record(&item, &BTreeSet::new()).unwrap();
        assert!(rec.contains("record point {"));
        assert!(rec.contains("x: s32,"));
        assert!(rec.contains("y-pos: u64,"));
    }

    #[test]
    fn struct_with_nested_user_type() {
        let item: ItemStruct = syn::parse_str("struct Line { from: Point, to: Point }").unwrap();
        let rec = struct_to_record(&item, &known(&["Point"])).unwrap();
        assert!(rec.contains("from: point,"));
        assert!(rec.contains("to: point,"));
    }

    #[test]
    fn enum_renders_to_variant() {
        let item: ItemEnum =
            syn::parse_str("enum Shape { Circle(u32), Rect(Point), Dot }").unwrap();
        let var = enum_to_variant(&item, &known(&["Point"])).unwrap();
        assert!(var.contains("variant shape {"));
        assert!(var.contains("circle(u32),"));
        assert!(var.contains("rect(point),"));
        assert!(var.contains("dot,"));
    }

    #[test]
    fn enum_multifield_variant_is_error() {
        let item: ItemEnum = syn::parse_str("enum E { Pair(u32, u32) }").unwrap();
        assert!(enum_to_variant(&item, &BTreeSet::new()).is_err());
    }

    #[test]
    fn full_document_places_type_defs_before_funcs() {
        let recs = vec![struct_to_record(
            &syn::parse_str("struct Point { x: i32, y: i32 }").unwrap(),
            &BTreeSet::new(),
        )
        .unwrap()];
        let methods = vec![WitMethod {
            name: "midpoint".into(),
            params: vec![("a".into(), "point".into()), ("b".into(), "point".into())],
            ret: Some("point".into()),
        }];
        let doc = render_wit_full("geo", &recs, &methods);
        assert!(doc.contains("package fidius:geo@0.1.0;"));
        let rec_at = doc.find("record point {").unwrap();
        let fn_at = doc.find("midpoint: func").unwrap();
        assert!(
            rec_at < fn_at,
            "records must precede funcs in the interface"
        );
        assert!(doc.contains("midpoint: func(a: point, b: point) -> point;"));
        assert!(doc.contains("fidius-interface-hash: func() -> u64;"));
        assert!(doc.contains("world geo-plugin {"));
    }
}
