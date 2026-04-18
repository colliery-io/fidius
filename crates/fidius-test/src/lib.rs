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

//! Testing helpers for Fidius plugin authors and hosts.
//!
//! This crate provides the infrastructure the Fidius codebase uses internally
//! for its own tests, now exposed so downstream users don't have to reinvent
//! the wheel. Add it under `[dev-dependencies]` and you get:
//!
//! - [`dylib_fixture`] — build a plugin crate's cdylib via `cargo build`,
//!   cached across tests in the same process. Optional signing via
//!   [`DylibFixtureBuilder::signed_with`].
//! - [`signing::fixture_keypair`] — deterministic Ed25519 keypair for tests.
//! - [`signing::sign_dylib`] — produce a `.sig` file next to a dylib.
//!
//! # Example
//!
//! ```ignore
//! use fidius_test::dylib_fixture;
//! use fidius_host::PluginHost;
//!
//! #[test]
//! fn loads_plugin() {
//!     let fixture = dylib_fixture("./path/to/my-plugin").build();
//!     let host = PluginHost::builder()
//!         .search_path(fixture.dir())
//!         .build()
//!         .unwrap();
//!     let plugins = host.discover().unwrap();
//!     assert!(!plugins.is_empty());
//! }
//! ```

pub mod dylib;
pub mod signing;

pub use dylib::{dylib_fixture, DylibFixture, DylibFixtureBuilder};
pub use signing::{fixture_keypair, fixture_keypair_with_seed, sign_dylib};
