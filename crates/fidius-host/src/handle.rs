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

//! `PluginHandle` — the unified, caller-facing proxy over a loaded plugin.
//!
//! A `PluginHandle` is backend-agnostic: callers use the same
//! `call_method` / `call_method_raw` API whether the plugin is a cdylib, a
//! Python package, or (Phase 2) a WASM component. The backend lives in the
//! private [`Backend`] enum.
//!
//! ## Why an enum backend (FIDIUS-I-0021)
//!
//! The backends don't share a typed wire: cdylib decodes concrete-type
//! **bincode** (not reconstructable from an erased value) while Python/WASM
//! consume a self-describing [`fidius_core::Value`]. An enum (rather than
//! `Box<dyn PluginExecutor>`) lets the generic `call_method<I, O>` branch with
//! the concrete `I`/`O` in scope and serialise with each backend's native
//! currency — so the **cdylib path stays byte-identical** to before this
//! refactor (`bincode(input)` straight to the FFI; `Value` is never involved).

use serde::de::DeserializeOwned;
use serde::Serialize;

use fidius_core::descriptor::PluginDescriptor;

use crate::error::{CallError, LoadError};
use crate::executor::cdylib::CdylibExecutor;
#[cfg(feature = "python")]
use crate::executor::python::Pyo3Executor;
#[cfg(feature = "wasm")]
use crate::executor::wasm::WasmComponentExecutor;
#[cfg(any(feature = "python", feature = "wasm"))]
use crate::executor::{PluginExecutor, ValueExecutor};
use crate::types::PluginInfo;

/// The execution backend behind a [`PluginHandle`].
///
/// One variant per runtime. The WASM variant lands in Phase 2.
enum Backend {
    Cdylib(CdylibExecutor),
    /// `.py` package via `fidius-python`'s embedded interpreter. Only present
    /// when the `python` feature is enabled.
    #[cfg(feature = "python")]
    Python(Pyo3Executor),
    /// `.wasm` component via wasmtime. Only present when the `wasm` feature is
    /// enabled.
    #[cfg(feature = "wasm")]
    Wasm(WasmComponentExecutor),
}

/// A handle to a loaded plugin, ready for calling methods.
///
/// Holds the active execution backend. `call_method()` handles serialization,
/// dispatch, and cleanup; concurrent calls from multiple threads are safe as
/// long as the underlying plugin is thread-safe (the cdylib macro enforces
/// `&self`-only methods; the Python backend serialises through the GIL).
pub struct PluginHandle {
    backend: Backend,
}

impl PluginHandle {
    /// Create a `PluginHandle` from a freshly loaded cdylib plugin.
    pub fn from_loaded(plugin: crate::loader::LoadedPlugin) -> Self {
        Self {
            backend: Backend::Cdylib(CdylibExecutor::from_loaded(plugin)),
        }
    }

    /// Create a `PluginHandle` from a descriptor already registered in the
    /// current process's inventory (a `#[plugin_impl]` linked as a normal
    /// rlib). No dylib is loaded. Used by `Client::in_process(plugin_name)`.
    pub fn from_descriptor(desc: &'static PluginDescriptor) -> Result<Self, LoadError> {
        Ok(Self {
            backend: Backend::Cdylib(CdylibExecutor::from_descriptor(desc)?),
        })
    }

    /// Construct a **configured** in-process plugin instance (FIDIUS-A-0006 /
    /// CI.2): serialize `config` and bind it once at construction. The plugin's
    /// `#[plugin_impl(Trait, config = C)]` `configure` constructor receives it;
    /// methods then close over it without re-passing. The config crosses the
    /// boundary exactly once, and N differently-configured instances can coexist.
    pub fn configure_in_process<C: Serialize>(
        desc: &'static PluginDescriptor,
        config: &C,
    ) -> Result<Self, LoadError> {
        let cfg = fidius_core::wire::serialize(config)
            .map_err(|e| LoadError::ConfigSerialization(e.to_string()))?;
        Ok(Self {
            backend: Backend::Cdylib(CdylibExecutor::from_descriptor_with_config(desc, &cfg)?),
        })
    }

