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

//! Host-function (plugin → host callback) FFI types — the stable C ABI
//! contract for the reverse direction (FIDIUS host-callback channel).
//!
//! The existing vtable direction lets the **host call the plugin**. This
//! module defines the mirror: a [`HostFunctionTable`] the host hands to a
//! plugin at bind time, through which **plugin code calls back into the
//! host** mid-execution. Arguments and returns are bincode-serialized with
//! the same wire conventions as the host → plugin direction (args as a
//! tuple, returns as the bare value, errors as [`PluginError`]).
//!
//! ## Shape
//!
//! One `#[repr(C)]` table per host interface, containing a single indexed
//! `dispatch` entry point plus identity fields (interface name, FNV-1a hash
//! of the method signatures, and a user-declared version). The plugin
//! advertises which host interfaces it wants via [`HostImportDescriptor`]s
//! collected behind the optional `fidius_get_host_imports` export; the host
//! validates version + hash and installs the table through the descriptor's
//! `bind` function. Both sides check: the host refuses to bind a mismatched
//! import ([`LoadError`]-level failure), and the plugin-side `bind` shim
//! re-validates the table before storing it — a mismatch can therefore never
//! reach `dispatch`, where positional bincode would corrupt arguments.
//!
//! ## Memory ownership
//!
//! Output buffers are allocated by the **host** (the callee) as `Box<[u8]>`
//! and freed by the host via the table's `free_buffer` — the plugin calls it
//! after copying the bytes out, so allocations never cross allocator
//! boundaries. This mirrors (in reverse) the `PluginAllocated` strategy of
//! the forward direction.
//!
//! ## Threading and reentrancy (the contract)
//!
//! See the `#[fidius::host_interface]` macro docs for the full contract.
//! The load-bearing rules:
//!
//! - Host functions are **synchronous** at the boundary. A host
//!   implementation that needs to await must bridge to its own runtime
//!   internally (e.g. `tokio::runtime::Handle::block_on`) and must tolerate
//!   being called from threads it does not own — including plugin-spawned
//!   threads and, most importantly, the host's own thread **while a
//!   host → plugin call is live on that stack** (host → plugin → host).
//! - The host must **not hold any lock across a plugin call** that a host
//!   function implementation could try to acquire — the callback re-enters
//!   the host while the plugin call is still on the stack, and that lock
//!   acquisition will deadlock.
//! - Host implementations must be `Send + Sync`: multiple plugin threads
//!   may call concurrently.
//!
//! [`host_callback_depth`] exposes a per-thread depth counter maintained by
//! the generated dispatch shims so host implementations can *detect*
//! reentrancy (e.g. `debug_assert!` invariants about which locks may be
//! taken at depth > 0).

use std::ffi::{c_char, c_void};
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::error::PluginError;

/// Current layout version of the [`HostImportRegistry`] struct.
pub const HOST_IMPORTS_VERSION: u32 = 1;

/// Name of the optional symbol a plugin dylib exports to advertise the host
/// interfaces it can consume. Emitted by `fidius_plugin_registry!()`; absent
/// from plugins built before the host-callback channel existed — the host
/// treats absence as "no host imports".
pub const HOST_IMPORTS_SYMBOL: &str = "fidius_get_host_imports";

// ── Bind status codes ───────────────────────────────────────────────────────
// Returned by a plugin-side `bind` shim (see `HostImportDescriptor::bind`).

/// The table was validated and installed.
pub const BIND_OK: i32 = 0;
/// The table pointer was null.
pub const BIND_ERR_NULL: i32 = -1;
/// A table for this interface was already installed (bind is once-only).
pub const BIND_ERR_ALREADY_BOUND: i32 = -2;
/// The table's `interface_hash` does not match what the plugin was built
/// against — the signatures drifted.
pub const BIND_ERR_HASH_MISMATCH: i32 = -3;
/// The table's `interface_version` does not match what the plugin expects.
pub const BIND_ERR_VERSION_MISMATCH: i32 = -4;
/// The table was built by an incompatible fidius ABI.
pub const BIND_ERR_ABI: i32 = -5;
/// The table's `fn_count` does not match the interface's method count.
pub const BIND_ERR_FN_COUNT: i32 = -6;
/// The bind shim panicked (never expected; defensive).
pub const BIND_ERR_PANIC: i32 = -7;
/// The handle's backend cannot accept this table (e.g. a wasm-only bind
/// entry point called on a cdylib/Python-backed handle, or vice versa).
pub const BIND_ERR_WRONG_BACKEND: i32 = -8;

