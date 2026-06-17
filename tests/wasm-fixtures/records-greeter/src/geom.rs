// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// A WitType record in a *submodule* (FIDIUS-T-0118) — the generator follows
// `mod geom;` from lib.rs and emits conversions against `crate::geom::Point`.

#[derive(fidius_macro::WitType, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
