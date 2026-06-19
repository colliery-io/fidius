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

//! WIT mapping/rendering for the macro's WASM codegen.
//!
//! The implementation lives in the shared `fidius-wit` crate so the same logic
//! backs the macro, the `build.rs` helper, and the `fidius wit` CLI
//! (FIDIUS-I-0023). This module re-exports the pieces the macro uses.

pub(crate) use fidius_wit::{
    contains_user_type, conv_expr, render_wit, result_ok_type, return_to_wit, return_to_wit_with,
    rust_type_to_wit, stream_item_type, to_kebab_case, wit_type_with, WitMethod,
};