// ── WASM host-call channel ──────────────────────────────────────────────────
// For WASM plugins, host functions cross as the `fidius:host-call/host@0.1.0`
// component import: `call(interface-name, expected-version, expected-hash,
// index, args) -> (status, payload)`. The identity triple travels with every
// call, so the version/hash gate is enforced per call (a u64 compare) — same
// guarantee as the dylib bind-time gate (typed loud failure, never a
// mis-dispatch of positional bincode), enforced at the first call instead of
// at instantiation (a component can't be introspected for the host
// interfaces its method bodies use).

/// Reserved dispatch index for the bind-probe: `call(..., PROBE, [])` runs
/// the name/version/hash gate and returns `STATUS_OK` with an empty payload
/// without dispatching a real function. Generated wasm clients use it to
/// implement `bound()` / `is_bound()`.
pub const HOST_CALL_PROBE_INDEX: u32 = u32::MAX;

/// host-call status: no host interface with this name is bound.
pub const HOST_CALL_STATUS_NOT_BOUND: i32 = -7;
/// host-call status: the bound table's version differs from what the plugin
/// was built against. Payload = bincode `(plugin_expects: u32, host_provides: u32)`.
pub const HOST_CALL_STATUS_VERSION_MISMATCH: i32 = -8;
/// host-call status: the bound table's signature hash differs. Payload =
/// bincode `(plugin_expects: u64, host_provides: u64)`.
pub const HOST_CALL_STATUS_HASH_MISMATCH: i32 = -9;

/// The dispatch signature of a host-function table: `(ctx, fn_index, in_ptr,
/// in_len, out_ptr, out_len) -> status`. Input is the bincode of the argument
/// tuple; on `STATUS_OK`/`STATUS_PLUGIN_ERROR`/`STATUS_PANIC` the host writes
/// a host-owned buffer to `out_ptr`/`out_len` which the caller must release
/// via [`HostFunctionTable::free_buffer`].
pub type HostDispatchFn =
    unsafe extern "C" fn(*mut c_void, u32, *const u8, u32, *mut *mut u8, *mut u32) -> i32;

/// A set of host functions offered to a plugin, as a C-ABI table.
///
/// Built by the host side of a `#[fidius::host_interface]` (the generated
/// `<Trait>Binding`) and installed into the plugin through the plugin's
/// [`HostImportDescriptor::bind`]. The table and everything it references
/// must live for the remainder of the process (the generated binding leaks
/// one table + one `Arc<dyn Trait>` per bind — a few dozen bytes, once per
/// loaded library).
///
/// # Safety
///
/// - `interface_name` points to a static, null-terminated UTF-8 C string.
/// - `dispatch` and `free_buffer` are valid for the process lifetime.
/// - `ctx` is an opaque pointer owned by the table; the plugin passes it to
///   every `dispatch` call and never dereferences or frees it.
#[repr(C)]
pub struct HostFunctionTable {
    /// Size in bytes of this struct at host build time. Read first; any
    /// field at an offset >= `table_size` is absent in the host's build
    /// (post-1.0 forward-compat, mirroring `PluginDescriptor`).
    pub table_size: u32,
    /// Must equal [`crate::descriptor::ABI_VERSION`].
    pub abi_version: u32,
    /// Null-terminated name of the host interface trait (e.g. `"CloacinaHost"`).
    pub interface_name: *const c_char,
    /// FNV-1a hash of the host interface's method signatures.
    pub interface_hash: u64,
    /// User-declared version from `#[host_interface(version = N)]`.
    pub interface_version: u32,
    /// Number of functions dispatchable through this table.
    pub fn_count: u32,
    /// Opaque host context, passed as the first argument to every `dispatch`.
    pub ctx: *mut c_void,
    /// The single indexed dispatch entry point.
    pub dispatch: HostDispatchFn,
    /// Frees a buffer previously returned through `dispatch`'s out params.
    /// Host-allocated memory is host-freed — never mixed across allocators.
    pub free_buffer: unsafe extern "C" fn(*mut u8, usize),
}

