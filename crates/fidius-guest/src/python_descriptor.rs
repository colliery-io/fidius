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

//! Compile-time descriptor of a fidius interface used by the Python loader.
//!
//! The cdylib path doesn't need any of this: the dylib carries its own
//! `PluginRegistry` + vtable that the host introspects at load time. A
//! Python plugin doesn't have a vtable; the host needs an out-of-band hint
//! about which method names exist on the trait, in what order, and with
//! which wire mode. The `#[plugin_interface]` macro emits a
//! `PythonInterfaceDescriptor` const into its companion module to provide
//! exactly that.
//!
//! The descriptor is `'static`-shaped (string slices, slice of structs) so
//! it can sit in the binary's `.rodata` and be referenced freely.

/// Static descriptor for one fidius interface, consumed by the Python
/// loader to validate and dispatch into a Python plugin.
#[derive(Debug, Clone, Copy)]
pub struct PythonInterfaceDescriptor {
    /// Trait name, used for diagnostics only.
    pub interface_name: &'static str,
    /// Same hash the cdylib path baked into its `PluginDescriptor`. The
    /// Python plugin's `__interface_hash__` constant must match this.
    pub interface_hash: u64,
    /// Methods in declaration order — the index here lines up with the
    /// vtable index the cdylib path uses for the same trait. The Python
    /// loader looks up callables in this order so `call_method(i, ...)`
    /// dispatches to the right Python function.
    pub methods: &'static [PythonMethodDesc],
}

/// One method on the interface.
#[derive(Debug, Clone, Copy)]
pub struct PythonMethodDesc {
    /// Function name to look up in the Python plugin module.
    pub name: &'static str,
    /// Whether this method uses raw byte-passthrough wire mode
    /// (`#[wire(raw)]`). Determines whether the dispatcher routes through
    /// `call_method_raw` (raw bytes both sides) or `call_method` (typed
    /// args via JSON conversion).
    pub wire_raw: bool,
}
