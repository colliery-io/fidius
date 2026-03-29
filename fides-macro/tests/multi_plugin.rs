//! Test that multiple #[plugin_impl] in one binary produces a registry with multiple plugins.

use fides_macro::{plugin_impl, plugin_interface};

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
fides_core::fides_plugin_registry!();

#[test]
fn registry_has_two_plugins() {
    let reg = fides_core::registry::get_registry();
    assert_eq!(&reg.magic, b"FIDES\0\0\0");
    assert_eq!(reg.registry_version, 1);
    assert_eq!(reg.plugin_count, 2);
}

#[test]
fn both_descriptors_are_valid() {
    let reg = fides_core::registry::get_registry();
    let descs: Vec<&fides_core::descriptor::PluginDescriptor> = (0..reg.plugin_count)
        .map(|i| unsafe { &**reg.descriptors.add(i as usize) })
        .collect();

    for desc in &descs {
        assert_eq!(desc.abi_version, 1);
        assert_eq!(desc.interface_hash, Greeter_INTERFACE_HASH);
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
    let reg = fides_core::registry::get_registry();
    let descs: Vec<&fides_core::descriptor::PluginDescriptor> = (0..reg.plugin_count)
        .map(|i| unsafe { &**reg.descriptors.add(i as usize) })
        .collect();

    let input = "World".to_string();
    let input_bytes = fides_core::wire::serialize(&input).unwrap();

    let mut results: Vec<String> = Vec::new();

    for desc in &descs {
        let vtable = unsafe { &*(desc.vtable as *const Greeter_VTable) };
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
        let result: String = fides_core::wire::deserialize(output_slice).unwrap();
        results.push(result);

        if let Some(free) = desc.free_buffer {
            unsafe { free(out_ptr, out_len as usize) };
        }
    }

    assert!(results.contains(&"Hello, World!".to_string()));
    assert!(results.contains(&"Goodbye, World!".to_string()));
}
