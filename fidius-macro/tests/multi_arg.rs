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

//! Test that multi-arg, single-arg, and zero-arg methods all work
//! with uniform tuple encoding.

extern crate fidius_core as fidius;

use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait MultiArg: Send + Sync {
    /// Zero args — returns a constant.
    fn status(&self) -> String;

    /// One arg — wraps in a message.
    fn echo(&self, msg: String) -> String;

    /// Two args — concatenates.
    fn concat(&self, a: String, b: String) -> String;

    /// Three args — arithmetic.
    fn add_three(&self, x: i64, y: i64, z: i64) -> i64;
}

pub struct MyMultiArg;

#[plugin_impl(MultiArg)]
impl MultiArg for MyMultiArg {
    fn status(&self) -> String {
        "ok".to_string()
    }

    fn echo(&self, msg: String) -> String {
        format!("echo: {msg}")
    }

    fn concat(&self, a: String, b: String) -> String {
        format!("{a}{b}")
    }

    fn add_three(&self, x: i64, y: i64, z: i64) -> i64 {
        x + y + z
    }
}

fidius_core::fidius_plugin_registry!();

fn get_registry() -> &'static fidius_core::descriptor::PluginRegistry {
    fidius_core::registry::get_registry()
}

/// Helper: call a vtable method by index with given input bytes.
unsafe fn call_vtable(
    vtable: &__fidius_MultiArg::MultiArg_VTable,
    index: usize,
    input_bytes: &[u8],
    free_buffer: Option<unsafe extern "C" fn(*mut u8, usize)>,
) -> (i32, Vec<u8>) {
    let fns: [unsafe extern "C" fn(*const u8, u32, *mut *mut u8, *mut u32) -> i32; 4] =
        [vtable.status, vtable.echo, vtable.concat, vtable.add_three];

    let mut out_ptr: *mut u8 = std::ptr::null_mut();
    let mut out_len: u32 = 0;

    let status = unsafe {
        (fns[index])(
            input_bytes.as_ptr(),
            input_bytes.len() as u32,
            &mut out_ptr,
            &mut out_len,
        )
    };

    let output = if !out_ptr.is_null() && out_len > 0 {
        let slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
        let v = slice.to_vec();
        if let Some(free) = free_buffer {
            unsafe { free(out_ptr, out_len as usize) };
        }
        v
    } else {
        vec![]
    };

    (status, output)
}

#[test]
fn zero_args_status() {
    let reg = get_registry();
    let desc = unsafe { &**reg.descriptors };
    let vtable = unsafe { &*(desc.vtable as *const __fidius_MultiArg::MultiArg_VTable) };

    // Zero args: serialize ()
    let input_bytes = fidius_core::wire::serialize(&()).unwrap();
    let (status, output) = unsafe { call_vtable(vtable, 0, &input_bytes, desc.free_buffer) };

    assert_eq!(status, 0, "status() should return STATUS_OK");
    let result: String = fidius_core::wire::deserialize(&output).unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn one_arg_echo() {
    let reg = get_registry();
    let desc = unsafe { &**reg.descriptors };
    let vtable = unsafe { &*(desc.vtable as *const __fidius_MultiArg::MultiArg_VTable) };

    // One arg: serialize (String,)
    let input_bytes = fidius_core::wire::serialize(&("hello".to_string(),)).unwrap();
    let (status, output) = unsafe { call_vtable(vtable, 1, &input_bytes, desc.free_buffer) };

    assert_eq!(status, 0, "echo() should return STATUS_OK");
    let result: String = fidius_core::wire::deserialize(&output).unwrap();
    assert_eq!(result, "echo: hello");
}

#[test]
fn two_args_concat() {
    let reg = get_registry();
    let desc = unsafe { &**reg.descriptors };
    let vtable = unsafe { &*(desc.vtable as *const __fidius_MultiArg::MultiArg_VTable) };

    // Two args: serialize (String, String)
    let input_bytes =
        fidius_core::wire::serialize(&("foo".to_string(), "bar".to_string())).unwrap();
    let (status, output) = unsafe { call_vtable(vtable, 2, &input_bytes, desc.free_buffer) };

    assert_eq!(status, 0, "concat() should return STATUS_OK");
    let result: String = fidius_core::wire::deserialize(&output).unwrap();
    assert_eq!(result, "foobar");
}

#[test]
fn three_args_add() {
    let reg = get_registry();
    let desc = unsafe { &**reg.descriptors };
    let vtable = unsafe { &*(desc.vtable as *const __fidius_MultiArg::MultiArg_VTable) };

    // Three args: serialize (i64, i64, i64)
    let input_bytes = fidius_core::wire::serialize(&(10i64, 20i64, 30i64)).unwrap();
    let (status, output) = unsafe { call_vtable(vtable, 3, &input_bytes, desc.free_buffer) };

    assert_eq!(status, 0, "add_three() should return STATUS_OK");
    let result: i64 = fidius_core::wire::deserialize(&output).unwrap();
    assert_eq!(result, 60);
}

#[test]
fn method_indices_correct() {
    assert_eq!(__fidius_MultiArg::METHOD_STATUS, 0);
    assert_eq!(__fidius_MultiArg::METHOD_ECHO, 1);
    assert_eq!(__fidius_MultiArg::METHOD_CONCAT, 2);
    assert_eq!(__fidius_MultiArg::METHOD_ADD_THREE, 3);
}
