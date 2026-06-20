// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// WASM author fixture (FIDIUS-I-0023): exercises user types end to end —
// a record in a *submodule* (geom::Point), and an enum with a single-field case,
// a record case (Rect), a *struct variant* (Triangle, → synthetic record), and a
// unit case. build.rs (fidius_build::emit_wit) follows `mod geom;` and generates
// wit/ + conversions; #[plugin_impl] exports the component.

use fidius_macro::{plugin_impl, plugin_interface, WitType};

pub mod geom;
use geom::Point;

#[derive(WitType, Clone)]
pub enum Shape {
    Circle(u32),
    Rect(Point),
    Triangle { base: u32, height: u32 },
    Dot,
}

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Geo: Send + Sync {
    fn midpoint(&self, a: Point, b: Point) -> Point;
    fn describe(&self, s: Shape) -> String;
    // PC.1: a map arg + tuple arg + map return — maps cross as `list<tuple<k,v>>`.
    fn tally(
        &self,
        counts: std::collections::HashMap<String, u32>,
        bump: (i32, i32),
    ) -> std::collections::HashMap<String, u32>;
}

pub struct MyGeo;

#[plugin_impl(Geo, crate = "fidius_guest")]
impl Geo for MyGeo {
    fn midpoint(&self, a: Point, b: Point) -> Point {
        Point {
            x: (a.x + b.x) / 2,
            y: (a.y + b.y) / 2,
        }
    }

    fn describe(&self, s: Shape) -> String {
        match s {
            Shape::Circle(r) => format!("circle r={r}"),
            Shape::Rect(p) => format!("rect at {},{}", p.x, p.y),
            Shape::Triangle { base, height } => format!("triangle {base}x{height}"),
            Shape::Dot => "dot".to_string(),
        }
    }

    fn tally(
        &self,
        counts: std::collections::HashMap<String, u32>,
        bump: (i32, i32),
    ) -> std::collections::HashMap<String, u32> {
        let add = (bump.0 + bump.1) as u32;
        counts.into_iter().map(|(k, v)| (k, v + add)).collect()
    }
}
