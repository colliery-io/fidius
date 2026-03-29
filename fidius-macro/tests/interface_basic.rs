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
    // The macro should generate __fidius_Greeter::Greeter_VTable
    let _size = std::mem::size_of::<__fidius_Greeter::Greeter_VTable>();
    // Required field is a function pointer (not Option)
    // Optional field is Option<fn pointer>
}

#[test]
fn interface_hash_is_nonzero() {
    assert_ne!(__fidius_Greeter::Greeter_INTERFACE_HASH, 0);
}

#[test]
fn interface_version_matches() {
    assert_eq!(__fidius_Greeter::Greeter_INTERFACE_VERSION, 1);
}

#[test]
fn buffer_strategy_matches() {
    assert_eq!(__fidius_Greeter::Greeter_BUFFER_STRATEGY, 1); // PluginAllocated = 1
}

#[test]
fn capability_constant_exists() {
    // The optional method `greet_fancy` should get bit 0
    assert_eq!(__fidius_Greeter::Greeter_CAP_GREET_FANCY, 1u64);
}
