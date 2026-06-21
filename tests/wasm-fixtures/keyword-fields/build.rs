// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// Regenerate wit/ + the author<->component conversions from src/lib.rs on every
// build (FIDIUS-I-0023). A proc-macro can't see the #[derive(WitType)] type
// definitions, so this build step produces the WIT the #[plugin_impl] adapter's
// `wit_bindgen::generate!{ path: "wit" }` consumes.
fn main() {
    fidius_build::emit_wit();
}