// SAFETY: all fields are primitives, function pointers, or pointers to static
// / process-lifetime data. `ctx` is only ever handed back to host code.
unsafe impl Send for HostFunctionTable {}
unsafe impl Sync for HostFunctionTable {}

/// A plugin's declaration that it can consume a host interface.
///
/// Generated (one per `#[host_interface]` trait) into the interface crate and
/// collected behind the plugin's `fidius_get_host_imports` export. The host
/// reads the identity fields to gate version/hash **before** calling `bind`.
#[repr(C)]
pub struct HostImportDescriptor {
    /// Size in bytes of this struct at plugin build time (forward-compat).
    pub descriptor_size: u32,
    /// Must equal [`crate::descriptor::ABI_VERSION`].
    pub abi_version: u32,
    /// Null-terminated name of the host interface trait.
    pub interface_name: *const c_char,
    /// FNV-1a hash of the method signatures the plugin was built against.
    pub interface_hash: u64,
    /// The `#[host_interface(version = N)]` the plugin was built against.
    pub interface_version: u32,
    /// Installs a validated [`HostFunctionTable`]. Returns a `BIND_*` status.
    /// The shim re-validates hash/version/ABI defensively before storing.
    pub bind: unsafe extern "C" fn(*const HostFunctionTable) -> i32,
}

// SAFETY: static identity data + a function pointer into the plugin.
unsafe impl Send for HostImportDescriptor {}
unsafe impl Sync for HostImportDescriptor {}

/// Registry of all host-import declarations in a plugin dylib, returned by
/// the optional `fidius_get_host_imports` export.
#[repr(C)]
pub struct HostImportRegistry {
    /// Must equal [`crate::descriptor::FIDIUS_MAGIC`].
    pub magic: [u8; 8],
    /// Must equal [`HOST_IMPORTS_VERSION`].
    pub registry_version: u32,
    /// Number of import descriptors.
    pub import_count: u32,
    /// Pointer to an array of `import_count` descriptor pointers.
    pub imports: *const *const HostImportDescriptor,
}

// SAFETY: immutable static data, same argument as `PluginRegistry`.
unsafe impl Send for HostImportRegistry {}
unsafe impl Sync for HostImportRegistry {}

/// A `Sync` wrapper for a raw `HostImportDescriptor` pointer, for use in
/// `static` arrays (mirrors `descriptor::DescriptorPtr`).
#[repr(transparent)]
pub struct HostImportPtr(pub *const HostImportDescriptor);

// SAFETY: points at static, immutable data.
unsafe impl Send for HostImportPtr {}
unsafe impl Sync for HostImportPtr {}

/// The once-only cell a plugin stores its bound [`HostFunctionTable`] in.
///
/// One static cell per `#[host_interface]` trait is generated into the
/// interface crate. `bind` installs a table exactly once (subsequent binds
/// report [`BIND_ERR_ALREADY_BOUND`] rather than swapping a table out from
/// under in-flight calls); `get` is a relaxed-cost atomic load on the call
/// path.
pub struct HostTableCell(AtomicPtr<HostFunctionTable>);

impl HostTableCell {
    /// An empty (unbound) cell, for `static` initialization.
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self(AtomicPtr::new(std::ptr::null_mut()))
    }

    /// Install `table` if the cell is empty. Returns [`BIND_OK`] on success
    /// or [`BIND_ERR_ALREADY_BOUND`] if a table was already installed.
    ///
    /// # Safety
    /// `table` must be non-null and valid for the remainder of the process.
    pub unsafe fn bind(&self, table: *const HostFunctionTable) -> i32 {
        match self.0.compare_exchange(
            std::ptr::null_mut(),
            table as *mut HostFunctionTable,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => BIND_OK,
            Err(_) => BIND_ERR_ALREADY_BOUND,
        }
    }

    /// The bound table, or `None` if the host never bound this interface.
    pub fn get(&self) -> Option<&HostFunctionTable> {
        let ptr = self.0.load(Ordering::Acquire);
        // SAFETY: a non-null pointer was installed by `bind`, whose contract
        // requires process-lifetime validity.
        unsafe { ptr.cast_const().as_ref() }
    }
}