    /// Look up a descriptor in the current process's inventory registry by
    /// `plugin_name` (the Rust struct name passed to `#[plugin_impl]`).
    pub fn find_in_process_descriptor(
        plugin_name: &str,
    ) -> Result<&'static PluginDescriptor, LoadError> {
        CdylibExecutor::find_in_process_descriptor(plugin_name)
    }

    /// Create a `PluginHandle` backed by a loaded Python plugin. `info` is
    /// built by the loader from the package manifest + interface descriptor.
    /// Only available with the `python` feature.
    #[cfg(feature = "python")]
    pub fn from_python(py: fidius_python::PythonPluginHandle, info: PluginInfo) -> Self {
        Self {
            backend: Backend::Python(Pyo3Executor::new(py, info)),
        }
    }

    /// Create a `PluginHandle` backed by a loaded WASM component. Only
    /// available with the `wasm` feature.
    #[cfg(feature = "wasm")]
    pub fn from_wasm(executor: WasmComponentExecutor) -> Self {
        Self {
            backend: Backend::Wasm(executor),
        }
    }

    /// Call a plugin method by vtable index.
    ///
    /// Serializes the input with the backend's native wire (cdylib → bincode;
    /// Python/WASM → [`fidius_core::Value`]), dispatches, and decodes the
    /// result into `O`. No built-in timeout — see the `fidius` crate docs.
    pub fn call_method<I: Serialize, O: DeserializeOwned>(
        &self,
        index: usize,
        input: &I,
    ) -> Result<O, CallError> {
        match &self.backend {
            // cdylib: serialise the concrete type with bincode directly — byte
            // for byte what the plugin's shim decodes (no `Value` hop).
            Backend::Cdylib(e) => e.call_method(index, input),
            // python: cross via the self-describing `Value` currency.
            #[cfg(feature = "python")]
            Backend::Python(e) => {
                let args = fidius_core::to_value(input)
                    .map_err(|err| CallError::Serialization(err.to_string()))?;
                let out = ValueExecutor::call(e, index, args)?;
                fidius_core::from_value(out)
                    .map_err(|err| CallError::Deserialization(err.to_string()))
            }
            // wasm: same self-describing `Value` currency as python.
            #[cfg(feature = "wasm")]
            Backend::Wasm(e) => {
                let args = fidius_core::to_value(input)
                    .map_err(|err| CallError::Serialization(err.to_string()))?;
                let out = ValueExecutor::call(e, index, args)?;
                fidius_core::from_value(out)
                    .map_err(|err| CallError::Deserialization(err.to_string()))
            }
        }
    }

    /// Start a server-streaming method call by vtable index (FIDIUS-I-0026).
    ///
    /// Returns a [`crate::stream::ChunkStream`] — a `futures::Stream` of
    /// `Result<Value, _>` the caller pulls with `.next().await`. Backpressure and
    /// cancellation are structural: a slow consumer parks the producer, and
    /// dropping the stream tears the producer down. All three backends stream:
    /// Python and WASM cross via the self-describing [`Value`] currency; cdylib
    /// crosses items as concrete bincode of the item type `O` and decodes them
    /// here (FIDIUS-T-0137).
    ///
    /// `O` is the stream's item type. Python/WASM ignore it (they're already
    /// `Value`-native); cdylib uses it to `bincode::<O>`-decode each item.
    #[cfg(feature = "streaming")]
    pub async fn call_streaming<I: Serialize, O: DeserializeOwned + Serialize>(
        &self,
        index: usize,
        input: &I,
    ) -> Result<crate::stream::ChunkStream, CallError> {
        match &self.backend {
            // cdylib: concrete bincode of the args (no `Value` hop), then the
            // iterator-handle streaming path (FIDIUS-I-0026 CS.1). Items also cross
            // as concrete bincode, decoded by `cdylib_stream_decode::<O>`.
            Backend::Cdylib(e) => {
                let input_bytes = fidius_core::wire::serialize(input)
                    .map_err(|err| CallError::Serialization(err.to_string()))?;
                e.call_streaming_raw(index, &input_bytes, cdylib_stream_decode::<O>)
            }
            #[cfg(feature = "python")]
            Backend::Python(e) => {
                let args = fidius_core::to_value(input)
                    .map_err(|err| CallError::Serialization(err.to_string()))?;
                crate::stream::StreamExecutor::call_streaming(e, index, args).await
            }
            #[cfg(feature = "wasm")]
            Backend::Wasm(e) => {
                let args = fidius_core::to_value(input)
                    .map_err(|err| CallError::Serialization(err.to_string()))?;
                crate::stream::StreamExecutor::call_streaming(e, index, args).await
            }
        }
    }

    /// Call a `#[wire(raw)]` method: raw bytes in, raw bytes out, no bincode.
    pub fn call_method_raw(&self, index: usize, input: &[u8]) -> Result<Vec<u8>, CallError> {
        match &self.backend {
            Backend::Cdylib(e) => e.call_method_raw(index, input),
            #[cfg(feature = "python")]
            Backend::Python(e) => PluginExecutor::call_raw(e, index, input),
            #[cfg(feature = "wasm")]
            Backend::Wasm(e) => PluginExecutor::call_raw(e, index, input),
        }
    }

    /// Client-streaming raw call (FIDIUS-I-0030 CS2.2): pass the host's producer
    /// `handle` (built via [`crate::client_stream::host_producer_handle`]) and the
    /// bincode of the non-stream args; returns the bincode of the method's result.
    /// Wired for the cdylib backend; WASM/Python land in CS2.3/CS2.4. The typed
    /// `call_client_streaming` wrapper is CS2.5.
    ///
    /// # Safety
    /// `handle` must be a valid, exclusively-owned producer handle (e.g. from
    /// [`crate::client_stream::host_producer_handle`]); it is consumed by the call.
    #[cfg(feature = "streaming")]
    pub unsafe fn call_client_streaming_raw(
        &self,
        index: usize,
        handle: *mut fidius_core::stream_ffi::FidiusStreamHandle,
        input: &[u8],
    ) -> Result<Vec<u8>, CallError> {
        match &self.backend {
            // SAFETY: forwarded per this fn's contract.
            Backend::Cdylib(e) => unsafe { e.call_client_streaming_raw(index, handle, input) },
            #[cfg(feature = "python")]
            Backend::Python(_) => Err(CallError::Backend {
                runtime: "python".into(),
                message: "client-streaming is not yet wired for Python (FIDIUS-I-0030 CS2.4)"
                    .into(),
            }),
            #[cfg(feature = "wasm")]
            Backend::Wasm(_) => Err(CallError::Backend {
                runtime: "wasm".into(),
                message: "use the typed `call_client_streaming` for the WASM backend".into(),
            }),
        }
    }

    /// Typed client-streaming (FIDIUS-I-0030): the host produces `items` (the
    /// `Stream<T>` argument); the plugin pulls + consumes them and returns `O`.
    /// `args` are the method's non-stream arguments (a tuple). Wired for cdylib
    /// (in-process producer handle) and WASM (the `fidius:stream-pull` import);
    /// Python is CS2.4. The safe wrapper over the per-backend mechanisms.
    #[cfg(feature = "streaming")]
    pub fn call_client_streaming<I, A, O>(
        &self,
        method: usize,
        items: impl IntoIterator<Item = I>,
        args: &A,
    ) -> Result<O, CallError>
    where
        I: Serialize,
        A: Serialize,
        O: DeserializeOwned,
    {
        match &self.backend {
            // cdylib + WASM consume the items as bincode (the guest deserializes each).
            Backend::Cdylib(e) => {
                let encoded = bincode_items(items)?;
                let handle = crate::client_stream::host_producer_handle(encoded.into_iter());
                let arg_bytes = fidius_core::wire::serialize(args)
                    .map_err(|e| CallError::Serialization(e.to_string()))?;
                // SAFETY: `handle` is a freshly-built, exclusively-owned producer.
                let out = unsafe { e.call_client_streaming_raw(method, handle, &arg_bytes) }?;
                fidius_core::wire::deserialize(&out)
                    .map_err(|e| CallError::Deserialization(e.to_string()))
            }
            #[cfg(feature = "wasm")]
            Backend::Wasm(e) => {
                let encoded = bincode_items(items)?;
                let arg_value = fidius_core::to_value(args)
                    .map_err(|err| CallError::Serialization(err.to_string()))?;
                let out = e.call_client_streaming(method, encoded, arg_value)?;
                fidius_core::from_value(out)
                    .map_err(|err| CallError::Deserialization(err.to_string()))
            }
            // Python crosses via the self-describing `Value` currency.
            #[cfg(feature = "python")]
            Backend::Python(e) => {
                let item_values: Vec<fidius_core::Value> = items
                    .into_iter()
                    .map(|i| fidius_core::to_value(&i))
                    .collect::<Result<_, _>>()
                    .map_err(|err| CallError::Serialization(err.to_string()))?;
                let arg_value = fidius_core::to_value(args)
                    .map_err(|err| CallError::Serialization(err.to_string()))?;
                let out = e.call_client_streaming(method, item_values, arg_value)?;
                fidius_core::from_value(out)
                    .map_err(|err| CallError::Deserialization(err.to_string()))
            }
        }
    }

    /// Check if an optional method is supported (capability bit set).
    /// Returns `false` for `bit >= 64` and for backends without capabilities.
    pub fn has_capability(&self, bit: u32) -> bool {
        if bit >= 64 {
            return false;
        }
        self.info().capabilities & (1u64 << bit) != 0
    }

    /// Access the plugin's owned metadata.
    pub fn info(&self) -> &PluginInfo {
        match &self.backend {
            Backend::Cdylib(e) => e.info(),
            #[cfg(feature = "python")]
            Backend::Python(e) => PluginExecutor::info(e),
            #[cfg(feature = "wasm")]
            Backend::Wasm(e) => PluginExecutor::info(e),
        }
    }

    /// Static `#[method_meta(...)]` key/value metadata for the given method,
    /// in declaration order. Empty for out-of-range ids, for interfaces that
    /// declared none, and for backends without descriptor metadata.
    pub fn method_metadata(&self, method_id: u32) -> Vec<(&str, &str)> {
        match &self.backend {
            Backend::Cdylib(e) => e.method_metadata(method_id),
            // Python/WASM plugins carry no descriptor-level method metadata.
            #[cfg(feature = "python")]
            Backend::Python(_) => Vec::new(),
            #[cfg(feature = "wasm")]
            Backend::Wasm(_) => Vec::new(),
        }
    }

    /// Static `#[trait_meta(...)]` key/value metadata declared on the trait.
    /// Empty when none was declared or for backends without descriptor metadata.
    pub fn trait_metadata(&self) -> Vec<(&str, &str)> {
        match &self.backend {
            Backend::Cdylib(e) => e.trait_metadata(),
            #[cfg(feature = "python")]
            Backend::Python(_) => Vec::new(),
            #[cfg(feature = "wasm")]
            Backend::Wasm(_) => Vec::new(),
        }
    }
}

