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

//! Smoke test: build a cdylib plugin and load it with libloading.
//!
//! This test builds the test-plugin-smoke crate as a cdylib, then
//! loads it via dlopen/dlsym and verifies the registry and vtable work.

use std::process::Command;

#[test]
fn load_cdylib_and_call_plugin() {
    // Build the test plugin cdylib in the same profile as the test binary
    let mut args = vec![
        "build",
        "--manifest-path",
        "../../tests/test-plugin-smoke/Cargo.toml",
    ];
    if !cfg!(debug_assertions) {
        args.push("--release");
    }
    let build = Command::new("cargo")
        .args(&args)
        .output()
        .expect("failed to run cargo build");

    assert!(
        build.status.success(),
        "failed to build test plugin: {}",
        String::from_utf8_lossy(&build.stderr)
    );

    // Load the dylib
    let dylib_name = if cfg!(target_os = "macos") {
        "libtest_plugin_smoke.dylib"
    } else if cfg!(target_os = "windows") {
        "test_plugin_smoke.dll"
    } else {
        "libtest_plugin_smoke.so"
    };
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let dylib_path = format!("../../tests/test-plugin-smoke/target/{profile}/{dylib_name}");
    let lib = unsafe { libloading::Library::new(dylib_path) }.expect("failed to load cdylib");

    // Get the registry via fidius_get_registry function
    let get_registry: libloading::Symbol<
        unsafe extern "C" fn() -> *const fidius_core::descriptor::PluginRegistry,
    > = unsafe { lib.get(b"fidius_get_registry") }
        .expect("failed to find fidius_get_registry symbol");

    let registry = unsafe { &*get_registry() };

    // Verify registry
    assert_eq!(&registry.magic, b"FIDIUS\0\0");
    assert_eq!(registry.registry_version, 1);
    // test-plugin-smoke has two plugins: BasicCalculator (PluginAllocated) and
    // ArenaEchoer (Arena). Find BasicCalculator for this smoke test.
    assert!(registry.plugin_count >= 1);

    let desc = (0..registry.plugin_count)
        .map(|i| unsafe { &**registry.descriptors.add(i as usize) })
        .find(|d| unsafe { d.plugin_name_str() } == "BasicCalculator")
        .expect("BasicCalculator descriptor not found");

    assert_eq!(desc.abi_version, 200);
    assert_eq!(desc.buffer_strategy, 1); // PluginAllocated
    assert!(desc.free_buffer.is_some());

    // Verify interface name
    let iface_name = unsafe { desc.interface_name_str() };
    assert_eq!(iface_name, "Calculator");

    // Verify plugin name
    let plugin_name = unsafe { desc.plugin_name_str() };
    assert_eq!(plugin_name, "BasicCalculator");

    // Call the `add` method through the vtable
    // The vtable has a known layout — first method is `add`
    // We need to serialize AddInput and deserialize AddOutput
    #[derive(serde::Serialize)]
    struct AddInput {
        a: i64,
        b: i64,
    }
    #[derive(serde::Deserialize, Debug)]
    struct AddOutput {
        result: i64,
    }

    let input = (AddInput { a: 3, b: 7 },);
    let input_bytes = fidius_core::wire::serialize(&input).unwrap();

    // The vtable's first function pointer is `add`
    // Read it as a raw fn pointer from the vtable
    let add_fn: unsafe extern "C" fn(*const u8, u32, *mut *mut u8, *mut u32) -> i32 = unsafe {
        *(desc.vtable as *const unsafe extern "C" fn(*const u8, u32, *mut *mut u8, *mut u32) -> i32)
    };

    let mut out_ptr: *mut u8 = std::ptr::null_mut();
    let mut out_len: u32 = 0;

    let status = unsafe {
        add_fn(
            input_bytes.as_ptr(),
            input_bytes.len() as u32,
            &mut out_ptr,
            &mut out_len,
        )
    };

    assert_eq!(status, 0, "FFI call returned error status {status}");

    let output_slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
    let output: AddOutput = fidius_core::wire::deserialize(output_slice).unwrap();
    assert_eq!(output.result, 10);

    // Free the output buffer
    if let Some(free) = desc.free_buffer {
        unsafe { free(out_ptr, out_len as usize) };
    }
}