/// Error returned to plugin code when a host-function call fails.
///
/// This is the plugin-side error surface of the callback channel — every
/// failure mode (unbound interface, host-raised typed error, host panic,
/// serialization fault) arrives as a variant here, never as a panic and
/// never as unwinding across the FFI boundary.
#[derive(Debug, thiserror::Error)]
pub enum HostCallError {
    /// The host never bound this interface for this plugin. Either the host
    /// application doesn't provide it, or it skipped binding after load.
    #[error("host interface '{interface}' is not bound — the host did not provide it")]
    NotBound {
        /// The host interface (trait) name.
        interface: &'static str,
    },

    /// The host function returned a typed error ([`PluginError`] is the
    /// shared error currency in both directions).
    #[error("host function error: {0}")]
    Host(PluginError),

    /// The host function panicked; the panic was caught at the boundary.
    #[error("host function panicked: {0}")]
    HostPanic(String),

    /// Argument serialization failed on the plugin side, or the host
    /// reported it could not decode the arguments / encode the result.
    #[error("host call serialization error: {0}")]
    Serialization(String),

    /// The host's response bytes did not decode as the expected type.
    #[error("host call deserialization error: {0}")]
    Deserialization(String),

    /// The dispatch index was out of range for the host's table. Cannot
    /// happen through generated clients when the bind-time gate passed.
    #[error("invalid host function index {index}")]
    InvalidIndex { index: u32 },

    /// The host returned a status code this plugin doesn't know.
    #[error("unknown host status code: {code}")]
    UnknownStatus { code: i32 },

    /// The host provides a different **version** of this interface than the
    /// plugin was built against. WASM host-call channel only — on the dylib
    /// path the same mismatch fails the host's bind at load, so plugin code
    /// sees [`HostCallError::NotBound`] instead.
    #[error("host interface '{interface}' version mismatch: plugin was built against v{plugin_expects}, host provides v{host_provides}")]
    VersionMismatch {
        interface: &'static str,
        plugin_expects: u32,
        host_provides: u32,
    },

    /// The host provides a different **signature set** (hash) of this
    /// interface than the plugin was built against. WASM host-call channel
    /// only; dylib mismatches fail the host's bind at load.
    #[error("host interface '{interface}' signature hash mismatch: plugin was built against {plugin_expects:#x}, host provides {host_provides:#x}")]
    HashMismatch {
        interface: &'static str,
        plugin_expects: u64,
        host_provides: u64,
    },
}

/// Decode a `(status, payload)` pair from the WASM `fidius:host-call` import
/// into the plugin-side result. Compiles natively too so the mapping is
/// unit-testable without a component build.
pub fn decode_host_call_status(
    interface: &'static str,
    index: u32,
    status: i32,
    payload: Vec<u8>,
) -> Result<Vec<u8>, HostCallError> {
    use crate::status::*;
    match status {
        STATUS_OK => Ok(payload),
        STATUS_PLUGIN_ERROR => {
            let err: PluginError = crate::wire::deserialize(&payload).unwrap_or_else(|_| {
                PluginError::new(
                    "UNKNOWN",
                    "host returned an error but no decodable error data",
                )
            });
            Err(HostCallError::Host(err))
        }
        STATUS_PANIC => {
            let msg = if payload.is_empty() {
                "unknown panic".to_string()
            } else {
                crate::wire::deserialize::<String>(&payload)
                    .unwrap_or_else(|_| "unknown panic".into())
            };
            Err(HostCallError::HostPanic(msg))
        }
        STATUS_SERIALIZATION_ERROR => Err(HostCallError::Serialization(
            "host could not decode arguments or encode the result".into(),
        )),
        STATUS_INVALID_INDEX => Err(HostCallError::InvalidIndex { index }),
        HOST_CALL_STATUS_NOT_BOUND => Err(HostCallError::NotBound { interface }),
        HOST_CALL_STATUS_VERSION_MISMATCH => {
            let (plugin_expects, host_provides): (u32, u32) =
                crate::wire::deserialize(&payload).unwrap_or((0, 0));
            Err(HostCallError::VersionMismatch {
                interface,
                plugin_expects,
                host_provides,
            })
        }
        HOST_CALL_STATUS_HASH_MISMATCH => {
            let (plugin_expects, host_provides): (u64, u64) =
                crate::wire::deserialize(&payload).unwrap_or((0, 0));
            Err(HostCallError::HashMismatch {
                interface,
                plugin_expects,
                host_provides,
            })
        }
        code => Err(HostCallError::UnknownStatus { code }),
    }
}

