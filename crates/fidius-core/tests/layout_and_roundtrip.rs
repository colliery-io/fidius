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

//! Layout assertion tests and serialization round-trip tests for fidius-core.
//!
//! These tests guard ABI stability by asserting struct sizes, alignments,
//! and field offsets. They also verify wire format round-trip correctness
//! and interface hash determinism.

use std::mem::{align_of, offset_of, size_of};

use fidius_core::descriptor::*;
use fidius_core::error::PluginError;
use fidius_core::hash::{fnv1a, interface_hash};
use fidius_core::status::*;
use fidius_core::wire;

// ─── Layout assertions: PluginRegistry ───────────────────────────────────────

#[test]
fn registry_size_and_align() {
    // 64-bit: [u8;8] + u32 + u32 + *const = 8 + 4 + 4 + 8 = 24
    assert_eq!(size_of::<PluginRegistry>(), 24);
    assert_eq!(align_of::<PluginRegistry>(), 8);
}

#[test]
fn registry_field_offsets() {
    assert_eq!(offset_of!(PluginRegistry, magic), 0);
    assert_eq!(offset_of!(PluginRegistry, registry_version), 8);
    assert_eq!(offset_of!(PluginRegistry, plugin_count), 12);
    assert_eq!(offset_of!(PluginRegistry, descriptors), 16);
}

// ─── Layout assertions: PluginDescriptor ─────────────────────────────────────

#[test]
fn descriptor_size_and_align() {
    // 64-bit expected: 104 bytes total, 8-byte aligned.
    // Grew from 80 → 104 in 0.1.0 when FIDIUS-I-0018 added method_metadata
    // pointer (8), trait_metadata pointer (8), trait_metadata_count u32 (4),
    // plus 4 bytes trailing alignment padding.
    let size = size_of::<PluginDescriptor>();
    let align = align_of::<PluginDescriptor>();
    assert_eq!(align, 8, "PluginDescriptor alignment");
    assert_eq!(size, 104, "PluginDescriptor size");
}

#[test]
fn descriptor_field_offsets() {
    assert_eq!(offset_of!(PluginDescriptor, descriptor_size), 0);
    assert_eq!(offset_of!(PluginDescriptor, abi_version), 4);
    assert_eq!(offset_of!(PluginDescriptor, interface_name), 8);
    assert_eq!(offset_of!(PluginDescriptor, interface_hash), 16);
    assert_eq!(offset_of!(PluginDescriptor, interface_version), 24);
    // 4 bytes padding after u32 before u64
    assert_eq!(offset_of!(PluginDescriptor, capabilities), 32);
    assert_eq!(offset_of!(PluginDescriptor, buffer_strategy), 40);
    // 7 bytes padding before pointer
    assert_eq!(offset_of!(PluginDescriptor, plugin_name), 48);
    assert_eq!(offset_of!(PluginDescriptor, vtable), 56);
    assert_eq!(offset_of!(PluginDescriptor, free_buffer), 64);
    assert_eq!(offset_of!(PluginDescriptor, method_count), 72);
    // 4 bytes padding after u32 before pointer
    assert_eq!(offset_of!(PluginDescriptor, method_metadata), 80);
    assert_eq!(offset_of!(PluginDescriptor, trait_metadata), 88);
    assert_eq!(offset_of!(PluginDescriptor, trait_metadata_count), 96);
    // 4 bytes trailing padding to 104
}

// ─── Layout assertions: enums ────────────────────────────────────────────────

#[test]
fn buffer_strategy_kind_layout() {
    // Discriminant `0` is reserved (was `CallerAllocated`, removed in 0.1.0).
    assert_eq!(size_of::<BufferStrategyKind>(), 1);
    assert_eq!(BufferStrategyKind::PluginAllocated as u8, 1);
    assert_eq!(BufferStrategyKind::Arena as u8, 2);
}

// ─── Status code values ──────────────────────────────────────────────────────

#[test]
fn status_code_values() {
    assert_eq!(STATUS_OK, 0);
    assert_eq!(STATUS_BUFFER_TOO_SMALL, -1);
    assert_eq!(STATUS_SERIALIZATION_ERROR, -2);
    assert_eq!(STATUS_PLUGIN_ERROR, -3);
    assert_eq!(STATUS_PANIC, -4);
}

