---
title: "fidius-host::handle"
description: "`PluginHandle` — the unified, caller-facing proxy over a loaded plugin."
editUrl: false
sidebar:
  order: 37
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

`PluginHandle` — the unified, caller-facing proxy over a loaded plugin.

A `PluginHandle` is backend-agnostic: callers use the same
`call_method` / `call_method_raw` API whether the plugin is a cdylib, a
Python package, or (Phase 2) a WASM component. The backend lives in the
private `Backend` enum.

**Why an enum backend (FIDIUS-I-0021)**

The backends don't share a typed wire: cdylib decodes concrete-type
**bincode** (not reconstructable from an erased value) while Python/WASM
consume a self-describing [`fidius_core::Value`](../../../rust/fidius-guest/value/#enum.Value). An enum (rather than
`Box<dyn PluginExecutor>`) lets the generic `call_method<I, O>` branch with
the concrete `I`/`O` in scope and serialise with each backend's native
currency — so the **cdylib path stays byte-identical** to before this
refactor (`bincode(input)` straight to the FFI; `Value` is never involved).

## Structs

### <span id="struct.PluginHandle"></span><code style="font-size: 0.85em">struct PluginHandle</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

A handle to a loaded plugin, ready for calling methods.

Holds the active execution backend. `call_method()` handles serialization,
dispatch, and cleanup; concurrent calls from multiple threads are safe as
long as the underlying plugin is thread-safe (the cdylib macro enforces
`&self`-only methods; the Python backend serialises through the GIL).

#### Methods

##### <span id="PluginHandle.from_loaded"></span><code>from_​loaded</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn from_loaded(plugin: crate::loader::LoadedPlugin) -> Self
```

Create a `PluginHandle` from a freshly loaded cdylib plugin.

<details>
<summary>Source</summary>

```rust
    pub fn from_loaded(plugin: crate::loader::LoadedPlugin) -> Self {
        Self {
            backend: Backend::Cdylib(CdylibExecutor::from_loaded(plugin)),
        }
    }
```

</details>

##### <span id="PluginHandle.from_descriptor"></span><code>from_​descriptor</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn from_descriptor(
    desc: &'static PluginDescriptor,
) -> Result<Self, LoadError>
```

Create a `PluginHandle` from a descriptor already registered in the current process's inventory (a `#[plugin_impl]` linked as a normal rlib). No dylib is loaded. Used by `Client::in_process(plugin_name)`.

<details>
<summary>Source</summary>

```rust
    pub fn from_descriptor(desc: &'static PluginDescriptor) -> Result<Self, LoadError> {
        Ok(Self {
            backend: Backend::Cdylib(CdylibExecutor::from_descriptor(desc)?),
        })
    }
```

</details>

##### <span id="PluginHandle.configure_in_process"></span><code>configure_​in_​process</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn configure_in_process<C: Serialize>(
    desc: &'static PluginDescriptor,
    config: &C,
) -> Result<Self, LoadError>
```

Construct a **configured** in-process plugin instance (FIDIUS-A-0006 / CI.2): serialize `config` and bind it once at construction. The plugin's `#[plugin_impl(Trait, config = C)]` `configure` constructor receives it; methods then close over it without re-passing. The config crosses the boundary exactly once, and N differently-configured instances can coexist.

<details>
<summary>Source</summary>

```rust
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
```

</details>

##### <span id="PluginHandle.configure_from_loaded"></span><code>configure_​from_​loaded</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn configure_from_loaded<C: Serialize>(
    plugin: crate::loader::LoadedPlugin,
    config: &C,
) -> Result<Self, LoadError>
```

Construct a **configured** plugin instance from a DYNAMICALLY loaded cdylib — the dynamic-load analogue of `Self::configure_in_process`. A [`LoadedPlugin`](../../../rust/fidius-host/loader/#struct.LoadedPlugin) from [`load_library`](../../../rust/fidius-host/loader/#fn.load_library) / [`PluginHost::load`](../../../rust/fidius-host/host/#PluginHost.load) is constructed with `config` bound once (the plugin's `#[plugin_impl(Trait, config = C)]` `configure` constructor receives it) instead of the singleton `Self::from_loaded` builds. This lets a host load a configured provider cdylib at runtime and bind N differently-configured instances from the same library. The config crosses the boundary exactly once, at construction.

