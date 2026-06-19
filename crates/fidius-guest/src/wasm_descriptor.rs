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

//! Compile-time descriptor of a fidius interface used by the WASM loader.
//!
//! Like [`crate::python_descriptor`], a WASM component has no host-readable
//! vtable, so the host needs an out-of-band hint: which exported interface the
//! component implements, the method names in declaration (vtable) order with
//! their wire mode, and the expected interface hash. The `#[plugin_interface]`
//! macro emits this (Phase 3); for Phase 2 it is hand-authored alongside the
//! reference WIT.

/// Static descriptor for one fidius interface, consumed by the WASM loader to
/// validate and dispatch into a component.
#[derive(Debug, Clone, Copy)]
pub struct WasmInterfaceDescriptor {
    /// Trait name, for diagnostics.
    pub interface_name: &'static str,
    /// Fully-qualified exported interface the component must provide, e.g.
    /// `"fidius:greeter/greeter@1.0.0"`. The host navigates to this interface's
    /// exports to dispatch methods.
    pub interface_export: &'static str,
    /// Same hash the cdylib path bakes into its `PluginDescriptor`. The
    /// component's `fidius-interface-hash` export must return this.
    pub interface_hash: u64,
    /// Methods in declaration order — index here lines up with the cdylib
    /// vtable index for the same trait.
    pub methods: &'static [WasmMethodDesc],
}

/// One method on the interface.
#[derive(Debug, Clone, Copy)]
pub struct WasmMethodDesc {
    /// Export name within the interface (e.g. `"greet"`).
    pub name: &'static str,
    /// Whether this method uses raw byte-passthrough wire mode (`#[wire(raw)]`).
    pub wire_raw: bool,
    /// Whether this method is **server-streaming** (`-> fidius::Stream<T>`,
    /// FIDIUS-I-0026). When true the export returns a `next()`-pollable resource
    /// rather than a value, so the host routes it through the streaming path.
    pub streaming: bool,
}