/// Per-item decoder for the cdylib streaming fast path (FIDIUS-T-0137): each item
/// crosses as concrete `bincode(O)` (byte-identical to the unary cdylib wire), so
/// we `wire::deserialize::<O>` then lift to a `Value`. This is the `decode_item`
/// fn pointer the typed caller hands to [`CdylibExecutor::call_streaming_raw`] —
/// `O` is monomorphised in by `call_streaming::<_, O>`.
#[cfg(feature = "streaming")]
fn cdylib_stream_decode<O: DeserializeOwned + Serialize>(
    bytes: &[u8],
) -> Result<fidius_core::Value, CallError> {
    let item: O = fidius_core::wire::deserialize(bytes)
        .map_err(|e| CallError::Deserialization(e.to_string()))?;
    fidius_core::to_value(&item).map_err(|e| CallError::Serialization(e.to_string()))
}

/// Bincode-encode each client-streaming item (the cdylib + WASM currency).
#[cfg(feature = "streaming")]
fn bincode_items<I: Serialize>(
    items: impl IntoIterator<Item = I>,
) -> Result<Vec<Vec<u8>>, CallError> {
    items
        .into_iter()
        .map(|i| fidius_core::wire::serialize(&i))
        .collect::<Result<_, _>>()
        .map_err(|e| CallError::Serialization(e.to_string()))
}
