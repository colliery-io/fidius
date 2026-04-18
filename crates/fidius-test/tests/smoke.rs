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

//! Self-tests exercising the public API against the real test-plugin-smoke
//! fixture.

use std::path::PathBuf;

use ed25519_dalek::Verifier;
use fidius_host::PluginHost;
use fidius_test::{dylib_fixture, fixture_keypair, fixture_keypair_with_seed, sign_dylib};
use test_plugin_smoke::{AddInput, CalculatorClient};

fn plugin_source_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/test-plugin-smoke")
}

#[test]
fn fixture_keypair_is_deterministic() {
    let (sk_a, pk_a) = fixture_keypair();
    let (sk_b, pk_b) = fixture_keypair();
    assert_eq!(sk_a.to_bytes(), sk_b.to_bytes());
    assert_eq!(pk_a.to_bytes(), pk_b.to_bytes());
}

#[test]
fn fixture_keypair_with_seed_differs_by_seed() {
    let (_, pk1) = fixture_keypair_with_seed(1);
    let (_, pk2) = fixture_keypair_with_seed(2);
    assert_ne!(pk1.to_bytes(), pk2.to_bytes());
}

#[test]
fn sign_dylib_produces_verifiable_signature() {
    let tmp = tempfile::NamedTempFile::new().expect("tempfile");
    std::fs::write(tmp.path(), b"plugin bytes").expect("write temp");

    let (sk, pk) = fixture_keypair();
    sign_dylib(tmp.path(), &sk).expect("sign_dylib");

    let existing_ext = tmp
        .path()
        .extension()
        .and_then(|e: &std::ffi::OsStr| e.to_str())
        .unwrap_or("");
    let sig_path = tmp.path().with_extension(format!("{existing_ext}.sig"));
    let raw = std::fs::read(&sig_path).expect("read sig");
    let sig_bytes: [u8; 64] = raw
        .as_slice()
        .try_into()
        .expect("signature file should be 64 bytes");
    let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);

    let body = std::fs::read(tmp.path()).expect("read temp");
    pk.verify(&body, &signature).expect("signature verifies");
}

#[test]
fn dylib_fixture_builds_plugin_and_host_can_discover() {
    let fixture = dylib_fixture(plugin_source_dir()).build();

    // Fixture points at a directory containing the dylib
    assert!(fixture.dir().is_dir());
    assert!(fixture.dylib_path().is_file());

    // Host can find the plugin at that search path
    let host = PluginHost::builder()
        .search_path(fixture.dir())
        .build()
        .unwrap();

    let names: Vec<String> = host
        .discover()
        .unwrap()
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert!(
        names.contains(&"BasicCalculator".to_string()),
        "expected BasicCalculator in {:?}",
        names
    );
}

#[test]
fn dylib_fixture_is_cached_across_builds() {
    // Two calls return fixtures with identical paths — the second is served
    // from the process-wide cache without re-running cargo build. We can't
    // assert on timing reliably, so we assert on identity of the returned
    // path fields.
    let first = dylib_fixture(plugin_source_dir()).build();
    let second = dylib_fixture(plugin_source_dir()).build();
    assert_eq!(first.dir(), second.dir());
    assert_eq!(first.dylib_path(), second.dylib_path());
}

#[test]
fn client_in_process_calls_plugin_without_dylib_load() {
    // test-plugin-smoke is linked as an rlib dev-dep into this test binary,
    // so its BasicCalculator #[plugin_impl] has registered a descriptor in
    // the in-process inventory. Client::in_process looks it up and returns
    // a typed client that invokes the shim directly (no libloading, no
    // cargo build, no subprocess).
    let client = CalculatorClient::in_process("BasicCalculator").expect("in_process lookup");
    let out = client
        .add(&AddInput { a: 3, b: 7 })
        .expect("add() call via in-process client");
    assert_eq!(out.result, 10);
}

#[test]
fn client_in_process_returns_not_found_for_missing_plugin() {
    let result = CalculatorClient::in_process("DoesNotExist");
    match result {
        Err(fidius_host::LoadError::PluginNotFound { .. }) => {}
        Err(other) => panic!("expected PluginNotFound, got {other:?}"),
        Ok(_) => panic!("expected error for missing plugin, got Ok"),
    }
}