<details>
<summary>Source</summary>

```rust
    pub fn configure_from_loaded<C: Serialize>(
        plugin: crate::loader::LoadedPlugin,
        config: &C,
    ) -> Result<Self, LoadError> {
        let cfg = fidius_core::wire::serialize(config)
            .map_err(|e| LoadError::ConfigSerialization(e.to_string()))?;
        Ok(Self {
            backend: Backend::Cdylib(CdylibExecutor::from_loaded_with_config(plugin, &cfg)),
        })
    }
```

</details>

##### <span id="PluginHandle.find_in_process_descriptor"></span><code>find_​in_​process_​descriptor</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn find_in_process_descriptor(
    plugin_name: &str,
) -> Result<&'static PluginDescriptor, LoadError>
```

Look up a descriptor in the current process's inventory registry by `plugin_name` (the Rust struct name passed to `#[plugin_impl]`).

<details>
<summary>Source</summary>

```rust
    pub fn find_in_process_descriptor(
        plugin_name: &str,
    ) -> Result<&'static PluginDescriptor, LoadError> {
        CdylibExecutor::find_in_process_descriptor(plugin_name)
    }
```

</details>

##### <span id="PluginHandle.from_python"></span><code>from_​python</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn from_python(
    py: fidius_python::PythonPluginHandle,
    info: PluginInfo,
) -> Self
```

Create a `PluginHandle` backed by a loaded Python plugin. `info` is built by the loader from the package manifest + interface descriptor. Only available with the `python` feature.

<details>
<summary>Source</summary>

```rust
    pub fn from_python(py: fidius_python::PythonPluginHandle, info: PluginInfo) -> Self {
        Self {
            backend: Backend::Python(Pyo3Executor::new(py, info)),
        }
    }
```

</details>

##### <span id="PluginHandle.from_wasm"></span><code>from_​wasm</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn from_wasm(executor: WasmComponentExecutor) -> Self
```

Create a `PluginHandle` backed by a loaded WASM component. Only available with the `wasm` feature.

<details>
<summary>Source</summary>

```rust
    pub fn from_wasm(executor: WasmComponentExecutor) -> Self {
        Self {
            backend: Backend::Wasm(executor),
        }
    }
```

</details>

##### <span id="PluginHandle.call_method"></span><code>call_​method</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn call_method<I: Serialize, O: DeserializeOwned>(
    &self,
    index: usize,
    input: &I,
) -> Result<O, CallError>
```

Call a plugin method by vtable index.

Serializes the input with the backend's native wire (cdylib → bincode;
Python/WASM → [`fidius_core::Value`](../../../rust/fidius-guest/value/#enum.Value)), dispatches, and decodes the
result into `O`. No built-in timeout — see the `fidius` crate docs.

<details>
<summary>Source</summary>

```rust
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
```

</details>

##### <span id="PluginHandle.call_streaming"></span><code>call_​streaming</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>
 <span class="plissken-badge plissken-badge-async" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-blue-low); color: var(--sl-color-blue-high);">async</span>

```rust
async fn call_streaming<I: Serialize, O: DeserializeOwned + Serialize>(
    &self,
    index: usize,
    input: &I,
) -> Result<crate::stream::ChunkStream, CallError>
```

Start a server-streaming method call by vtable index (FIDIUS-I-0026).

Returns a [`crate::stream::ChunkStream`](../../../rust/fidius-host/stream/#struct.ChunkStream) — a `futures::Stream` of
`Result<Value, _>` the caller pulls with `.next().await`. Backpressure and
cancellation are structural: a slow consumer parks the producer, and
dropping the stream tears the producer down. All three backends stream:
Python and WASM cross via the self-describing `Value` currency; cdylib
crosses items as concrete bincode of the item type `O` and decodes them
here (FIDIUS-T-0137).

`O` is the stream's item type. Python/WASM ignore it (they're already
`Value`-native); cdylib uses it to `bincode::<O>`-decode each item.

<details>
<summary>Source</summary>

```rust
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
```

</details>

##### <span id="PluginHandle.call_bidi_streaming"></span><code>call_​bidi_​streaming</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>
 <span class="plissken-badge plissken-badge-async" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-blue-low); color: var(--sl-color-blue-high);">async</span>

```rust
async fn call_bidi_streaming<I, A, O>(
    &self,
    index: usize,
    items: impl IntoIterator<Item = I, IntoIter: Send + 'static>,
    args: &A,
) -> Result<crate::stream::ChunkStream, CallError>
where
    I: Serialize + 'static,
    A: Serialize,
    O: DeserializeOwned + Serialize,
