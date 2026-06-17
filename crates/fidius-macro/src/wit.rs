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

//! WIT generation for the WASM component target (FIDIUS-I-0021 Phase 3).
//!
//! Maps the Rust types in a `#[plugin_interface]`/`#[plugin_impl]` method to
//! WIT, per the mapping in `docs/explanation/wasm-component-abi.md`, and renders
//! the `.wit` text that `#[plugin_impl]` feeds to `wit_bindgen::generate!`.
//!
//! **Scope (v1):** a proc-macro can't introspect external `struct`/`enum`
//! definitions (it only sees the method signatures), so the supported type set
//! is the WIT-expressible primitives plus `String`, `Vec<u8>`→`list<u8>`,
//! `Vec<T>`→`list<T>`, `Option<T>`, and `Result<T, PluginError>`. User
//! records/variants need a future `#[derive(WitType)]`; an unsupported type is
//! a clear compile error rather than silently-wrong WIT.

use syn::{GenericArgument, PathArguments, Type};

/// Convert a Rust identifier (CamelCase or snake_case) to kebab-case, the WIT
/// naming convention. `BytePipe` → `byte-pipe`, `echo_bytes` → `echo-bytes`.
pub(crate) fn to_kebab_case(s: &str) -> String {
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
pub(crate) fn result_ok_type(ty: &Type) -> Option<&Type> {
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
pub(crate) struct WitMethod {
    /// kebab-case export name.
    pub name: String,
    /// `(param-name, wit-type)` pairs, in order.
    pub params: Vec<(String, String)>,
    /// WIT return type, or `None` for no return.
    pub ret: Option<String>,
}

/// Render a complete `.wit` document for an interface and its methods. Mirrors
/// the hand-authored reference (`tests/wasm-fixtures/greeter/wit/world.wit`):
/// a `plugin-error` record, the interface's funcs, the `fidius-interface-hash`
/// carrier, and a `<iface>-plugin` world exporting the interface.
pub(crate) fn render_wit(iface_kebab: &str, methods: &[WitMethod]) -> String {
    let mut s = String::new();
    s.push_str(&format!("package fidius:{iface_kebab}@0.1.0;\n\n"));
    s.push_str(&format!("interface {iface_kebab} {{\n"));
    s.push_str("    record plugin-error {\n");
    s.push_str("        code: string,\n");
    s.push_str("        message: string,\n");
    s.push_str("        details: option<string>,\n");
    s.push_str("    }\n");
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

/// Map a Rust argument/return type to its WIT spelling. Returns `Err(msg)` for
/// types outside the supported set.
pub(crate) fn rust_type_to_wit(ty: &Type) -> Result<String, String> {
    match ty {
        // Peel references: `&str` → string, `&[u8]` → list<u8>, `&T` → wit(T).
        Type::Reference(r) => {
            // `&str`
            if let Type::Path(p) = r.elem.as_ref() {
                if path_is(p, "str") {
                    return Ok("string".to_string());
                }
            }
            // `&[u8]` / `&[T]`
            if let Type::Slice(s) = r.elem.as_ref() {
                let inner = rust_type_to_wit(&s.elem)?;
                return Ok(format!("list<{inner}>"));
            }
            rust_type_to_wit(&r.elem)
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
                    let it = rust_type_to_wit(inner)?;
                    Ok(format!("list<{it}>"))
                }
                "Option" => {
                    let inner = single_generic(seg, "Option")?;
                    let it = rust_type_to_wit(inner)?;
                    Ok(format!("option<{it}>"))
                }
                other => Err(format!(
                    "type `{other}` is not supported in a WASM fidius interface yet \
                     (supported: bool, i8..i64, u8..u64, f32/f64, char, String, \
                     Vec<T>, Option<T>, and Result<T, PluginError> as a return; \
                     user structs/enums need a future #[derive(WitType)])"
                )),
            }
        }
        // The unit type `()` only appears as a return and is handled by
        // `return_to_wit`; as an argument it's unsupported.
        Type::Tuple(t) if t.elems.is_empty() => {
            Err("unit `()` is not a valid argument type".to_string())
        }
        _ => Err("unsupported type in a WASM fidius interface".to_string()),
    }
}

/// Map a method's return type to an optional WIT return. `Result<T, PluginError>`
/// → `result<T, plugin-error>` (or `result<_, plugin-error>` for `T = ()`);
/// `()`/none → no return.
pub(crate) fn return_to_wit(ret: Option<&Type>) -> Result<Option<String>, String> {
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
                    rust_type_to_wit(ok)?
                };
                return Ok(Some(format!("result<{ok_wit}, plugin-error>")));
            }
        }
    }
    Ok(Some(rust_type_to_wit(ty)?))
}

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

    fn wit(s: &str) -> String {
        rust_type_to_wit(&syn::parse_str::<Type>(s).unwrap()).unwrap()
    }
    fn ret(s: &str) -> Option<String> {
        return_to_wit(Some(&syn::parse_str::<Type>(s).unwrap())).unwrap()
    }

    #[test]
    fn primitives_and_strings() {
        assert_eq!(wit("bool"), "bool");
        assert_eq!(wit("i64"), "s64");
        assert_eq!(wit("u32"), "u32");
        assert_eq!(wit("f64"), "f64");
        assert_eq!(wit("char"), "char");
        assert_eq!(wit("String"), "string");
        assert_eq!(wit("& str"), "string");
    }

    #[test]
    fn containers() {
        assert_eq!(wit("Vec<u8>"), "list<u8>");
        assert_eq!(wit("Vec<String>"), "list<string>");
        assert_eq!(wit("Option<i32>"), "option<s32>");
        assert_eq!(wit("Vec<Option<u64>>"), "list<option<u64>>");
        assert_eq!(wit("&[u8]"), "list<u8>");
    }

    #[test]
    fn returns() {
        assert_eq!(ret("String"), Some("string".to_string()));
        assert_eq!(ret("()"), None);
        assert_eq!(
            ret("Result<i64, PluginError>"),
            Some("result<s64, plugin-error>".to_string())
        );
        assert_eq!(
            ret("Result<(), PluginError>"),
            Some("result<_, plugin-error>".to_string())
        );
        assert_eq!(return_to_wit(None).unwrap(), None);
    }

    #[test]
    fn unsupported_is_error() {
        assert!(rust_type_to_wit(&syn::parse_str::<Type>("MyStruct").unwrap()).is_err());
    }

    #[test]
    fn renders_greeter_like_wit() {
        let methods = vec![
            WitMethod {
                name: "greet".into(),
                params: vec![("name".into(), "string".into())],
                ret: Some("string".into()),
            },
            WitMethod {
                name: "echo-bytes".into(),
                params: vec![("data".into(), "list<u8>".into())],
                ret: Some("list<u8>".into()),
            },
        ];
        let doc = render_wit("greeter", &methods);
        assert!(doc.contains("package fidius:greeter@0.1.0;"));
        assert!(doc.contains("interface greeter {"));
        assert!(doc.contains("greet: func(name: string) -> string;"));
        assert!(doc.contains("echo-bytes: func(data: list<u8>) -> list<u8>;"));
        assert!(doc.contains("fidius-interface-hash: func() -> u64;"));
        assert!(doc.contains("world greeter-plugin {"));
        assert!(doc.contains("export greeter;"));
    }
}