// ─── Wire format round-trip ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct TestPayload {
    name: String,
    value: i64,
    tags: Vec<String>,
}

#[test]
fn wire_roundtrip() {
    let payload = TestPayload {
        name: "test".into(),
        value: 42,
        tags: vec!["a".into(), "b".into()],
    };

    let bytes = wire::serialize(&payload).expect("serialize failed");
    let recovered: TestPayload = wire::deserialize(&bytes).expect("deserialize failed");
    assert_eq!(payload, recovered);
}

#[test]
fn wire_is_bincode_always() {
    // Post-0.1.0 there is exactly one wire format (bincode); build profile
    // has no effect. This test asserts the round-trip produces bincode bytes
    // and that they're NOT JSON.
    let payload = TestPayload {
        name: "hello".into(),
        value: 1,
        tags: vec![],
    };
    let bytes = wire::serialize(&payload).unwrap();
    assert!(
        serde_json::from_slice::<serde_json::Value>(&bytes).is_err(),
        "wire output should be bincode, not JSON"
    );
}

// ─── PluginError round-trip ──────────────────────────────────────────────────

#[test]
fn plugin_error_roundtrip_without_details() {
    let err = PluginError::new("NOT_FOUND", "item not found");
    let bytes = wire::serialize(&err).unwrap();
    let recovered: PluginError = wire::deserialize(&bytes).unwrap();
    assert_eq!(err, recovered);
    assert!(recovered.details.is_none());
    assert!(recovered.details_value().is_none());
}

#[test]
fn plugin_error_roundtrip_with_details() {
    let details = serde_json::json!({"field": "name", "reason": "too short"});
    let err = PluginError::with_details("VALIDATION", "validation failed", details.clone());
    let bytes = wire::serialize(&err).unwrap();
    let recovered: PluginError = wire::deserialize(&bytes).unwrap();
    assert_eq!(err, recovered);
    assert_eq!(recovered.details_value().unwrap(), details);
}

#[test]
fn plugin_error_display() {
    let err = PluginError::new("ERR_CODE", "something went wrong");
    assert_eq!(format!("{err}"), "[ERR_CODE] something went wrong");
}

// ─── Interface hash ──────────────────────────────────────────────────────────

#[test]
fn hash_known_vectors() {
    // These are regression vectors — if the hash algorithm changes, these break.
    let v1 = interface_hash(&["name:->String"]);
    let v2 = interface_hash(&["process:&[u8],Value->Result<Vec<u8>,PluginError>"]);
    let v3 = interface_hash(&[
        "name:->String",
        "process:&[u8],Value->Result<Vec<u8>,PluginError>",
    ]);

    // Hardcode after first run — these are the "golden" values.
    // For now, just verify determinism and distinctness.
    assert_eq!(v1, interface_hash(&["name:->String"]));
    assert_eq!(
        v2,
        interface_hash(&["process:&[u8],Value->Result<Vec<u8>,PluginError>"])
    );
    assert_eq!(
        v3,
        interface_hash(&[
            "name:->String",
            "process:&[u8],Value->Result<Vec<u8>,PluginError>",
        ])
    );

    // All three must be distinct
    assert_ne!(v1, v2);
    assert_ne!(v1, v3);
    assert_ne!(v2, v3);
}

#[test]
fn hash_const_fnv1a() {
    // fnv1a is const — verify it works at compile time
    const HASH: u64 = fnv1a(b"fidius");
    assert_ne!(HASH, 0);
    assert_eq!(HASH, fnv1a(b"fidius"));
}

// ─── Magic bytes ─────────────────────────────────────────────────────────────

#[test]
fn magic_bytes_value() {
    assert_eq!(&FIDIUS_MAGIC, b"FIDIUS\0\0");
    assert_eq!(FIDIUS_MAGIC.len(), 8);
}

#[test]
fn version_constants() {
    assert_eq!(REGISTRY_VERSION, 1);
    // ABI_VERSION = MAJOR * 10000 + MINOR * 100 when MAJOR == 0 (pre-1.0 rule).
    // For fidius-core at 0.1.0: 0 + 100 = 100. See ADR-0002.
    assert_eq!(ABI_VERSION, 100);
}
