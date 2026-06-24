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

//! FIDIUS-I-0033 Phase 4: property test for the Rust→WIT type mapping.
//!
//! The mapping has no inverse, so the "round-trip" is model-driven: a generated
//! `TyModel` produces BOTH a Rust type-source string and the expected WIT string;
//! parsing the Rust source and running `rust_type_to_wit` must reproduce the WIT.
//! This validates the mapping is structurally faithful over arbitrary nestings of
//! the supported types (primitives, `Vec`, `Option`, maps, tuples).

use proptest::prelude::*;

/// A type the WASM interface mapping supports. Knows both how to render itself as
/// Rust source and what WIT it must map to — built from the same node so they can't
/// drift.
#[derive(Clone, Debug)]
enum TyModel {
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Char,
    Str,
    Vec(Box<TyModel>),
    Opt(Box<TyModel>),
    Map(Box<TyModel>, Box<TyModel>),
    Tuple(Vec<TyModel>),
}

impl TyModel {
    fn rust(&self) -> String {
        match self {
            TyModel::Bool => "bool".into(),
            TyModel::I8 => "i8".into(),
            TyModel::I16 => "i16".into(),
            TyModel::I32 => "i32".into(),
            TyModel::I64 => "i64".into(),
            TyModel::U8 => "u8".into(),
            TyModel::U16 => "u16".into(),
            TyModel::U32 => "u32".into(),
            TyModel::U64 => "u64".into(),
            TyModel::F32 => "f32".into(),
            TyModel::F64 => "f64".into(),
            TyModel::Char => "char".into(),
            TyModel::Str => "String".into(),
            TyModel::Vec(t) => format!("Vec<{}>", t.rust()),
            TyModel::Opt(t) => format!("Option<{}>", t.rust()),
            TyModel::Map(k, v) => format!("HashMap<{}, {}>", k.rust(), v.rust()),
            TyModel::Tuple(es) if es.len() == 1 => format!("({},)", es[0].rust()),
            TyModel::Tuple(es) => {
                let inner: Vec<_> = es.iter().map(TyModel::rust).collect();
                format!("({})", inner.join(", "))
            }
        }
    }

    fn wit(&self) -> String {
        match self {
            TyModel::Bool => "bool".into(),
            TyModel::I8 => "s8".into(),
            TyModel::I16 => "s16".into(),
            TyModel::I32 => "s32".into(),
            TyModel::I64 => "s64".into(),
            TyModel::U8 => "u8".into(),
            TyModel::U16 => "u16".into(),
            TyModel::U32 => "u32".into(),
            TyModel::U64 => "u64".into(),
            TyModel::F32 => "f32".into(),
            TyModel::F64 => "f64".into(),
            TyModel::Char => "char".into(),
            TyModel::Str => "string".into(),
            TyModel::Vec(t) => format!("list<{}>", t.wit()),
            TyModel::Opt(t) => format!("option<{}>", t.wit()),
            TyModel::Map(k, v) => format!("list<tuple<{}, {}>>", k.wit(), v.wit()),
            TyModel::Tuple(es) => {
                let inner: Vec<_> = es.iter().map(TyModel::wit).collect();
                format!("tuple<{}>", inner.join(", "))
            }
        }
    }
}

fn arb_tymodel() -> impl Strategy<Value = TyModel> {
    let leaf = prop_oneof![
        Just(TyModel::Bool),
        Just(TyModel::I8),
        Just(TyModel::I16),
        Just(TyModel::I32),
        Just(TyModel::I64),
        Just(TyModel::U8),
        Just(TyModel::U16),
        Just(TyModel::U32),
        Just(TyModel::U64),
        Just(TyModel::F32),
        Just(TyModel::F64),
        Just(TyModel::Char),
        Just(TyModel::Str),
    ];
    leaf.prop_recursive(4, 24, 4, |inner| {
        prop_oneof![
            inner.clone().prop_map(|t| TyModel::Vec(Box::new(t))),
            inner.clone().prop_map(|t| TyModel::Opt(Box::new(t))),
            (inner.clone(), inner.clone())
                .prop_map(|(k, v)| TyModel::Map(Box::new(k), Box::new(v))),
            // Non-empty tuples only — unit `()` is an explicit error in the mapping.
            prop::collection::vec(inner.clone(), 1..4).prop_map(TyModel::Tuple),
        ]
    })
}

proptest! {
    #[test]
    fn rust_type_maps_to_expected_wit(m in arb_tymodel()) {
        let src = m.rust();
        let ty: syn::Type = syn::parse_str(&src)
            .unwrap_or_else(|e| panic!("failed to parse generated type `{src}`: {e}"));
        let got = fidius_wit::rust_type_to_wit(&ty)
            .unwrap_or_else(|e| panic!("mapping `{src}` failed: {e}"));
        prop_assert_eq!(got, m.wit(), "Rust→WIT mapping diverged for `{}`", src);
    }
}