/// Error installing a [`HostFunctionTable`] into a plugin — the typed form
/// of a non-`BIND_OK` status from a plugin's bind shim.
#[derive(Debug, thiserror::Error)]
#[error("binding host interface '{interface}' failed: {message} (status {code})")]
pub struct HostBindError {
    /// The host interface (trait) name.
    pub interface: &'static str,
    /// The raw `BIND_*` status code.
    pub code: i32,
    /// Human-readable description of the status.
    pub message: &'static str,
}

/// Map a `BIND_*` status to a `Result` for host-side bind entry points.
pub fn bind_status_to_result(interface: &'static str, code: i32) -> Result<(), HostBindError> {
    if code == BIND_OK {
        Ok(())
    } else {
        Err(HostBindError {
            interface,
            code,
            message: bind_status_message(code),
        })
    }
}

/// Map a `BIND_*` status to a human-readable description (for error paths).
pub fn bind_status_message(code: i32) -> &'static str {
    match code {
        BIND_OK => "ok",
        BIND_ERR_NULL => "null table pointer",
        BIND_ERR_ALREADY_BOUND => "a host table is already bound for this interface",
        BIND_ERR_HASH_MISMATCH => "interface hash mismatch between plugin and host table",
        BIND_ERR_VERSION_MISMATCH => "interface version mismatch between plugin and host table",
        BIND_ERR_ABI => "fidius ABI version mismatch",
        BIND_ERR_FN_COUNT => "host table fn_count does not match the interface",
        BIND_ERR_PANIC => "plugin bind shim panicked",
        _ => "unknown bind status",
    }
}

/// Per-thread depth of live host-function dispatches.
///
/// Incremented by the generated dispatch shim for the duration of each host
/// function call. `0` means "not inside a host callback"; `1` is the normal
/// depth while a host function runs; `> 1` means a host function (directly
/// or via a nested plugin call) triggered another host function on the same
/// thread. Host implementations can use this to assert lock-discipline
/// invariants — e.g. `debug_assert!(host_callback_depth() <= 1)` in code
/// that must never be reached reentrantly.
pub fn host_callback_depth() -> usize {
    CALLBACK_DEPTH.with(|d| d.get())
}

thread_local! {
    static CALLBACK_DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// RAII guard incrementing [`host_callback_depth`] — used by the generated
/// host-side dispatch shims. Public so macro-generated code can name it; not
/// intended for direct use.
pub struct CallbackDepthGuard(());

impl CallbackDepthGuard {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        CALLBACK_DEPTH.with(|d| d.set(d.get() + 1));
        Self(())
    }
}

impl Drop for CallbackDepthGuard {
    fn drop(&mut self) {
        CALLBACK_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
    }
}