```

Start a **bidirectional** streaming call (FIDIUS-I-0032 / ADR-0010): the host produces `items` (the plugin's `Stream<In>` argument) and consumes the plugin's `Stream<Out>` return as the returned [`crate::stream::ChunkStream`](../../../rust/fidius-host/stream/#struct.ChunkStream). Pulling the output drives the plugin, which pulls the input on demand — the synchronous lazy-pull composition. `args` are the non-stream arguments. `O` is the output item type. Wired for cdylib; WASM/Python are BD.3/BD.4.

<details>
<summary>Source</summary>

```rust
    pub async fn call_bidi_streaming<I, A, O>(
        &self,
        index: usize,
        items: impl IntoIterator<Item = I, IntoIter: Send + 'static>,
        args: &A,
    ) -> Result<crate::stream::ChunkStream, CallError>
    where
        I: Serialize + 'static,
        A: Serialize,
        O: DeserializeOwned + Serialize,
    {
        match &self.backend {
            // Lazy producer — items are encoded only as the plugin pulls them (T-0172).
            Backend::Cdylib(e) => {
                let handle = crate::client_stream::host_producer_handle_typed(items.into_iter());
                let arg_bytes = fidius_core::wire::serialize(args)
                    .map_err(|err| CallError::Serialization(err.to_string()))?;
                // SAFETY: `handle` is a freshly-built, exclusively-owned producer.
                unsafe {
                    e.call_bidi_streaming_raw(index, handle, &arg_bytes, cdylib_stream_decode::<O>)
                }
            }
            #[cfg(feature = "python")]
            Backend::Python(e) => {
                // Python crosses via the self-describing `Value` currency, streamed lazily.
                let producer = lazy_json_producer(items);
                let arg_value = fidius_core::to_value(args)
                    .map_err(|err| CallError::Serialization(err.to_string()))?;
                e.call_bidi_streaming(index, producer, arg_value)
            }
            #[cfg(feature = "wasm")]
            Backend::Wasm(e) => {
                let producer = lazy_bincode_producer(items);
                let arg_value = fidius_core::to_value(args)
                    .map_err(|err| CallError::Serialization(err.to_string()))?;
                e.call_bidi_streaming(index, producer, arg_value).await
            }
        }
    }
```

</details>

##### <span id="PluginHandle.call_method_raw"></span><code>call_​method_​raw</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn call_method_raw(
    &self,
    index: usize,
    input: &[u8],
) -> Result<Vec<u8>, CallError>
```

Call a `#[wire(raw)]` method: raw bytes in, raw bytes out, no bincode.

<details>
<summary>Source</summary>

```rust
    pub fn call_method_raw(&self, index: usize, input: &[u8]) -> Result<Vec<u8>, CallError> {
        match &self.backend {
            Backend::Cdylib(e) => e.call_method_raw(index, input),
            #[cfg(feature = "python")]
            Backend::Python(e) => PluginExecutor::call_raw(e, index, input),
            #[cfg(feature = "wasm")]
            Backend::Wasm(e) => PluginExecutor::call_raw(e, index, input),
        }
    }
```

</details>

##### <span id="PluginHandle.call_client_streaming_raw"></span><code>call_​client_​streaming_​raw</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>
 <span class="plissken-badge plissken-badge-unsafe" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-red-low); color: var(--sl-color-red-high);">unsafe</span>

```rust
unsafe fn call_client_streaming_raw(
    &self,
    index: usize,
    handle: *mut fidius_core::stream_ffi::FidiusStreamHandle,
    input: &[u8],
) -> Result<Vec<u8>, CallError>
```

