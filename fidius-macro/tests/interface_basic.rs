//! Basic test that #[plugin_interface] compiles and generates expected items.

use fidius_macro::plugin_interface;

#[plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;

    #[optional(since = 2)]
    fn greet_fancy(&self, name: String) -> String;
}

#[test]
fn vtable_struct_exists() {
    // The macro should generate Greeter_VTable
    let _size = std::mem::size_of::<Greeter_VTable>();
    // Required field is a function pointer (not Option)
    // Optional field is Option<fn pointer>
}

#[test]
fn interface_hash_is_nonzero() {
    assert_ne!(Greeter_INTERFACE_HASH, 0);
}

#[test]
fn interface_version_matches() {
    assert_eq!(Greeter_INTERFACE_VERSION, 1);
}

#[test]
fn buffer_strategy_matches() {
    assert_eq!(Greeter_BUFFER_STRATEGY, 1); // PluginAllocated = 1
}

#[test]
fn capability_constant_exists() {
    // The optional method `greet_fancy` should get bit 0
    assert_eq!(Greeter_CAP_GREET_FANCY, 1u64);
}
