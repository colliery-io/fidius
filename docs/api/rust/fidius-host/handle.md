# fidius-host::handle <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


`PluginHandle` — the unified, caller-facing proxy over a loaded plugin.

A `PluginHandle` is backend-agnostic: callers use the same
`call_method` / `call_method_raw` API whether the plugin is a cdylib, a
Python package, or (Phase 2) a WASM component. The backend lives in the
private [`Backend`] enum.

## Structs

### `fidius-host::handle::PluginHandle`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


A handle to a loaded plugin, ready for calling methods.

Holds the active execution backend. `call_method()` handles serialization,
dispatch, and cleanup; concurrent calls from multiple threads are safe as
long as the underlying plugin is thread-safe (the cdylib macro enforces
`&self`-only methods; the Python backend serialises through the GIL).

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `backend` | `Backend` |  |

#### Methods

##### `from_loaded` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn from_loaded (plugin : crate :: loader :: LoadedPlugin) -> Self
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



##### `from_descriptor` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn from_descriptor (desc : & 'static PluginDescriptor) -> Result < Self , LoadError >
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



##### `find_in_process_descriptor` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn find_in_process_descriptor (plugin_name : & str ,) -> Result < & 'static PluginDescriptor , LoadError >
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



##### `from_python` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn from_python (py : fidius_python :: PythonPluginHandle , info : PluginInfo) -> Self
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



##### `from_wasm` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn from_wasm (executor : WasmComponentExecutor) -> Self
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



##### `call_method` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn call_method < I : Serialize , O : DeserializeOwned > (& self , index : usize , input : & I ,) -> Result < O , CallError >
```

Call a plugin method by vtable index.

Serializes the input with the backend's native wire (cdylib → bincode;
Python/WASM → [`fidius_core::Value`]), dispatches, and decodes the
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



##### `call_streaming` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>
 <span class="plissken-badge plissken-badge-async" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-primary-fg-color); color: white;">async</span>


```rust
async fn call_streaming < I : Serialize , O : DeserializeOwned + Serialize > (& self , index : usize , input : & I ,) -> Result < crate :: stream :: ChunkStream , CallError >
```

Start a server-streaming method call by vtable index (FIDIUS-I-0026).

Returns a [`crate::stream::ChunkStream`] — a `futures::Stream` of
`Result<Value, _>` the caller pulls with `.next().await`. Backpressure and
cancellation are structural: a slow consumer parks the producer, and
dropping the stream tears the producer down. All three backends stream:
Python and WASM cross via the self-describing [`Value`] currency; cdylib
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



##### `call_method_raw` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn call_method_raw (& self , index : usize , input : & [u8]) -> Result < Vec < u8 > , CallError >
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



##### `has_capability` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn has_capability (& self , bit : u32) -> bool
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



##### `info` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn info (& self) -> & PluginInfo
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



##### `method_metadata` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn method_metadata (& self , method_id : u32) -> Vec < (& str , & str) >
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



##### `trait_metadata` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn trait_metadata (& self) -> Vec < (& str , & str) >
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





## Enums

### `fidius-host::handle::Backend` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


The execution backend behind a [`PluginHandle`].

One variant per runtime. The WASM variant lands in Phase 2.

#### Variants

- **`Cdylib`**
- **`Python`** - `.py` package via `fidius-python`'s embedded interpreter. Only present
when the `python` feature is enabled.
- **`Wasm`** - `.wasm` component via wasmtime. Only present when the `wasm` feature is
enabled.



## Functions

### `fidius-host::handle::cdylib_stream_decode`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn cdylib_stream_decode < O : DeserializeOwned + Serialize > (bytes : & [u8] ,) -> Result < fidius_core :: Value , CallError >
```

Per-item decoder for the cdylib streaming fast path (FIDIUS-T-0137): each item crosses as concrete `bincode(O)` (byte-identical to the unary cdylib wire), so we `wire::deserialize::<O>` then lift to a `Value`. This is the `decode_item` fn pointer the typed caller hands to [`CdylibExecutor::call_streaming_raw`] — `O` is monomorphised in by `call_streaming::<_, O>`.

<details>
<summary>Source</summary>

```rust
fn cdylib_stream_decode<O: DeserializeOwned + Serialize>(
    bytes: &[u8],
) -> Result<fidius_core::Value, CallError> {
    let item: O = fidius_core::wire::deserialize(bytes)
        .map_err(|e| CallError::Deserialization(e.to_string()))?;
    fidius_core::to_value(&item).map_err(|e| CallError::Serialization(e.to_string()))
}
```

</details>



