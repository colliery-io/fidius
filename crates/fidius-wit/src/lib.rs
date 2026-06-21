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
pub use generate::{contains_user_type, conv_expr, generate, generate_from_path, Generated};

/// Convert a Rust identifier (CamelCase or snake_case) to kebab-case, the WIT
/// naming convention. `BytePipe` → `byte-pipe`, `echo_bytes` → `echo-bytes`.
///
/// A leading `r#` raw-ident prefix is stripped first: it is Rust source syntax
/// for using a keyword as an identifier (`r#type`), and denotes the bare name
/// (`type`). Stripping it lets WIT keywords that are *also* Rust keywords
/// (`type`, `enum`, `use`, `static`, `as`, `async`) reach [`wit_ident`] as their
/// real name so they can be `%`-escaped. The result is the *semantic* WIT name
/// (no `%`); apply [`wit_ident`] when writing it into WIT source.
pub fn to_kebab_case(s: &str) -> String {
    let s = s.strip_prefix("r#").unwrap_or(s);
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

/// The reserved keywords of the WIT grammar (mirrors wit-parser 0.236's
/// `ast::lex` keyword table). A generated identifier that collides with one of
/// these is rejected by the WIT parser unless written with a leading `%`.
/// Kebab-cased forms are included where the keyword is itself kebab
/// (`error-context`).
///
/// Over-/under-inclusion is safe for consistency — both the host descriptor WIT
/// and the guest `emit_wit` WIT run through [`wit_ident`], so they always agree —
/// but this set is kept faithful to the parser so output stays minimal.
const WIT_KEYWORDS: &[&str] = &[
    "use",
    "type",
    "func",
    "u8",
    "u16",
    "u32",
    "u64",
    "s8",
    "s16",
    "s32",
    "s64",
    "f32",
    "f64",
    "char",
    "resource",
    "own",
    "borrow",
    "record",
    "flags",
    "variant",
    "enum",
    "bool",
    "string",
    "option",
    "result",
    "future",
    "stream",
    "error-context",
    "list",
    "as",
    "from",
    "static",
    "interface",
    "tuple",
    "world",
    "import",
    "export",
    "package",
    "constructor",
    "include",
    "with",
    "async",
];

/// Whether `ident` (an already-kebab-cased WIT identifier) is a reserved WIT
/// keyword and so must be `%`-escaped when written into WIT source.
pub fn is_wit_keyword(ident: &str) -> bool {
    WIT_KEYWORDS.contains(&ident)
}

/// Escape a generated WIT identifier so it is valid even when it collides with a
/// WIT keyword: `stream` → `%stream`, `row` → `row`.
///
/// The leading `%` is WIT *source syntax* only — `parse_id` strips it and the
/// identifier it denotes is unchanged. So this is applied **only** when
/// rendering WIT text; runtime export-name lookups and the interface hash use
/// the un-escaped name (and the hash derives from Rust signatures regardless).
/// This is the single source of truth shared by the host descriptor WIT and the
/// guest `emit_wit` output, so the two can never disagree on a keyword field.
pub fn wit_ident(ident: &str) -> String {
    if is_wit_keyword(ident) {
        format!("%{ident}")
    } else {
        ident.to_string()
    }
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
    /// For a **server-streaming** method (`-> fidius::Stream<T>`): the WIT item
    /// type `T` (FIDIUS-I-0026). When `Some`, the method renders as an exported
    /// `resource <name>-stream { next: func() -> result<option<T>, plugin-error>; }`
    /// and the func returns that resource (`-> <name>-stream`). `None` for a
    /// normal func.
    pub stream_item: Option<String>,
}

/// If `ty` is `fidius::Stream<T>` (final path segment `Stream`, exactly one type
/// argument), return `T`. The server-streaming marker (FIDIUS-I-0026, D4). Public
/// so the macro's WASM adapter shares the same detection.
pub fn stream_item_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            if seg.ident == "Stream" {
                return first_generic(seg);
            }
        }
    }
    None
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
                // Maps have no native WIT type; project to a list of key/value
                // pairs — `list<tuple<k, v>>` — which round-trips any key type
                // (not just strings). Insertion order is not preserved.
                "HashMap" | "BTreeMap" => {
                    let (k, v) = two_generics(seg, &ident)?;
                    Ok(format!(
                        "list<tuple<{}, {}>>",
                        wit_type_with(k, known)?,
                        wit_type_with(v, known)?
                    ))
                }
                // A user type with `#[derive(WitType)]` → its kebab record/variant
                // name (escaped to match its `%`-escaped declaration).
                other if known.contains(other) => Ok(wit_ident(&to_kebab_case(other))),
                other => Err(format!(
                    "type `{other}` is not supported in a WASM fidius interface \
                     (supported: bool, i8..i64, u8..u64, f32/f64, char, String, Vec<T>, \
                     Option<T>, HashMap/BTreeMap<K, V>, tuples, Result<T, PluginError>, \
                     and #[derive(WitType)] structs/enums)"
                )),
            }
        }
        Type::Tuple(t) if t.elems.is_empty() => {
            Err("unit `()` is not a valid argument type".to_string())
        }
        // Non-empty tuple `(A, B, …)` → WIT `tuple<a, b, …>`.
        Type::Tuple(t) => {
            let elems = t
                .elems
                .iter()
                .map(|e| wit_type_with(e, known))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("tuple<{}>", elems.join(", ")))
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
    let mut out = format!("    record {} {{\n", wit_ident(&name));
    for f in &fields.named {
        let fname = to_kebab_case(&f.ident.as_ref().unwrap().to_string());
        let fty = wit_type_with(&f.ty, known)
            .map_err(|e| format!("field `{}` of `{}`: {e}", fname, item.ident))?;
        out.push_str(&format!("        {}: {fty},\n", wit_ident(&fname)));
    }
    out.push_str("    }\n");
    Ok(out)
}

