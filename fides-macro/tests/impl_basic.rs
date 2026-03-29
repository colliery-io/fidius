//! Test that #[plugin_impl] compiles and generates expected items.

use fides_macro::{plugin_impl, plugin_interface};

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
fides_core::fides_plugin_registry!();

fn get_registry() -> &'static fides_core::descriptor::PluginRegistry {
    fides_core::registry::get_registry()
}

#[test]
fn registry_exists_and_is_valid() {
    let reg = get_registry();
    assert_eq!(&reg.magic, b"FIDES\0\0\0");
    assert_eq!(reg.registry_version, 1);
    assert_eq!(reg.plugin_count, 1);
}

#[test]
fn descriptor_fields_are_correct() {
    let reg = get_registry();
    let desc = unsafe { &**reg.descriptors };
    assert_eq!(desc.abi_version, 1);
    assert_eq!(desc.interface_hash, Greeter_INTERFACE_HASH);
    assert_eq!(desc.interface_version, 1);
    assert_eq!(desc.buffer_strategy, 1); // PluginAllocated
    assert!(desc.free_buffer.is_some());
}

#[test]
fn can_call_shim_via_vtable() {
    let reg = get_registry();
    let desc = unsafe { &**reg.descriptors };
    let vtable = unsafe { &*(desc.vtable as *const Greeter_VTable) };

    // Serialize the input argument
    let input = "World".to_string();
    let input_bytes = fides_core::wire::serialize(&input).unwrap();

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
    let result: String = fides_core::wire::deserialize(output_slice).unwrap();
    assert_eq!(result, "Hello, World!");

    // Free the buffer
    if let Some(free) = desc.free_buffer {
        unsafe { free(out_ptr, out_len as usize) };
    }
}
