// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// WASM author fixture (FIDIUS-I-0023): an interface that passes a user `record`
// (Point) and a `variant` (Shape). Built to wasm32-wasip2; loaded by the host
// E2E. The build.rs (fidius_build::emit_wit) generates wit/ + the conversions;
// #[plugin_impl] consumes them and exports the component.

use fidius_macro::{plugin_impl, plugin_interface, WitType};

#[derive(WitType, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(WitType, Clone)]
pub enum Shape {
    Circle(u32),
    Rect(Point),
    Dot,
}

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Geo: Send + Sync {
    fn midpoint(&self, a: Point, b: Point) -> Point;
    fn describe(&self, s: Shape) -> String;
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
            Shape::Dot => "dot".to_string(),
        }
    }
}