/// Render a Rust enum to WIT: a `variant <name> { ... }` plus any **synthetic
/// records** for struct-shaped cases. Returns `(synthetic_records, variant)`.
///
/// Case shapes: unit → `case`; single tuple field → `case(type)`; named fields
/// (`Case { .. }`) → a synthesized `record <enum>-<case>` and `case(<enum>-<case>)`.
/// A multi-field *tuple* case is rejected: WIT cases take one payload, and a
/// multi-field tuple serializes as a sequence (not a record) via serde, so it
/// can't round-trip — use a struct variant `Case { .. }` instead.
pub fn enum_to_wit(
    item: &ItemEnum,
    known: &BTreeSet<String>,
) -> Result<(Vec<String>, String), String> {
    let ename = item.ident.to_string();
    let name = to_kebab_case(&ename);
    let mut records = Vec::new();
    let mut out = format!("    variant {} {{\n", wit_ident(&name));
    for v in &item.variants {
        let case = to_kebab_case(&v.ident.to_string());
        match &v.fields {
            Fields::Unit => out.push_str(&format!("        {},\n", wit_ident(&case))),
            Fields::Unnamed(u) if u.unnamed.len() == 1 => {
                let payload = wit_type_with(&u.unnamed[0].ty, known)
                    .map_err(|e| format!("variant `{ename}::{}`: {e}", v.ident))?;
                out.push_str(&format!("        {}({payload}),\n", wit_ident(&case)));
            }
            Fields::Named(f) => {
                // Synthesize `record <enum>-<case> { .. }` for the case payload.
                // The compound `<name>-<case>` cannot collide with a keyword (it
                // always contains a `-`), so it needs no escaping — but its own
                // field names do.
                let rec_name = format!("{name}-{case}");
                let mut rec = format!("    record {rec_name} {{\n");
                for fl in &f.named {
                    let fname = to_kebab_case(&fl.ident.as_ref().unwrap().to_string());
                    let fty = wit_type_with(&fl.ty, known)
                        .map_err(|e| format!("field `{fname}` of `{ename}::{}`: {e}", v.ident))?;
                    rec.push_str(&format!("        {}: {fty},\n", wit_ident(&fname)));
                }
                rec.push_str("    }\n");
                records.push(rec);
                out.push_str(&format!("        {}({rec_name}),\n", wit_ident(&case)));
            }
            Fields::Unnamed(_) => {
                return Err(format!(
                    "WitType enum `{ename}` variant `{}` has multiple tuple fields; \
                     a WIT variant case takes one payload — use a struct variant \
                     `{} {{ .. }}` (or a single field)",
                    v.ident, v.ident
                ));
            }
        }
    }
    out.push_str("    }\n");
    Ok((records, out))
}

