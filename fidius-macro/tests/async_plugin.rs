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

//! Test that async methods work with the fidius macros.
extern crate fidius;

use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait AsyncProcessor: Send + Sync {
    async fn process(&self, input: String) -> String;
}

pub struct MyProcessor;

#[plugin_impl(AsyncProcessor)]
impl AsyncProcessor for MyProcessor {
    async fn process(&self, input: String) -> String {
        // Simulate async work
        format!("processed: {}", input)
    }
}

fidius_core::fidius_plugin_registry!();

#[test]
fn can_call_async_method_via_vtable() {
    let reg = fidius_core::registry::get_registry();
    let desc = unsafe { &**reg.descriptors };
    let vtable =
        unsafe { &*(desc.vtable as *const __fidius_AsyncProcessor::AsyncProcessor_VTable) };

    let input = "hello".to_string();
    let input_bytes = fidius_core::wire::serialize(&input).unwrap();

    let mut out_ptr: *mut u8 = std::ptr::null_mut();
    let mut out_len: u32 = 0;

    let status = unsafe {
        (vtable.process)(
            input_bytes.as_ptr(),
            input_bytes.len() as u32,
            &mut out_ptr,
            &mut out_len,
        )
    };

    assert_eq!(status, 0); // STATUS_OK

    let output_slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
    let result: String = fidius_core::wire::deserialize(output_slice).unwrap();
    assert_eq!(result, "processed: hello");

    if let Some(free) = desc.free_buffer {
        unsafe { free(out_ptr, out_len as usize) };
    }
}
