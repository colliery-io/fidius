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

//! Test that `crate = "..."` attribute works on both plugin_interface and plugin_impl.
//!
//! Uses `fidius_core` directly (the real crate name) instead of the `fidius` alias
//! to verify custom crate path resolution.

use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_core")]
pub trait Calculator: Send + Sync {
    fn add(&self, input: String) -> String;
}

pub struct MyCalculator;

#[plugin_impl(Calculator, crate = "fidius_core")]
impl Calculator for MyCalculator {
    fn add(&self, input: String) -> String {
        format!("result: {}", input)
    }
}

fidius_core::fidius_plugin_registry!();

#[test]
fn custom_crate_path_compiles_and_works() {
    let reg = fidius_core::registry::get_registry();
    assert_eq!(&reg.magic, b"FIDIUS\0\0");
    assert_eq!(reg.plugin_count, 1);
}

#[test]
fn custom_crate_path_shim_callable() {
    let reg = fidius_core::registry::get_registry();
    let desc = unsafe { &**reg.descriptors };
    let vtable = unsafe { &*(desc.vtable as *const __fidius_Calculator::Calculator_VTable) };

    let input = ("42".to_string(),);
    let input_bytes = fidius_core::wire::serialize(&input).unwrap();

    let mut out_ptr: *mut u8 = std::ptr::null_mut();
    let mut out_len: u32 = 0;

    let status = unsafe {
        (vtable.add)(
            input_bytes.as_ptr(),
            input_bytes.len() as u32,
            &mut out_ptr,
            &mut out_len,
        )
    };

    assert_eq!(status, 0);

    let output_slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
    let result: String = fidius_core::wire::deserialize(output_slice).unwrap();
    assert_eq!(result, "result: 42");

    if let Some(free) = desc.free_buffer {
        unsafe { free(out_ptr, out_len as usize) };
    }
}
