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

//! End-to-end validation tests: signing, negative cases.

use std::path::{Path, PathBuf};

use fidius_host::{LoadError, PluginHandle, PluginHost};
use fidius_test::{dylib_fixture, fixture_keypair_with_seed, sign_dylib};

fn plugin_source_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/test-plugin-smoke")
}

/// Cached plugin build directory — same fixture shared across all e2e tests.
fn plugin_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        dylib_fixture(plugin_source_dir())
            .build()
            .dir()
            .to_path_buf()
    })
}

fn dylib_path() -> PathBuf {
    let name = if cfg!(target_os = "macos") {
        "libtest_plugin_smoke.dylib"
    } else if cfg!(target_os = "windows") {
        "test_plugin_smoke.dll"
    } else {
        "libtest_plugin_smoke.so"
    };
    plugin_dir().join(name)
}

fn cleanup_sig() {
    let dylib = dylib_path();
    let ext = dylib.extension().unwrap().to_str().unwrap();
    let sig_path = dylib.with_extension(format!("{ext}.sig"));
    let _ = std::fs::remove_file(sig_path);
}

#[test]
#[serial_test::serial]
fn signed_plugin_loads_with_correct_key() {
    let (sk, pk) = fixture_keypair_with_seed(10);
    sign_dylib(&dylib_path(), &sk).expect("sign");

    let host = PluginHost::builder()
        .search_path(plugin_dir())
        .require_signature(true)
        .trusted_keys(&[pk])
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    assert_eq!(loaded.info.name, "BasicCalculator");

    cleanup_sig();
}

#[test]
#[serial_test::serial]
fn signed_plugin_fails_with_wrong_key() {
    let (sk, _) = fixture_keypair_with_seed(11);
    let (_, wrong_pk) = fixture_keypair_with_seed(12);
    sign_dylib(&dylib_path(), &sk).expect("sign");

    let host = PluginHost::builder()
        .search_path(plugin_dir())
        .require_signature(true)
        .trusted_keys(&[wrong_pk])
        .build()
        .unwrap();

    let result = host.load("BasicCalculator");
    assert!(
        matches!(result, Err(LoadError::SignatureInvalid { .. })),
        "expected SignatureInvalid, got {:?}",
        result
    );

    cleanup_sig();
}

#[test]
#[serial_test::serial]
fn unsigned_plugin_fails_when_signature_required() {
    cleanup_sig();

    let (_, pk) = fixture_keypair_with_seed(13);

    let host = PluginHost::builder()
        .search_path(plugin_dir())
        .require_signature(true)
        .trusted_keys(&[pk])
        .build()
        .unwrap();

    let result = host.load("BasicCalculator");
    assert!(
        matches!(result, Err(LoadError::SignatureRequired { .. })),
        "expected SignatureRequired, got {:?}",
        result
    );
}

#[test]
#[serial_test::serial]
fn unsigned_plugin_loads_without_signature_requirement() {
    cleanup_sig();

    let host = PluginHost::builder()
        .search_path(plugin_dir())
        .build()
        .unwrap();

    let loaded = host.load("BasicCalculator").unwrap();
    let handle = PluginHandle::from_loaded(loaded);

    #[derive(serde::Serialize)]
    struct AddInput {
        a: i64,
        b: i64,
    }
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct AddOutput {
        result: i64,
    }

    let output: AddOutput = handle
        .call_method(0, &(AddInput { a: 100, b: 200 },))
        .unwrap();
    assert_eq!(output, AddOutput { result: 300 });
}

#[test]
#[serial_test::serial]
fn lenient_policy_still_enforces_signatures() {
    // Lenient policy no longer bypasses signature enforcement.
    // require_signature(true) always enforces, regardless of policy.
    cleanup_sig();

    let (_, pk) = fixture_keypair_with_seed(14);

    let host = PluginHost::builder()
        .search_path(plugin_dir())
        .require_signature(true)
        .trusted_keys(&[pk])
        .load_policy(fidius_host::LoadPolicy::Lenient)
        .build()
        .unwrap();

    let result = host.load("BasicCalculator");
    assert!(
        matches!(result, Err(LoadError::SignatureRequired { .. })),
        "Lenient should still enforce signatures: got {:?}",
        result
    );
}

#[test]
#[serial_test::serial]
fn lenient_policy_still_rejects_wrong_key() {
    let (sk, _) = fixture_keypair_with_seed(15);
    let (_, wrong_pk) = fixture_keypair_with_seed(16);
    sign_dylib(&dylib_path(), &sk).expect("sign");

    let host = PluginHost::builder()
        .search_path(plugin_dir())
        .require_signature(true)
        .trusted_keys(&[wrong_pk])
        .load_policy(fidius_host::LoadPolicy::Lenient)
        .build()
        .unwrap();

    let result = host.load("BasicCalculator");
    assert!(
        matches!(result, Err(LoadError::SignatureInvalid { .. })),
        "Lenient should still reject wrong key: got {:?}",
        result
    );

    cleanup_sig();
}
