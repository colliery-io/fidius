//! Test that async methods work with the fides macros.

use fides_macro::{plugin_impl, plugin_interface};

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

fides_core::fides_plugin_registry!();

#[test]
fn can_call_async_method_via_vtable() {
    let reg = fides_core::registry::get_registry();
    let desc = unsafe { &**reg.descriptors };
    let vtable = unsafe { &*(desc.vtable as *const AsyncProcessor_VTable) };

    let input = "hello".to_string();
    let input_bytes = fides_core::wire::serialize(&input).unwrap();

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
    let result: String = fides_core::wire::deserialize(output_slice).unwrap();
    assert_eq!(result, "processed: hello");

    if let Some(free) = desc.free_buffer {
        unsafe { free(out_ptr, out_len as usize) };
    }
}
