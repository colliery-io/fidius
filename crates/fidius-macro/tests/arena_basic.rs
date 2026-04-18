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

//! Arena buffer strategy: compile test + raw vtable invocation.
//!
//! Exercises the Arena codegen path without going through the host — we
//! call the vtable fn pointer directly with a heap-allocated scratch
//! buffer as the arena.

extern crate fidius_core as fidius;

use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = Arena)]
pub trait EchoArena: Send + Sync {
    fn echo(&self, input: String) -> String;
}

pub struct MyEcho;

#[plugin_impl(EchoArena, buffer = Arena)]
impl EchoArena for MyEcho {
    fn echo(&self, input: String) -> String {
        format!("echo: {input}")
    }
}

fidius_core::fidius_plugin_registry!();

#[test]
fn arena_shim_round_trip_with_sufficient_buffer() {
    let reg = fidius_core::registry::get_registry();
    let desc = unsafe { &**reg.descriptors };

    // Buffer strategy is Arena (discriminant 2).
    assert_eq!(desc.buffer_strategy, 2);
    // Free buffer is None for Arena.
    assert!(desc.free_buffer.is_none());

    // Grab the echo fn pointer from the vtable.
    let vtable = unsafe { &*(desc.vtable as *const __fidius_EchoArena::EchoArena_VTable) };
    let echo_fn = vtable.echo;

    // Serialize the input as a 1-tuple.
    let input = ("hello".to_string(),);
    let input_bytes = fidius_core::wire::serialize(&input).unwrap();

    // Provide a 1 KB arena — more than enough for "echo: hello".
    let mut arena = vec![0u8; 1024];
    let mut out_offset: u32 = 0;
    let mut out_len: u32 = 0;

    let status = unsafe {
        echo_fn(
            input_bytes.as_ptr(),
            input_bytes.len() as u32,
            arena.as_mut_ptr(),
            arena.len() as u32,
            &mut out_offset,
            &mut out_len,
        )
    };

    assert_eq!(status, 0); // STATUS_OK
    assert_eq!(out_offset, 0);
    assert!(out_len > 0);

    // Read the serialized output back from the arena.
    let out_slice = &arena[out_offset as usize..(out_offset + out_len) as usize];
    let result: String = fidius_core::wire::deserialize(out_slice).unwrap();
    assert_eq!(result, "echo: hello");
}

#[test]
fn arena_shim_returns_buffer_too_small() {
    let reg = fidius_core::registry::get_registry();
    let desc = unsafe { &**reg.descriptors };
    let vtable = unsafe { &*(desc.vtable as *const __fidius_EchoArena::EchoArena_VTable) };
    let echo_fn = vtable.echo;

    // Serialize a long input so the output exceeds a tiny arena.
    let input = ("this is a reasonably long payload".to_string(),);
    let input_bytes = fidius_core::wire::serialize(&input).unwrap();

    // 4-byte arena: too small for the serialized output.
    let mut arena = [0u8; 4];
    let mut out_offset: u32 = 0;
    let mut out_len: u32 = 0;

    let status = unsafe {
        echo_fn(
            input_bytes.as_ptr(),
            input_bytes.len() as u32,
            arena.as_mut_ptr(),
            arena.len() as u32,
            &mut out_offset,
            &mut out_len,
        )
    };

    // Plugin returned STATUS_BUFFER_TOO_SMALL with the needed size in out_len.
    assert_eq!(status, -1);
    assert!(out_len as usize > arena.len());
}
