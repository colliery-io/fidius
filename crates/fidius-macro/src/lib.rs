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

mod host_interface;
mod impl_macro;
mod interface;
mod ir;
mod wit;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemImpl, ItemTrait};

use host_interface::HostInterfaceAttrs;
use impl_macro::PluginImplAttrs;
use ir::InterfaceAttrs;

/// Define a plugin interface from a trait.
///
/// Generates a `#[repr(C)]` vtable struct, interface hash constant,
/// capability bit constants, and a descriptor builder function.
///
/// # Example
///
/// ```ignore
/// #[plugin_interface(version = 1, buffer = PluginAllocated)]
/// pub trait Greeter: Send + Sync {
///     fn greet(&self, name: String) -> String;
///
///     #[optional(since = 2)]
///     fn greet_fancy(&self, name: String) -> String;
/// }
/// ```
#[proc_macro_attribute]
pub fn plugin_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as InterfaceAttrs);
    let item_trait = parse_macro_input!(item as ItemTrait);

    match ir::parse_interface(attrs, &item_trait) {
        Ok(ir) => match interface::generate_interface(&ir) {
            Ok(tokens) => tokens.into(),
            Err(err) => err.to_compile_error().into(),
        },
        Err(err) => err.to_compile_error().into(),
    }
}

/// Define a **host interface** from a trait — the plugin → host callback
/// channel (the reverse direction of [`macro@plugin_interface`]).
///
/// The host implements the trait and offers it to plugins as a C-ABI
/// function table; plugin code calls back into the host mid-execution
/// through a generated typed client. Arguments and returns are
/// bincode-serialized with the same wire conventions as the host → plugin
/// direction.
///
/// # Example
///
/// ```ignore
/// // Interface crate — shared by host and plugins:
/// #[fidius::host_interface(version = 1)]
/// pub trait CloacinaHost: Send + Sync {
///     fn release_slot(&self, task_execution_id: String) -> Result<(), PluginError>;
///     fn reclaim_slot(&self, task_execution_id: String) -> Result<(), PluginError>;
/// }
///
/// // Plugin code — call the host mid-execution:
/// let host = CloacinaHostClient::bound()?;   // Err(NotBound) if the host didn't bind
/// host.release_slot(&id)?;
///
/// // Host application — implement + bind after loading the plugin library:
/// let lib = fidius_host::loader::load_library(path)?;
/// CloacinaHostBinding::bind(&lib, std::sync::Arc::new(MyHost { .. }))?;
/// ```
///
/// # Versioning
///
/// The host-function surface is gated at **bind time** (during plugin
/// load) on two axes: the declared `version = N` and an FNV-1a hash of the
/// method signatures. A plugin built against a different version or a
/// drifted signature set fails the bind with a typed `LoadError` — the
/// table is never installed, so a mismatched surface can never mis-dispatch
/// positional bincode. An unbound interface surfaces to plugin code as the
/// typed `HostCallError::NotBound`, never a crash.
///
/// # Threading, blocking, and reentrancy contract
///
/// Host functions are **synchronous** at the FFI boundary and may be
/// invoked from any plugin thread — including plugin-owned tokio runtime
/// threads, and including the host's own calling thread while a
/// host → plugin call is live on that stack (host → plugin → host).
///
/// **Host implementations must:**
/// - be `Send + Sync` (the trait must declare those supertraits) and
///   tolerate concurrent calls from multiple plugin threads;
/// - bridge to async internally if needed (e.g.
///   `tokio::runtime::Handle::block_on` on a handle to the host's runtime).
///   The calling plugin thread simply blocks — it is never a host runtime
///   worker unless the *host* called the plugin from one, which the next
///   rule forbids;
/// - be callable without any host lock held: the **host must not hold a
///   lock, runtime worker thread, or other non-reentrant resource across a
///   plugin call** if any host function could need it — the callback
///   re-enters the host while the plugin call is still on the stack, and
///   that acquisition deadlocks. Call plugins from dedicated or blocking
///   threads (`spawn_blocking`), never from async executor workers a host
///   function might need to make progress.
///
/// **Plugin code may**, while one of its methods executes: call host
/// functions from the method's thread or from threads/runtimes it spawned;
/// call several host functions concurrently. **Plugin code must not** stash
/// the client and call host functions after its originating library could
/// be unloaded, and must expect host functions to block (e.g.
/// `reclaim_slot` waiting for capacity).
///
/// `fidius::host_ffi::host_callback_depth()` exposes a per-thread depth
/// counter (0 outside callbacks) that host implementations can use in
/// debug assertions to detect unexpected reentrancy.
///
/// # Panic safety
///
/// A panic in a host function is caught at the boundary and surfaces to
/// the plugin as `HostCallError::HostPanic(message)`; a panic in the
/// plugin method that made the call is caught by the existing plugin-shim
/// guard and surfaces to the host as `CallError::Panic`. Unwinding never
/// crosses the FFI boundary in either direction.
///
/// # WASM
///
/// v1 of the host-function channel is **dylib-only**: for WASM plugins,
/// host functions are component imports and need a different mechanism.
/// The API is shaped so that can be added later without breaking changes
/// (the identity triple `name`/`version`/`hash` and the bincode
/// request/response map directly onto a future `fidius:host-call` import).
/// On a wasm build this macro emits only the constants; the client,
/// binding, and table machinery are compiled out.
#[proc_macro_attribute]
pub fn host_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as HostInterfaceAttrs);
    let item_trait = parse_macro_input!(item as ItemTrait);

    match host_interface::generate_host_interface(&attrs, &item_trait) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Implement a plugin interface for a concrete type.
///
/// Generates extern "C" FFI shims, a static vtable, a plugin descriptor,
/// and a plugin registry.
///
/// # Example
///
/// ```ignore
/// pub struct MyGreeter;
///
/// #[plugin_impl(Greeter)]
/// impl Greeter for MyGreeter {
///     fn greet(&self, name: String) -> String {
///         format!("Hello, {name}!")
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn plugin_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as PluginImplAttrs);
    let item_impl = parse_macro_input!(item as ItemImpl);

    match impl_macro::generate_plugin_impl(&attrs, &item_impl) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Mark a `struct`/`enum` as usable in a WASM plugin interface (FIDIUS-I-0023).
///
/// This is a **marker** derive: it emits no code. The `fidius wit` generator
/// (run from `build.rs`) keys on the `#[derive(WitType)]` attribute when it
/// parses the crate source, mapping the struct to a WIT `record` (named fields)
/// or the enum to a WIT `variant` (unit / single-field cases) and emitting the
/// generated↔author conversions the wasm adapter uses. The same type continues
/// to cross the cdylib/Python boundary via serde, unchanged.
///
/// ```ignore
/// #[derive(WitType, serde::Serialize, serde::Deserialize, Clone)]
/// pub struct Point { pub x: i32, pub y: i32 }
/// ```
#[proc_macro_derive(WitType)]
pub fn derive_wit_type(_item: TokenStream) -> TokenStream {
    // Intentionally empty — the build-time WIT generator reads the annotation
    // from source; no per-type codegen is needed here.
    TokenStream::new()
}
