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

//! FIDIUS-I-0033 Phase 4: property tests for the wire round-trip and the Value
//! model, over arbitrary nested data.
//!
//! **Why a concrete type, not `Value` directly:** the bincode wire is *not*
//! self-describing, and `Value`'s `Deserialize` uses `deserialize_any`, so
//! `wire::deserialize::<Value>` cannot decode — `Value` never crosses the bincode
//! wire. Concrete user types do. So the wire invariant is asserted over an
//! arbitrary concrete tree (`Concrete`), and `Value` is exercised as the *bridge*
//! it actually is: `from_value(to_value(&c)) == c`.

use fidius_guest::value::{from_value, to_value};
use fidius_guest::wire;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

/// A representative nested payload — covers the shapes that cross the FFI boundary
/// (primitives, floats, text, bytes, options, sequences, string-keyed maps,
/// tuples). Derives the *normal* serde impls, so it round-trips through bincode
/// (unlike `Value`).
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
enum Concrete {
    Bool(bool),
    Int(i64),
    Uint(u64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),
    Unit,
    Opt(Option<Box<Concrete>>),
    List(Vec<Concrete>),
    Map(Vec<(String, Concrete)>),
    Pair(Box<Concrete>, Box<Concrete>),
}

fn arb_concrete() -> impl Strategy<Value = Concrete> {
    let leaf = prop_oneof![
        any::<bool>().prop_map(Concrete::Bool),
        any::<i64>().prop_map(Concrete::Int),
        any::<u64>().prop_map(Concrete::Uint),
        // Exclude NaN: bincode preserves the bits faithfully, but NaN != NaN
        // would break the structural equality the property asserts.
        any::<f64>()
            .prop_filter("no NaN", |f| !f.is_nan())
            .prop_map(Concrete::Float),
        ".*".prop_map(Concrete::Text),
        any::<Vec<u8>>().prop_map(Concrete::Bytes),
        Just(Concrete::Unit),
    ];
    // Up to depth 4, ~24 total nodes, branching ≤4.
    leaf.prop_recursive(4, 24, 4, |inner| {
        prop_oneof![
            proptest::option::of(inner.clone()).prop_map(|o| Concrete::Opt(o.map(Box::new))),
            prop::collection::vec(inner.clone(), 0..4).prop_map(Concrete::List),
            prop::collection::vec((".*", inner.clone()), 0..4).prop_map(Concrete::Map),
            (inner.clone(), inner.clone())
                .prop_map(|(a, b)| Concrete::Pair(Box::new(a), Box::new(b))),
        ]
    })
}

proptest! {
    /// The core wire contract: `deserialize(serialize(x)) == x` over arbitrary
    /// nested concrete data.
    #[test]
    fn wire_bincode_roundtrip(c in arb_concrete()) {
        let bytes = wire::serialize(&c).expect("serialize");
        let back: Concrete = wire::deserialize(&bytes).expect("deserialize");
        prop_assert_eq!(c, back);
    }

    /// The neutral `Value` model as a lossless bridge for concrete types:
    /// `from_value(to_value(&x)) == x`. (This is how `Value` is actually used —
    /// to carry concrete payloads across the dynamic/component boundary.)
    #[test]
    fn value_bridge_roundtrip(c in arb_concrete()) {
        let v = to_value(&c).expect("to_value");
        let back: Concrete = from_value(v).expect("from_value");
        prop_assert_eq!(c, back);
    }
}

// Multi-arg tuple packing (FIDIUS-I-0033 Phase 4): the macro packs N method args
// as a tuple — `()` for 0, `(a,)` for 1, `(a, b, …)` for N — and the wire must
// round-trip that packing. Cover the 0/1 edge cases plus a few arities and types.
proptest! {
    #[test]
    fn pack_arity0(_u in Just(())) {
        let t: () = ();
        let back: () = wire::deserialize(&wire::serialize(&t).unwrap()).unwrap();
        prop_assert_eq!(t, back);
    }

    #[test]
    fn pack_arity1(a in any::<i64>()) {
        let t = (a,);
        let back: (i64,) = wire::deserialize(&wire::serialize(&t).unwrap()).unwrap();
        prop_assert_eq!(t, back);
    }

    #[test]
    fn pack_arity2(a in any::<u32>(), b in ".*") {
        let t = (a, b);
        let back: (u32, String) = wire::deserialize(&wire::serialize(&t).unwrap()).unwrap();
        prop_assert_eq!(t, back);
    }

    #[test]
    fn pack_arity3(a in any::<bool>(), b in any::<i16>(), c in any::<Vec<u8>>()) {
        let t = (a, b, c);
        let back: (bool, i16, Vec<u8>) =
            wire::deserialize(&wire::serialize(&t).unwrap()).unwrap();
        prop_assert_eq!(t, back);
    }

    #[test]
    fn pack_arity4(
        a in any::<u64>(),
        b in ".*",
        c in any::<Option<i32>>(),
        d in any::<f64>().prop_filter("no NaN", |f| !f.is_nan()),
    ) {
        let t = (a, b, c, d);
        let back: (u64, String, Option<i32>, f64) =
            wire::deserialize(&wire::serialize(&t).unwrap()).unwrap();
        prop_assert_eq!(t, back);
    }
}