/// Invoke a host function through a bound table: bincode-of-args in,
/// bincode-of-result out, with the status protocol mapped to
/// [`HostCallError`]. Shared by every generated `<Trait>Client` method.
///
/// On `STATUS_OK` the host's output buffer is copied into a `Vec<u8>` and
/// released via the table's `free_buffer`; error payloads are decoded and
/// released the same way. Never panics on malformed host output — decoding
/// failures surface as [`HostCallError::Deserialization`].
pub fn call_host_fn(
    table: &HostFunctionTable,
    index: u32,
    input: &[u8],
) -> Result<Vec<u8>, HostCallError> {
    use crate::status::*;

    let mut out_ptr: *mut u8 = std::ptr::null_mut();
    let mut out_len: u32 = 0;

    // SAFETY: `table` was validated at bind time; dispatch/free_buffer are
    // process-lifetime function pointers per the HostFunctionTable contract.
    let status = unsafe {
        (table.dispatch)(
            table.ctx,
            index,
            input.as_ptr(),
            input.len() as u32,
            &mut out_ptr,
            &mut out_len,
        )
    };

    // Copy-then-free helper for host-owned output buffers.
    let take_output = |out_ptr: *mut u8, out_len: u32| -> Vec<u8> {
        if out_ptr.is_null() || out_len == 0 {
            return Vec::new();
        }
        // SAFETY: the host wrote a valid buffer of out_len bytes; we copy it
        // out and hand it straight back to the host's free_buffer.
        unsafe {
            let bytes = std::slice::from_raw_parts(out_ptr, out_len as usize).to_vec();
            (table.free_buffer)(out_ptr, out_len as usize);
            bytes
        }
    };

    match status {
        STATUS_OK => Ok(take_output(out_ptr, out_len)),
        STATUS_PLUGIN_ERROR => {
            let bytes = take_output(out_ptr, out_len);
            let err: PluginError = crate::wire::deserialize(&bytes).unwrap_or_else(|_| {
                PluginError::new(
                    "UNKNOWN",
                    "host returned an error but no decodable error data",
                )
            });
            Err(HostCallError::Host(err))
        }
        STATUS_PANIC => {
            let bytes = take_output(out_ptr, out_len);
            let msg = if bytes.is_empty() {
                "unknown panic".to_string()
            } else {
                crate::wire::deserialize::<String>(&bytes)
                    .unwrap_or_else(|_| "unknown panic".into())
            };
            Err(HostCallError::HostPanic(msg))
        }
        STATUS_SERIALIZATION_ERROR => Err(HostCallError::Serialization(
            "host could not decode arguments or encode the result".into(),
        )),
        STATUS_INVALID_INDEX => Err(HostCallError::InvalidIndex { index }),
        code => Err(HostCallError::UnknownStatus { code }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_binds_exactly_once() {
        static CELL: HostTableCell = HostTableCell::new();
        assert!(CELL.get().is_none());

        // A minimal table living for the test-process lifetime.
        unsafe extern "C" fn dispatch(
            _: *mut c_void,
            _: u32,
            _: *const u8,
            _: u32,
            _: *mut *mut u8,
            _: *mut u32,
        ) -> i32 {
            0
        }
        unsafe extern "C" fn free_buffer(_: *mut u8, _: usize) {}
        let table = Box::leak(Box::new(HostFunctionTable {
            table_size: std::mem::size_of::<HostFunctionTable>() as u32,
            abi_version: crate::descriptor::ABI_VERSION,
            interface_name: c"Test".as_ptr(),
            interface_hash: 1,
            interface_version: 1,
            fn_count: 0,
            ctx: std::ptr::null_mut(),
            dispatch,
            free_buffer,
        }));

        assert_eq!(unsafe { CELL.bind(table) }, BIND_OK);
        assert!(CELL.get().is_some());
        assert_eq!(unsafe { CELL.bind(table) }, BIND_ERR_ALREADY_BOUND);
    }

    #[test]
    fn depth_guard_nests_and_unwinds() {
        assert_eq!(host_callback_depth(), 0);
        {
            let _g1 = CallbackDepthGuard::new();
            assert_eq!(host_callback_depth(), 1);
            {
                let _g2 = CallbackDepthGuard::new();
                assert_eq!(host_callback_depth(), 2);
            }
            assert_eq!(host_callback_depth(), 1);
        }
        assert_eq!(host_callback_depth(), 0);
    }
}
