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

//! Test that multiple #[plugin_impl] in one binary produces a registry with multiple plugins.
extern crate fidius_core as fidius;

use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;
}

// --- Plugin 1 ---
pub struct HelloGreeter;

#[plugin_impl(Greeter)]
impl Greeter for HelloGreeter {
    fn greet(&self, name: String) -> String {
        format!("Hello, {}!", name)
    }
}

// --- Plugin 2 ---
pub struct GoodbyeGreeter;

#[plugin_impl(Greeter)]
impl Greeter for GoodbyeGreeter {
    fn greet(&self, name: String) -> String {
        format!("Goodbye, {}!", name)
    }
}

// Emit the combined registry
fidius_core::fidius_plugin_registry!();

#[test]
fn registry_has_two_plugins() {
    let reg = fidius_core::registry::get_registry();
    assert_eq!(&reg.magic, b"FIDIUS\0\0");
    assert_eq!(reg.registry_version, 1);
    assert_eq!(reg.plugin_count, 2);
}

#[test]
fn both_descriptors_are_valid() {
    let reg = fidius_core::registry::get_registry();
    let descs: Vec<&fidius_core::descriptor::PluginDescriptor> = (0..reg.plugin_count)
        .map(|i| unsafe { &**reg.descriptors.add(i as usize) })
        .collect();

    for desc in &descs {
        assert_eq!(desc.abi_version, 100);
        assert_eq!(
            desc.interface_hash,
            __fidius_Greeter::Greeter_INTERFACE_HASH
        );
        assert_eq!(desc.buffer_strategy, 1);
        assert!(desc.free_buffer.is_some());
    }

    // Verify both plugin names are present
    let names: Vec<&str> = descs
        .iter()
        .map(|d| unsafe { d.plugin_name_str() })
        .collect();
    assert!(names.contains(&"HelloGreeter"));
    assert!(names.contains(&"GoodbyeGreeter"));
}

#[test]
fn can_call_both_plugins() {
    let reg = fidius_core::registry::get_registry();
    let descs: Vec<&fidius_core::descriptor::PluginDescriptor> = (0..reg.plugin_count)
        .map(|i| unsafe { &**reg.descriptors.add(i as usize) })
        .collect();

    let input = ("World".to_string(),);
    let input_bytes = fidius_core::wire::serialize(&input).unwrap();

    let mut results: Vec<String> = Vec::new();

    for desc in &descs {
        let vtable = unsafe { &*(desc.vtable as *const __fidius_Greeter::Greeter_VTable) };
        let mut out_ptr: *mut u8 = std::ptr::null_mut();
        let mut out_len: u32 = 0;

        let status = unsafe {
            (vtable.greet)(
                input_bytes.as_ptr(),
                input_bytes.len() as u32,
                &mut out_ptr,
                &mut out_len,
            )
        };
        assert_eq!(status, 0);

        let output_slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
        let result: String = fidius_core::wire::deserialize(output_slice).unwrap();
        results.push(result);

        if let Some(free) = desc.free_buffer {
            unsafe { free(out_ptr, out_len as usize) };
        }
    }

    assert!(results.contains(&"Hello, World!".to_string()));
    assert!(results.contains(&"Goodbye, World!".to_string()));
}