/// Render a complete `.wit` document: package + interface (the `plugin-error`
/// record, any user `type_defs` (records/variants), the funcs, and the
/// `fidius-interface-hash` carrier) + the `<iface>-plugin` world. `type_defs`
/// are pre-rendered (see [`struct_to_record`] / [`enum_to_variant`]).
pub fn render_wit_full(iface_kebab: &str, type_defs: &[String], methods: &[WitMethod]) -> String {
    // The interface name (derived from the trait) is itself an emitted identifier
    // — a trait named e.g. `Stream` kebabs to a keyword. Escape it everywhere it
    // appears bare (package name, interface decl, `export`); the `<iface>-plugin`
    // world name is compound and cannot collide.
    let iface = wit_ident(iface_kebab);
    let mut s = String::new();
    s.push_str(&format!("package fidius:{iface}@0.1.0;\n\n"));
    s.push_str(&format!("interface {iface} {{\n"));
    s.push_str("    record plugin-error {\n");
    s.push_str("        code: string,\n");
    s.push_str("        message: string,\n");
    s.push_str("        details: option<string>,\n");
    s.push_str("    }\n");
    for def in type_defs {
        s.push_str(def);
    }
    // Resource declarations for streaming methods, before the funcs that return
    // them (FIDIUS-I-0026): `resource <m>-stream { next: ... }`.
    for m in methods {
        if let Some(item) = &m.stream_item {
            s.push_str(&format!("    resource {}-stream {{\n", m.name));
            s.push_str(&format!(
                "        next: func() -> result<option<{item}>, plugin-error>;\n"
            ));
            s.push_str("    }\n");
        }
    }
    for m in methods {
        let params = m
            .params
            .iter()
            .map(|(n, t)| format!("{}: {t}", wit_ident(n)))
            .collect::<Vec<_>>()
            .join(", ");
        // Func name is a bare identifier (escape); the `<name>-stream` resource
        // reference is compound and cannot collide.
        let fname = wit_ident(&m.name);
        if m.stream_item.is_some() {
            // Streaming: the func returns the owned stream resource.
            s.push_str(&format!(
                "    {fname}: func({params}) -> {}-stream;\n",
                m.name
            ));
        } else {
            match &m.ret {
                Some(r) => s.push_str(&format!("    {fname}: func({params}) -> {r};\n")),
                None => s.push_str(&format!("    {fname}: func({params});\n")),
            }
        }
    }
    s.push_str("    fidius-interface-hash: func() -> u64;\n");
    // FIDIUS-A-0006 / CI.3: configured-instance constructor. The host calls it
    // once to bind config (empty bytes for a zero-config plugin); the guest sets
    // its instance. A carrier like fidius-interface-hash — not part of the
    // interface hash (which derives from the trait method signatures).
    s.push_str("    fidius-configure: func(config: list<u8>);\n");
    s.push_str("}\n\n");
    s.push_str(&format!(
        "world {iface_kebab}-plugin {{\n    export {iface};\n}}\n"
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

/// Extract the first two type arguments (e.g. the key and value of a `Map<K, V>`).
fn two_generics<'a>(seg: &'a syn::PathSegment, what: &str) -> Result<(&'a Type, &'a Type), String> {
    if let PathArguments::AngleBracketed(ab) = &seg.arguments {
        let types: Vec<&Type> = ab
            .args
            .iter()
            .filter_map(|a| match a {
                GenericArgument::Type(t) => Some(t),
                _ => None,
            })
            .collect();
        if types.len() >= 2 {
            return Ok((types[0], types[1]));
        }
    }
    Err(format!("`{what}` needs two type arguments (key, value)"))
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
    fn maps_tuples_and_nesting() {
        // Maps → list<tuple<k, v>> (any key type, not just strings).
        assert_eq!(wit("HashMap<String, i64>"), "list<tuple<string, s64>>");
        assert_eq!(wit("BTreeMap<u32, String>"), "list<tuple<u32, string>>");
        // Tuples → tuple<...>.
        assert_eq!(wit("(i32, String)"), "tuple<s32, string>");
        assert_eq!(wit("(u8, u8, bool)"), "tuple<u8, u8, bool>");
        // Nesting composes recursively.
        assert_eq!(wit("Vec<Option<i32>>"), "list<option<s32>>");
        assert_eq!(wit("Option<Vec<u8>>"), "option<list<u8>>");
        assert_eq!(
            wit("HashMap<String, Vec<i64>>"),
            "list<tuple<string, list<s64>>>"
        );
        // Map/tuple carrying a user record (kebab) via the known set.
        let k = known(&["Row"]);
        let wk = |s: &str| wit_type_with(&syn::parse_str::<Type>(s).unwrap(), &k).unwrap();
        assert_eq!(wk("Vec<Row>"), "list<row>");
        assert_eq!(wk("HashMap<String, Row>"), "list<tuple<string, row>>");
        assert_eq!(wk("(Row, i32)"), "tuple<row, s32>");
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
        let (records, var) = enum_to_wit(&item, &known(&["Point"])).unwrap();
        assert!(records.is_empty());
        assert!(var.contains("variant shape {"));
        assert!(var.contains("circle(u32),"));
        assert!(var.contains("rect(point),"));
        assert!(var.contains("dot,"));
    }

    #[test]
    fn struct_variant_synthesizes_a_record() {
        let item: ItemEnum = syn::parse_str("enum Shape { Rect { w: u32, h: u32 }, Dot }").unwrap();
        let (records, var) = enum_to_wit(&item, &BTreeSet::new()).unwrap();
        assert_eq!(records.len(), 1);
        assert!(records[0].contains("record shape-rect {"));
        assert!(records[0].contains("w: u32,"));
        assert!(records[0].contains("h: u32,"));
        assert!(var.contains("rect(shape-rect),"));
        assert!(var.contains("dot,"));
    }

    #[test]
    fn multifield_tuple_variant_is_rejected() {
        let item: ItemEnum = syn::parse_str("enum E { Pair(u32, u32) }").unwrap();
        assert!(enum_to_wit(&item, &BTreeSet::new()).is_err());
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
            stream_item: None,
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

    #[test]
    fn streaming_method_renders_a_resource() {
        let methods = vec![WitMethod {
            name: "tick".into(),
            params: vec![("count".into(), "u32".into())],
            ret: None,
            stream_item: Some("u64".into()),
        }];
        let doc = render_wit("ticker", &methods);
        // Resource declared, with the poll method...
        assert!(doc.contains("resource tick-stream {"));
        assert!(doc.contains("next: func() -> result<option<u64>, plugin-error>;"));
        // ...and the func returns the owned resource.
        assert!(doc.contains("tick: func(count: u32) -> tick-stream;"));
        // Resource precedes the func that returns it.
        assert!(doc.find("resource tick-stream").unwrap() < doc.find("tick: func").unwrap());
    }

    #[test]
    fn keyword_idents_are_escaped_others_untouched() {
        assert_eq!(wit_ident("stream"), "%stream");
        assert_eq!(wit_ident("record"), "%record");
        assert_eq!(wit_ident("from"), "%from");
        assert_eq!(wit_ident("error-context"), "%error-context");
        assert_eq!(wit_ident("row"), "row");
        assert_eq!(wit_ident("dead-letter"), "dead-letter");
        assert!(is_wit_keyword("result") && !is_wit_keyword("results"));
    }

    #[test]
    fn raw_idents_lose_their_prefix_then_escape() {
        // `r#type` is a WIT keyword that is *also* a Rust keyword.
        assert_eq!(to_kebab_case("r#type"), "type");
        assert_eq!(wit_ident(&to_kebab_case("r#type")), "%type");
        assert_eq!(wit_ident(&to_kebab_case("r#async")), "%async");
        // a raw ident that isn't a WIT keyword still drops `r#`.
        assert_eq!(to_kebab_case("r#match"), "match");
    }

    #[test]
    fn record_with_keyword_fields_escapes_field_and_record_names() {
        // `record`, `stream`, `from`, `type` are all WIT keywords.
        let item: ItemStruct =
            syn::parse_str("struct Record { record: String, stream: u64, from: bool, r#type: u8 }")
                .unwrap();
        let rec = struct_to_record(&item, &BTreeSet::new()).unwrap();
        assert!(rec.contains("record %record {"), "got: {rec}");
        assert!(rec.contains("%record: string,"));
        assert!(rec.contains("%stream: u64,"));
        assert!(rec.contains("%from: bool,"));
        assert!(rec.contains("%type: u8,"));
    }

    #[test]
    fn variant_with_keyword_cases_escapes_them() {
        let item: ItemEnum =
            syn::parse_str("enum Variant { Stream, Record(u32), List { from: u8 } }").unwrap();
        let (records, var) = enum_to_wit(&item, &BTreeSet::new()).unwrap();
        assert!(var.contains("variant %variant {"), "got: {var}");
        assert!(var.contains("%stream,"));
        assert!(var.contains("%record(u32),"));
        // struct-variant payload record is compound (`variant-list`) → not escaped,
        // but its keyword field is.
        assert!(var.contains("%list(variant-list),"));
        assert!(records[0].contains("record variant-list {"));
        assert!(records[0].contains("%from: u8,"));
    }

    #[test]
    fn keyword_user_type_reference_matches_its_declaration() {
        // A field whose *type* is a keyword-named user record must reference the
        // same `%`-escaped name the record is declared with.
        let k = known(&["Record"]);
        let ty: Type = syn::parse_str("Vec<Record>").unwrap();
        assert_eq!(wit_type_with(&ty, &k).unwrap(), "list<%record>");
    }

    #[test]
    fn stream_item_type_detects_marker() {
        let ty: Type = syn::parse_str("fidius::Stream<u64>").unwrap();
        let item = stream_item_type(&ty).unwrap();
        assert_eq!(rust_type_to_wit(item).unwrap(), "u64");
        let bare: Type = syn::parse_str("Stream<String>").unwrap();
        assert!(stream_item_type(&bare).is_some());
        let not: Type = syn::parse_str("Vec<u64>").unwrap();
        assert!(stream_item_type(&not).is_none());
    }
}