Client-streaming raw call (FIDIUS-I-0030 CS2.2): pass the host's producer `handle` (built via [`crate::client_stream::host_producer_handle`](../../../rust/fidius-host/client_stream/#fn.host_producer_handle)) and the bincode of the non-stream args; returns the bincode of the method's result. Wired for the cdylib backend; WASM/Python land in CS2.3/CS2.4. The typed `call_client_streaming` wrapper is CS2.5.

**Safety**

`handle` must be a valid, exclusively-owned producer handle (e.g. from
[`crate::client_stream::host_producer_handle`](../../../rust/fidius-host/client_stream/#fn.host_producer_handle)); it is consumed by the call.

<details>
<summary>Source</summary>

```rust
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
```

</details>

##### <span id="PluginHandle.call_client_streaming"></span><code>call_​client_​streaming</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn call_client_streaming<I, A, O>(
    &self,
    method: usize,
    items: impl IntoIterator<Item = I, IntoIter: Send + 'static>,
    args: &A,
) -> Result<O, CallError>
where
    I: Serialize + 'static,
    A: Serialize,
    O: DeserializeOwned,
```

Typed client-streaming (FIDIUS-I-0030): the host produces `items` (the `Stream<T>` argument); the plugin pulls + consumes them and returns `O`. `args` are the method's non-stream arguments (a tuple). Wired for cdylib (in-process producer handle) and WASM (the `fidius:stream-pull` import); Python is CS2.4. The safe wrapper over the per-backend mechanisms.

<details>
<summary>Source</summary>

```rust
    pub fn call_client_streaming<I, A, O>(
        &self,
        method: usize,
        items: impl IntoIterator<Item = I, IntoIter: Send + 'static>,
        args: &A,
    ) -> Result<O, CallError>
    where
        I: Serialize + 'static,
        A: Serialize,
        O: DeserializeOwned,
    {
        match &self.backend {
            // cdylib: a lazy producer handle — each item is bincode-encoded only as the
            // plugin pulls it, so an unbounded input stays bounded in memory (T-0172).
            Backend::Cdylib(e) => {
                let handle = crate::client_stream::host_producer_handle_typed(items.into_iter());
                let arg_bytes = fidius_core::wire::serialize(args)
                    .map_err(|e| CallError::Serialization(e.to_string()))?;
                // SAFETY: `handle` is a freshly-built, exclusively-owned producer.
                let out = unsafe { e.call_client_streaming_raw(method, handle, &arg_bytes) }?;
                fidius_core::wire::deserialize(&out)
                    .map_err(|e| CallError::Deserialization(e.to_string()))
            }
            // WASM: same laziness — the boxed producer encodes on pull from the import.
            #[cfg(feature = "wasm")]
            Backend::Wasm(e) => {
                let producer = lazy_bincode_producer(items);
                let arg_value = fidius_core::to_value(args)
                    .map_err(|err| CallError::Serialization(err.to_string()))?;
                let out = e.call_client_streaming(method, producer, arg_value)?;
                fidius_core::from_value(out)
                    .map_err(|err| CallError::Deserialization(err.to_string()))
            }
            // Python crosses via the self-describing `Value` currency, streamed lazily
            // (FIDIUS-T-0174) — each item is converted only as the Python iterator pulls it.
            #[cfg(feature = "python")]
            Backend::Python(e) => {
                let producer = lazy_json_producer(items);
                let arg_value = fidius_core::to_value(args)
                    .map_err(|err| CallError::Serialization(err.to_string()))?;
                let out = e.call_client_streaming(method, producer, arg_value)?;
                fidius_core::from_value(out)
                    .map_err(|err| CallError::Deserialization(err.to_string()))
            }
        }
    }
```

</details>

##### <span id="PluginHandle.bind_wasm_host_table"></span><code>bind_​wasm_​host_​table</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>
 <span class="plissken-badge plissken-badge-unsafe" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-red-low); color: var(--sl-color-red-high);">unsafe</span>

```rust
unsafe fn bind_wasm_host_table(
    &self,
    table: *const fidius_core::host_ffi::HostFunctionTable,
) -> Result<(), LoadError>
```

Bind a host-function table (plugin → host callback channel) to a **WASM-backed** handle. The generated `<Trait>Binding::bind_wasm` resolves through this: `table` must be a process-lifetime table built by a `#[host_interface]`-generated binding (`<Trait>Binding::table`). The guest's `fidius:host-call` import gates every dispatch against the table's version + signature hash, so a mismatched surface fails with a typed error and can never mis-dispatch.

Returns `LoadError::HostBindFailed` with
[`fidius_core::host_ffi::BIND_ERR_WRONG_BACKEND`](../../../rust/fidius-guest/host_ffi/#const.BIND_ERR_WRONG_BACKEND) for non-WASM
backends — cdylib handles bind through `<Trait>Binding::bind` /
`bind_plugin` (the dylib import registry) instead.

**Safety**

`table` must be null or a valid, **process-lifetime**
[`fidius_core::host_ffi::HostFunctionTable`](../../../rust/fidius-guest/host_ffi/#struct.HostFunctionTable) (e.g. the leaked table a
generated `<Trait>Binding::table` builds); the backend retains it and
dispatches through it for its remaining lifetime.

<details>
<summary>Source</summary>

```rust
    pub unsafe fn bind_wasm_host_table(
        &self,
        table: *const fidius_core::host_ffi::HostFunctionTable,
    ) -> Result<(), LoadError> {
        match &self.backend {
            // SAFETY: forwarded per this fn's contract.
            Backend::Wasm(e) => unsafe { e.bind_host_table(table) },
            _ => {
                use fidius_core::host_ffi::{bind_status_message, BIND_ERR_WRONG_BACKEND};
                let interface = if table.is_null() {
                    "<null>".to_string()
                } else {
                    // SAFETY: non-null table per the bind contract.
                    unsafe { std::ffi::CStr::from_ptr((*table).interface_name) }
                        .to_str()
                        .unwrap_or("<invalid>")
                        .to_string()
                };
                Err(LoadError::HostBindFailed {
                    interface,
                    code: BIND_ERR_WRONG_BACKEND,
                    message: bind_status_message(BIND_ERR_WRONG_BACKEND).to_string(),
                })
            }
        }
    }
```

</details>

##### <span id="PluginHandle.has_capability"></span><code>has_​capability</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn has_capability(&self, bit: u32) -> bool
```

Check if an optional method is supported (capability bit set). Returns `false` for `bit >= 64` and for backends without capabilities.

<details>
<summary>Source</summary>

```rust
    pub fn has_capability(&self, bit: u32) -> bool {
        if bit >= 64 {
            return false;
        }
        self.info().capabilities & (1u64 << bit) != 0
    }
```

</details>

##### <span id="PluginHandle.info"></span><code>info</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn info(&self) -> &PluginInfo
```

Access the plugin's owned metadata.

<details>
<summary>Source</summary>

```rust
    pub fn info(&self) -> &PluginInfo {
        match &self.backend {
            Backend::Cdylib(e) => e.info(),
            #[cfg(feature = "python")]
            Backend::Python(e) => PluginExecutor::info(e),
            #[cfg(feature = "wasm")]
            Backend::Wasm(e) => PluginExecutor::info(e),
        }
    }
```

</details>

##### <span id="PluginHandle.method_metadata"></span><code>method_​metadata</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn method_metadata(&self, method_id: u32) -> Vec<(&str, &str)>
```

Static `#[method_meta(...)]` key/value metadata for the given method, in declaration order. Empty for out-of-range ids, for interfaces that declared none, and for backends without descriptor metadata.

<details>
<summary>Source</summary>

```rust
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
```

</details>

##### <span id="PluginHandle.trait_metadata"></span><code>trait_​metadata</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn trait_metadata(&self) -> Vec<(&str, &str)>
```

Static `#[trait_meta(...)]` key/value metadata declared on the trait. Empty when none was declared or for backends without descriptor metadata.

<details>
<summary>Source</summary>

```rust
    pub fn trait_metadata(&self) -> Vec<(&str, &str)> {
        match &self.backend {
            Backend::Cdylib(e) => e.trait_metadata(),
            #[cfg(feature = "python")]
            Backend::Python(_) => Vec::new(),
            #[cfg(feature = "wasm")]
            Backend::Wasm(_) => Vec::new(),
        }
    }
```

</details>
