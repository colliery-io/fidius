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

//! Metadata declaration and descriptor round-trip test.
//!
//! Exercises `#[method_meta]` + `#[trait_meta]` attribute parsing, emission
//! of static MetaKv arrays and MethodMetaEntry table, and correct wiring
//! into the PluginDescriptor at the plugin-link level (not dylib).
extern crate fidius_core as fidius;

use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = PluginAllocated)]
#[trait_meta("kind", "integration")]
#[trait_meta("stability", "stable")]
pub trait Tagged: Send + Sync {
    #[method_meta("effect", "write")]
    #[method_meta("idempotent", "false")]
    fn create(&self, name: String) -> String;

    #[method_meta("effect", "read")]
    fn list(&self) -> String;

    // No metadata on this method — entry in the table should be empty.
    fn version(&self) -> String;
}

pub struct MyTagged;

#[plugin_impl(Tagged)]
impl Tagged for MyTagged {
    fn create(&self, name: String) -> String {
        format!("created:{name}")
    }
    fn list(&self) -> String {
        "listed".into()
    }
    fn version(&self) -> String {
        "v1".into()
    }
}

fidius_core::fidius_plugin_registry!();

fn read_cstr(ptr: *const std::ffi::c_char) -> &'static str {
    unsafe { std::ffi::CStr::from_ptr(ptr) }
        .to_str()
        .expect("valid utf-8")
}

#[test]
fn trait_metadata_is_populated() {
    let reg = fidius_core::registry::get_registry();
    assert_eq!(reg.plugin_count, 1);
    let desc = unsafe { &**reg.descriptors };

    assert!(!desc.trait_metadata.is_null());
    assert_eq!(desc.trait_metadata_count, 2);

    let slice = unsafe { std::slice::from_raw_parts(desc.trait_metadata, 2) };
    assert_eq!(read_cstr(slice[0].key), "kind");
    assert_eq!(read_cstr(slice[0].value), "integration");
    assert_eq!(read_cstr(slice[1].key), "stability");
    assert_eq!(read_cstr(slice[1].value), "stable");
}

#[test]
fn method_metadata_is_populated_per_method() {
    let reg = fidius_core::registry::get_registry();
    let desc = unsafe { &**reg.descriptors };

    assert!(!desc.method_metadata.is_null());
    let table =
        unsafe { std::slice::from_raw_parts(desc.method_metadata, desc.method_count as usize) };

    // create (index 0): 2 kvs
    assert!(!table[0].kvs.is_null());
    assert_eq!(table[0].kv_count, 2);
    let kvs_create = unsafe { std::slice::from_raw_parts(table[0].kvs, 2) };
    assert_eq!(read_cstr(kvs_create[0].key), "effect");
    assert_eq!(read_cstr(kvs_create[0].value), "write");
    assert_eq!(read_cstr(kvs_create[1].key), "idempotent");
    assert_eq!(read_cstr(kvs_create[1].value), "false");

    // list (index 1): 1 kv
    assert!(!table[1].kvs.is_null());
    assert_eq!(table[1].kv_count, 1);
    let kvs_list = unsafe { std::slice::from_raw_parts(table[1].kvs, 1) };
    assert_eq!(read_cstr(kvs_list[0].key), "effect");
    assert_eq!(read_cstr(kvs_list[0].value), "read");

    // version (index 2): no metadata — empty entry
    assert!(table[2].kvs.is_null());
    assert_eq!(table[2].kv_count, 0);
}

#[test]
fn interface_hash_unaffected_by_metadata() {
    // Adding/removing metadata annotations must not change the interface hash.
    // We assert this by recomputing the hash from the method signatures
    // directly and comparing to the trait's INTERFACE_HASH constant.
    use fidius_core::hash::interface_hash;
    let expected = interface_hash(&["create:String->String", "list:->String", "version:->String"]);
    assert_eq!(__fidius_Tagged::Tagged_INTERFACE_HASH, expected);
}
