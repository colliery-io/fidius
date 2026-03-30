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

//! Test that #[plugin_impl] compiles and generates expected items.
extern crate fidius_core as fidius;

use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;
}

pub struct HelloGreeter;

#[plugin_impl(Greeter)]
impl Greeter for HelloGreeter {
    fn greet(&self, name: String) -> String {
        format!("Hello, {}!", name)
    }
}

// Emit the registry export
fidius_core::fidius_plugin_registry!();

fn get_registry() -> &'static fidius_core::descriptor::PluginRegistry {
    fidius_core::registry::get_registry()
}

#[test]
fn registry_exists_and_is_valid() {
    let reg = get_registry();
    assert_eq!(&reg.magic, b"FIDIUS\0\0");
    assert_eq!(reg.registry_version, 1);
    assert_eq!(reg.plugin_count, 1);
}

#[test]
fn descriptor_fields_are_correct() {
    let reg = get_registry();
    let desc = unsafe { &**reg.descriptors };
    assert_eq!(desc.abi_version, 2);
    assert_eq!(
        desc.interface_hash,
        __fidius_Greeter::Greeter_INTERFACE_HASH
    );
    assert_eq!(desc.interface_version, 1);
    assert_eq!(desc.buffer_strategy, 1); // PluginAllocated
    assert!(desc.free_buffer.is_some());
}

#[test]
fn can_call_shim_via_vtable() {
    let reg = get_registry();
    let desc = unsafe { &**reg.descriptors };
    let vtable = unsafe { &*(desc.vtable as *const __fidius_Greeter::Greeter_VTable) };

    // Serialize the input argument
    let input = "World".to_string();
    let input_bytes = fidius_core::wire::serialize(&input).unwrap();

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

    assert_eq!(status, 0); // STATUS_OK

    // Deserialize the output
    let output_slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
    let result: String = fidius_core::wire::deserialize(output_slice).unwrap();
    assert_eq!(result, "Hello, World!");

    // Free the buffer
    if let Some(free) = desc.free_buffer {
        unsafe { free(out_ptr, out_len as usize) };
    }
}
