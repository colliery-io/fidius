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

//! End-to-end for user-defined types in a WASM plugin (FIDIUS-I-0023): an
//! interface passing a `record` (Point) and a `variant` (Shape). The
//! `records-greeter` fixture is built (its build.rs generates wit/ + the
//! conversions); the host loads it via `load_wasm` against the macro-emitted
//! descriptor and round-trips records/variants through the `Value` boundary —
//! exercising the kebab↔snake/Pascal name normalization end to end.

#![cfg(feature = "wasm")]
#![allow(unexpected_cfgs)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use fidius_host::PluginHost;

// Host-side mirror of the fixture's interface. Same method signatures → same
// interface hash + export name as the guest. `Serialize`/`Deserialize` drive the
// `Value` marshalling; the names need no kebab annotations (the WASM executor
// normalizes record/variant names to/from WIT's kebab-case).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Shape {
    Circle(u32),
    Rect(Point),
    Triangle { base: u32, height: u32 },
    Dot,
}

#[fidius_macro::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Geo: Send + Sync {
    fn midpoint(&self, a: Point, b: Point) -> Point;
    fn describe(&self, s: Shape) -> String;
    fn tally(
        &self,
        counts: std::collections::HashMap<String, u32>,
        bump: (i32, i32),
    ) -> std::collections::HashMap<String, u32>;
}

fn records_greeter_component() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/wasm-fixtures/records-greeter");
        let status = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .current_dir(&fixture)
            .status()
            .expect("run `cargo build --target wasm32-wasip2` (see T-0094 for the toolchain)");
        assert!(status.success(), "records-greeter wasm build failed");
        let art = fixture.join("target/wasm32-wasip2/release/records_greeter.wasm");
        std::fs::read(&art).unwrap_or_else(|e| panic!("read {}: {e}", art.display()))
    })
}

fn stage_pkg(root: &std::path::Path) {
    let dir = root.join("records-greeter-pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("package.toml"),
        r#"
[package]
name = "records-greeter-pkg"
version = "0.1.0"
interface = "geo"
interface_version = 1
runtime = "wasm"

[metadata]
category = "test"

[wasm]
component = "records_greeter.wasm"
"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("records_greeter.wasm"),
        records_greeter_component(),
    )
    .unwrap();
}

#[test]
fn record_in_record_out_round_trips() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm("records-greeter-pkg", &__fidius_Geo::Geo_WASM_DESCRIPTOR)
        .expect("load records-greeter against the macro descriptor");

    // midpoint: record args in, record out.
    let mid: Point = handle
        .call_method(0, &(Point { x: 0, y: 0 }, Point { x: 4, y: 8 }))
        .unwrap();
    assert_eq!(mid, Point { x: 2, y: 4 });
}

#[test]
fn variant_in_round_trips_all_cases() {
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm("records-greeter-pkg", &__fidius_Geo::Geo_WASM_DESCRIPTOR)
        .unwrap();

    // describe: variant in (each case shape), string out.
    let circle: String = handle.call_method(1, &(Shape::Circle(7),)).unwrap();
    assert_eq!(circle, "circle r=7");

    let rect: String = handle
        .call_method(1, &(Shape::Rect(Point { x: 1, y: 2 }),))
        .unwrap();
    assert_eq!(rect, "rect at 1,2");

    // Struct variant → synthetic WIT record payload.
    let tri: String = handle
        .call_method(1, &(Shape::Triangle { base: 3, height: 4 },))
        .unwrap();
    assert_eq!(tri, "triangle 3x4");

    let dot: String = handle.call_method(1, &(Shape::Dot,)).unwrap();
    assert_eq!(dot, "dot");
}

#[test]
fn maps_and_tuples_round_trip() {
    use std::collections::HashMap;
    let tmp = tempfile::TempDir::new().unwrap();
    stage_pkg(tmp.path());
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let handle = host
        .load_wasm("records-greeter-pkg", &__fidius_Geo::Geo_WASM_DESCRIPTOR)
        .unwrap();

    // tally (method 2): HashMap arg + (i32,i32) tuple arg → HashMap return.
    // Maps cross as `list<tuple<string, u32>>`; the tuple bump (2+3) is added to
    // each value. Exercises type-directed tuple lowering + map round-trip.
    let mut counts = HashMap::new();
    counts.insert("a".to_string(), 1u32);
    counts.insert("b".to_string(), 10u32);
    let out: HashMap<String, u32> = handle.call_method(2, &(counts, (2i32, 3i32))).unwrap();
    assert_eq!(out.get("a"), Some(&6));
    assert_eq!(out.get("b"), Some(&15));
    assert_eq!(out.len(), 2);
}
