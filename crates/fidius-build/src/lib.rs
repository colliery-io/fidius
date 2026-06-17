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

//! `build.rs` helper for Fidius WASM plugins (FIDIUS-I-0023).
//!
//! A proc-macro can't see a crate's `struct`/`enum` definitions, so the WIT for
//! `#[derive(WitType)]` user types must be generated from the source by a build
//! step. Call this from a one-line `build.rs`:
//!
//! ```ignore
//! // build.rs
//! fn main() {
//!     fidius_build::emit_wit();
//! }
//! ```
//!
//! It parses the crate's `src/lib.rs`, and writes:
//! - `wit/<interface>.wit` (consumed by the `#[plugin_impl]` adapter's
//!   `wit_bindgen::generate!{ path: "wit" }`), and
//! - `$OUT_DIR/fidius_wit_conversions.rs` (the generated↔author `From` impls the
//!   adapter `include!`s).
//!
//! Re-runs whenever `src/lib.rs` changes. v1 expects the `#[plugin_interface]`
//! trait and the `#[derive(WitType)]` types to live in `src/lib.rs`.

use std::path::Path;

/// Regenerate `wit/` and the conversions from `src/lib.rs`. Call from `build.rs`.
/// Panics with a clear message on any error (the build-script convention).
pub fn emit_wit() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is set by cargo for build scripts");
    let out = std::env::var("OUT_DIR").expect("OUT_DIR is set by cargo for build scripts");
    if let Err(e) = run(Path::new(&manifest), Path::new(&out)) {
        panic!("fidius_build::emit_wit failed: {e}");
    }
}

/// Core of [`emit_wit`], parameterized on the crate dir + output dir so it is
/// testable without a cargo build-script environment.
pub fn run(manifest_dir: &Path, out_dir: &Path) -> Result<(), String> {
    let lib = manifest_dir.join("src").join("lib.rs");
    println!("cargo:rerun-if-changed={}", lib.display());

    let src =
        std::fs::read_to_string(&lib).map_err(|e| format!("reading {}: {e}", lib.display()))?;
    let generated = fidius_wit::generate(&src)?;

    let wit_dir = manifest_dir.join("wit");
    std::fs::create_dir_all(&wit_dir).map_err(|e| format!("creating wit/: {e}"))?;
    let wit_file = wit_dir.join(format!("{}.wit", generated.iface_kebab));
    std::fs::write(&wit_file, &generated.wit)
        .map_err(|e| format!("writing {}: {e}", wit_file.display()))?;

    // The adapter `include!`s this unconditionally on the user-type path; write
    // it (possibly empty for a primitives-only interface) so the include never
    // dangles.
    let conv = out_dir.join("fidius_wit_conversions.rs");
    std::fs::write(&conv, &generated.conversions)
        .map_err(|e| format!("writing {}: {e}", conv.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_wit_and_conversions_for_a_user_typed_interface() {
        let tmp = tempfile::TempDir::new().unwrap();
        let manifest = tmp.path();
        std::fs::create_dir_all(manifest.join("src")).unwrap();
        std::fs::write(
            manifest.join("src/lib.rs"),
            r#"
                #[derive(WitType)]
                pub struct Point { pub x: i32, pub y: i32 }
                #[plugin_interface(version = 1, crate = "fidius_guest")]
                pub trait Geo { fn midpoint(&self, a: Point, b: Point) -> Point; }
            "#,
        )
        .unwrap();
        let out = tmp.path().join("out");
        std::fs::create_dir_all(&out).unwrap();

        run(manifest, &out).unwrap();

        let wit = std::fs::read_to_string(manifest.join("wit/geo.wit")).unwrap();
        assert!(wit.contains("record point {"));
        assert!(wit.contains("midpoint: func(a: point, b: point) -> point;"));

        let conv = std::fs::read_to_string(out.join("fidius_wit_conversions.rs")).unwrap();
        assert!(conv.contains("From<exports::fidius::geo::geo::Point> for crate::Point"));
    }

    #[test]
    fn primitives_only_writes_empty_conversions() {
        let tmp = tempfile::TempDir::new().unwrap();
        let manifest = tmp.path();
        std::fs::create_dir_all(manifest.join("src")).unwrap();
        std::fs::write(
            manifest.join("src/lib.rs"),
            r#"
                #[plugin_interface(version = 1)]
                pub trait Greeter { fn greet(&self, name: String) -> String; }
            "#,
        )
        .unwrap();
        let out = tmp.path().join("out");
        std::fs::create_dir_all(&out).unwrap();

        run(manifest, &out).unwrap();
        assert!(manifest.join("wit/greeter.wit").exists());
        let conv = std::fs::read_to_string(out.join("fidius_wit_conversions.rs")).unwrap();
        assert!(conv.is_empty());
    }
}
