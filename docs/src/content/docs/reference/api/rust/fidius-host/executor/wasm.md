---
title: "fidius-host::executor::wasm"
description: "`WasmComponentExecutor` — the sandboxed WASM (Component Model) backend."
editUrl: false
sidebar:
  order: 36
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

`WasmComponentExecutor` — the sandboxed WASM (Component Model) backend.

FIDIUS-I-0021 Phase 2, ADR FIDIUS-A-0003 (Path B). The **only** module that
depends on `wasmtime`; it maps the neutral [`fidius_core::Value`](../../../../rust/fidius-guest/value/#enum.Value) to/from
`wasmtime::component::Val` and dispatches by method index into a loaded
component's exported interface.

Sandbox model (human-ratified, FIDIUS-T-0102 finding): real std-built
components import `wasi:cli/io/clocks/filesystem` even when unused, so an
*empty* `Linker` can't instantiate them. We wire `wasmtime-wasi` into the
`Linker` but give the guest a **zero-grant `WasiCtx`** (no FS preopens, no
env, no inherited stdio, no sockets). T-0104 opens specific capabilities
from the package manifest's allow-list.

## Structs

### <span id="struct.EgressDenied"></span><code style="font-size: 0.85em">struct EgressDenied</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

**Derives:** `Debug`, `Clone`

Denial returned by an [`EgressPolicy`](../../../../rust/fidius-host/executor/wasm/#trait.EgressPolicy) to refuse an outbound request.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `reason` | `String` | Human-readable reason (for the embedder's logs; not shown to the guest, which only sees a generic HTTP "request denied"). |

#### Methods

##### <span id="EgressDenied.new"></span><code>new</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn new(reason: impl Into<String>) -> Self
```

A denial with a reason.

<details>
<summary>Source</summary>

```rust
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
```

</details>

### <span id="struct.TcpTarget"></span><code style="font-size: 0.85em">struct TcpTarget&lt;&#39;a&gt;</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `host` | `Option<&'a str>` | The hostname the guest dialed, when it dialed by name and fidius could pin the lookup (lowercased; DNS is case-insensitive). `None` = the guest dialed an IP literal (no lookup happened), or no pin was available for the resolved IP. A name-keyed policy should deny `None` — that is the honest default for an allow-list that speaks names. |
| `addr` | `SocketAddr` | The resolved peer the connect will actually reach (the same value [`EgressPolicy::authorize_tcp`](../../../../rust/fidius-host/executor/wasm/#EgressPolicy.authorize_tcp) has always received). |

### <span id="struct.WasmMethod"></span><code style="font-size: 0.85em">struct WasmMethod</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

**Derives:** `Debug`, `Clone`

A method on the WASM interface, in declaration (vtable) order.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `name` | `String` | Export name within the interface (e.g. `"greet"`). |
| `wire_raw` | `bool` | Whether this method uses `#[wire(raw)]` (bytes in/out). |
| `streaming` | `bool` | Whether this method is server-streaming (`-> fidius::Stream<T>`); the export returns a `next()`-pollable resource the host pumps (WS.3). |

### <span id="struct.WasmComponentExecutor"></span><code style="font-size: 0.85em">struct WasmComponentExecutor</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

WASM component execution backend.

#### Methods

##### <span id="WasmComponentExecutor.from_component_bytes"></span><code>from_​component_​bytes</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn from_component_bytes(
    bytes: &[u8],
    interface: String,
    methods: Vec<WasmMethod>,
    capabilities: Vec<String>,
    info: PluginInfo,
) -> Result<Self, CallError>
```

Build an executor from raw component bytes (a `.wasm` component). For the AOT fast path, prefer `Self::from_cwasm`.

<details>
<summary>Source</summary>

```rust
    pub fn from_component_bytes(
        bytes: &[u8],
        interface: String,
        methods: Vec<WasmMethod>,
        capabilities: Vec<String>,
        info: PluginInfo,
    ) -> Result<Self, CallError> {
        Self::from_component_bytes_with_egress(bytes, interface, methods, capabilities, None, info)
    }
```

</details>

##### <span id="WasmComponentExecutor.from_component_bytes_with_egress"></span><code>from_​component_​bytes_​with_​egress</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn from_component_bytes_with_egress(
    bytes: &[u8],
    interface: String,
    methods: Vec<WasmMethod>,
    capabilities: Vec<String>,
    egress: Option<Arc<dyn EgressPolicy>>,
    info: PluginInfo,
) -> Result<Self, CallError>
```

Like `Self::from_component_bytes` but with an embedder [`EgressPolicy`](../../../../rust/fidius-host/executor/wasm/#trait.EgressPolicy) (FIDIUS-I-0027). `wasi:http` outbound egress is linked only when the package declares the `http` capability **and** `egress` is `Some`.

<details>
<summary>Source</summary>

```rust
    pub fn from_component_bytes_with_egress(
        bytes: &[u8],
        interface: String,
        methods: Vec<WasmMethod>,
        capabilities: Vec<String>,
        egress: Option<Arc<dyn EgressPolicy>>,
        info: PluginInfo,
    ) -> Result<Self, CallError> {
        validate_capabilities(&capabilities)?;
        let engine = Engine::default();
        let component = Component::new(&engine, bytes).map_err(|e| CallError::Backend {
            runtime: "wasm".into(),
            message: e.to_string(),
        })?;
        Self::build(
            engine,
            &component,
            interface,
            methods,
            capabilities,
            egress,
            info,
        )
    }
```

</details>

##### <span id="WasmComponentExecutor.from_cwasm"></span><code>from_​cwasm</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>
 <span class="plissken-badge plissken-badge-unsafe" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-red-low); color: var(--sl-color-red-high);">unsafe</span>

```rust
unsafe fn from_cwasm(
    cwasm: &[u8],
    interface: String,
    methods: Vec<WasmMethod>,
    capabilities: Vec<String>,
    info: PluginInfo,
) -> Result<Self, CallError>
```

Build from a precompiled `.cwasm` (engine/version-specific). ~83 µs load per the spike vs ~6.6 ms JIT.

**Safety**

The bytes must have been produced by `Engine::precompile_component` with a
compatible engine; wasmtime validates the header and refuses a mismatch.

<details>
<summary>Source</summary>

```rust
    pub unsafe fn from_cwasm(
        cwasm: &[u8],
        interface: String,
        methods: Vec<WasmMethod>,
        capabilities: Vec<String>,
        info: PluginInfo,
    ) -> Result<Self, CallError> {
        Self::from_cwasm_with_egress(cwasm, interface, methods, capabilities, None, info)
    }
```

</details>

##### <span id="WasmComponentExecutor.from_cwasm_with_egress"></span><code>from_​cwasm_​with_​egress</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>
 <span class="plissken-badge plissken-badge-unsafe" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-red-low); color: var(--sl-color-red-high);">unsafe</span>

```rust
unsafe fn from_cwasm_with_egress(
    cwasm: &[u8],
    interface: String,
    methods: Vec<WasmMethod>,
    capabilities: Vec<String>,
    egress: Option<Arc<dyn EgressPolicy>>,
    info: PluginInfo,
) -> Result<Self, CallError>
```

Like `Self::from_cwasm` but with an embedder [`EgressPolicy`](../../../../rust/fidius-host/executor/wasm/#trait.EgressPolicy) (FIDIUS-I-0027) — the AOT counterpart of `Self::from_component_bytes_with_egress`.

**Safety**

Same as `Self::from_cwasm`.

<details>
<summary>Source</summary>

```rust
    pub unsafe fn from_cwasm_with_egress(
        cwasm: &[u8],
        interface: String,
        methods: Vec<WasmMethod>,
        capabilities: Vec<String>,
        egress: Option<Arc<dyn EgressPolicy>>,
        info: PluginInfo,
    ) -> Result<Self, CallError> {
        validate_capabilities(&capabilities)?;
        let engine = Engine::default();
        let component = Component::deserialize(&engine, cwasm).map_err(|e| CallError::Backend {
            runtime: "wasm".into(),
            message: e.to_string(),
        })?;
        Self::build(
            engine,
            &component,
            interface,
            methods,
            capabilities,
            egress,
            info,
        )
    }
```

</details>

##### <span id="WasmComponentExecutor.set_resolver"></span><code>set_​resolver</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn set_resolver(
    &mut self,
    resolver: Arc<dyn Fn(&str) -> std::io::Result<Vec<IpAddr>> + Send + Sync>,
)
```

Replace the host-side resolver behind the shadowed `ip-name-lookup` (FIDIUS-I-0034). Test seam — lets the e2e suite model multi-name/ same-IP and rotation without real DNS. Not part of the stable API. Takes effect for stores created after the call (per-call stores and a subsequent `configure`'s persistent store).

<details>
<summary>Source</summary>

```rust
    pub fn set_resolver(
        &mut self,
        resolver: Arc<dyn Fn(&str) -> std::io::Result<Vec<IpAddr>> + Send + Sync>,
    ) {
        self.resolver = resolver;
    }
```

</details>

##### <span id="WasmComponentExecutor.bind_host_table"></span><code>bind_​host_​table</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>
 <span class="plissken-badge plissken-badge-unsafe" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-red-low); color: var(--sl-color-red-high);">unsafe</span>

```rust
unsafe fn bind_host_table(
    &self,
    table: *const fidius_core::host_ffi::HostFunctionTable,
) -> Result<(), crate::error::LoadError>
```

Bind a host-function table (plugin → host callback channel) to this executor. Its identity fields are read here and gated against the guest's expectation on every `fidius:host-call` dispatch. Once-only per interface: a second bind fails rather than swapping the table under in-flight calls.

**Safety**

`table` must be null or a valid, **process-lifetime**
[`HostFunctionTable`](../../../../rust/fidius-guest/host_ffi/#struct.HostFunctionTable) —
e.g. the leaked table a `#[host_interface]`-generated
`<Trait>Binding::table` builds. The executor retains the pointer and
dispatches through it for its remaining lifetime.

<details>
<summary>Source</summary>

```rust
    pub unsafe fn bind_host_table(
        &self,
        table: *const fidius_core::host_ffi::HostFunctionTable,
    ) -> Result<(), crate::error::LoadError> {
        use fidius_core::host_ffi::{bind_status_message, BIND_ERR_ALREADY_BOUND, BIND_ERR_NULL};
        if table.is_null() {
            return Err(crate::error::LoadError::HostBindFailed {
                interface: "<null>".into(),
                code: BIND_ERR_NULL,
                message: bind_status_message(BIND_ERR_NULL).to_string(),
            });
        }
        // SAFETY: non-null; process-lifetime per this method's contract.
        let name = unsafe { std::ffi::CStr::from_ptr((*table).interface_name) }
            .to_str()
            .map_err(|_| crate::error::LoadError::HostImportRegistryInvalid {
                reason: "host table interface_name is not valid UTF-8".into(),
            })?
            .to_string();
        let mut guard = self
            .host_tables
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if guard.contains_key(&name) {
            return Err(crate::error::LoadError::HostBindFailed {
                interface: name,
                code: BIND_ERR_ALREADY_BOUND,
                message: bind_status_message(BIND_ERR_ALREADY_BOUND).to_string(),
            });
        }
        guard.insert(name, HostTableRef(table));
        Ok(())
    }
```

</details>

##### <span id="WasmComponentExecutor.configure"></span><code>configure</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn configure(&mut self, cfg: &[u8]) -> Result<(), CallError>
```

Bind config once (FIDIUS-A-0006 / CI.3): instantiate a *persistent* store, call the guest's `fidius-configure` export with `cfg`, and retain the store so subsequent method calls dispatch on the configured instance. `cfg` is the bincode of the plugin's config type (empty = the zero-config no-op).

<details>
<summary>Source</summary>

```rust
    pub fn configure(&mut self, cfg: &[u8]) -> Result<(), CallError> {
        let (mut store, instance) = self.instantiate()?;
        let func = self.func(&mut store, &instance, "fidius-configure")?;
        let typed = func
            .typed::<(Vec<u8>,), ()>(&store)
            .map_err(|e| CallError::Backend {
                runtime: "wasm".into(),
                message: format!("fidius-configure signature: {e}"),
            })?;
        typed
            .call(&mut store, (cfg.to_vec(),))
            .map_err(|e| CallError::Backend {
                runtime: "wasm".into(),
                message: e.to_string(),
            })?;
        self.configured = Some(std::sync::Mutex::new(ConfiguredStore { store, instance }));
        self.config_bytes = Some(cfg.to_vec());
        Ok(())
    }
```

</details>

##### <span id="WasmComponentExecutor.call_client_streaming"></span><code>call_​client_​streaming</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn call_client_streaming(
    &self,
    method: usize,
    producer: Box<dyn Iterator<Item = Vec<u8>> + Send>,
    args: Value,
) -> Result<Value, CallError>
```

Client-streaming (FIDIUS-I-0030 CS2.3): call a method whose `Stream<T>` argument is fed by the host. `producer` is the bincode-encoded items the guest pulls via the `fidius:stream-pull` import; `args` are the non-stream args (tuple-packed into a `Value`); returns the method's result as a `Value`.

<details>
<summary>Source</summary>

```rust
    pub fn call_client_streaming(
        &self,
        method: usize,
        producer: Box<dyn Iterator<Item = Vec<u8>> + Send>,
        args: Value,
    ) -> Result<Value, CallError> {
        let m = self.method(method, false)?.clone();
        self.with_store(|store, instance| {
            // Lazy: `producer` encodes each item only when the guest's import pulls it
            // (FIDIUS-T-0172), so an unbounded input stays bounded in host memory.
            store.data_mut().client_stream = Some(producer);
            let func = self.func(store, instance, &m.name)?;
            let func_ty = func.ty(&*store);
            let param_types: Vec<wasmtime::component::Type> =
                func_ty.params().map(|(_name, t)| t).collect();
            let params: Vec<Val> = match &args {
                Value::List(items) => items
                    .iter()
                    .zip(param_types.iter())
                    .map(|(v, t)| value_to_val_typed(v, t))
                    .collect::<Result<_, _>>()?,
                Value::Unit => Vec::new(),
                single => {
                    let t = param_types.first().ok_or_else(|| {
                        CallError::Serialization(
                            "client-streaming method takes no non-stream params but an \
                             argument was supplied"
                                .into(),
                        )
                    })?;
                    vec![value_to_val_typed(single, t)?]
                }
            };
            let mut out = [Val::Bool(false)];
            func.call(&mut *store, &params, &mut out)
                .map_err(|e| CallError::Backend {
                    runtime: "wasm".into(),
                    message: e.to_string(),
                })?;
            store.data_mut().client_stream = None;
            if let Val::Result(Err(payload)) = &out[0] {
                return Err(plugin_error_from_val(payload.as_deref()));
            }
            Ok(match &out[0] {
                Val::Result(Ok(inner)) => inner.as_deref().map(val_to_value).unwrap_or(Value::Unit),
                other => val_to_value(other),
            })
        })
    }
```

</details>

##### <span id="WasmComponentExecutor.call_bidi_streaming"></span><code>call_​bidi_​streaming</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>
 <span class="plissken-badge plissken-badge-async" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-blue-low); color: var(--sl-color-blue-high);">async</span>

```rust
async fn call_bidi_streaming(
    &self,
    method: usize,
    producer: Box<dyn Iterator<Item = Vec<u8>> + Send>,
    args: Value,
) -> Result<crate::stream::ChunkStream, CallError>
```

Bidirectional streaming (FIDIUS-I-0032 / ADR-0010): the host produces `producer` (the plugin's `Stream<In>` argument, pulled via the `fidius:stream-pull` import) and consumes the plugin's `Stream<Out>` output resource as a `ChunkStream`. Pulling the output drives the plugin, which pulls input on demand. `args` are the non-stream args (as a `Value`).

<details>
<summary>Source</summary>

```rust
    pub async fn call_bidi_streaming(
        &self,
        method: usize,
        producer: Box<dyn Iterator<Item = Vec<u8>> + Send>,
        args: Value,
    ) -> Result<crate::stream::ChunkStream, CallError> {
        self.stream_with_producer(method, args, Some(producer))
            .await
    }
```

</details>

##### <span id="WasmComponentExecutor.interface_hash"></span><code>interface_​hash</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn interface_hash(&self) -> Result<u64, CallError>
```

Call the `fidius-interface-hash` export — the integrity check the loader (T-0103) runs against the expected interface hash.

<details>
<summary>Source</summary>

```rust
    pub fn interface_hash(&self) -> Result<u64, CallError> {
        let (mut store, instance) = self.instantiate()?;
        let func = self.func(&mut store, &instance, "fidius-interface-hash")?;
        let mut out = [Val::U64(0)];
        func.call(&mut store, &[], &mut out)
            .map_err(|e| CallError::Backend {
                runtime: "wasm".into(),
                message: e.to_string(),
            })?;
        match &out[0] {
            Val::U64(h) => Ok(*h),
            other => Err(CallError::Backend {
                runtime: "wasm".into(),
                message: format!("fidius-interface-hash returned non-u64: {other:?}"),
            }),
        }
    }
```

</details>

## Enums

### <span id="enum.ResponseDirective"></span><code style="font-size: 0.85em">enum ResponseDirective</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

Embedder-supplied policy governing a sandboxed WASM guest's **outbound HTTP** (FIDIUS-I-0027). This is the *only* egress seam fidius ships — it contains **no** allow-list, SSRF, or credential logic; those are deployment-specific policy the embedder implements here.

`wasi:http` is enabled for a guest only when its package declares the `http`
capability **and** a `PluginHost`/executor was given one of these (two-key,
fail-closed). [`authorize`](../../../../rust/fidius-host/executor/wasm/#EgressPolicy.authorize) is then called for
**every** outbound request the guest makes — every request is a host call
across the sandbox boundary, so this is a true per-request checkpoint, not a
one-time gate. Inspect `parts.uri` / `parts.method`, mutate `parts.headers`
to inject credentials, or return `Err(EgressDenied)` to refuse (the guest
then sees an HTTP error and the request is never dispatched).
The target of an outbound TCP connect, as the guest expressed it
(FIDIUS-I-0034). Handed to [`EgressPolicy::authorize_tcp_target`](../../../../rust/fidius-host/executor/wasm/#EgressPolicy.authorize_tcp_target).

Plain public fields on purpose: an embedder constructs these literally in
policy tests. Additional context (e.g. exhaustive name candidates for an
IP) would land as new fields in a breaking rev, not `#[non_exhaustive]`.
What an [`EgressPolicy`](../../../../rust/fidius-host/executor/wasm/#trait.EgressPolicy) wants done with a response it has observed via
[`on_response`](../../../../rust/fidius-host/executor/wasm/#EgressPolicy.on_response) (FIDIUS-I-0035).

#### Variants

- **`Forward`** - Hand the response to the guest unchanged (the default).
- **`RetryOnce`** - Discard this response; re-run [`authorize`](../../../../rust/fidius-host/executor/wasm/#EgressPolicy.authorize) on
  a fresh clone of the **original** (pre-`authorize`) request parts —
  letting the policy inject a fresh credential — and dispatch again. The
  second response is forwarded to the guest unconditionally.

  Bounded by fidius to **at most one retry per original guest request**;
  a `RetryOnce` returned when `retry_available` is `false` (second
  observation, or a non-replayable body) is ignored and the response
  forwards. Each dispatch attempt gets its own connect/first-byte
  timeouts, so a retried request can take up to ~2× the configured
  budget. If the re-run `authorize` denies, the guest sees the same
  generic HTTP "request denied" as any refused request — the policy
  consumed the original response and then refused to re-stamp.

## Traits

### <span id="trait.EgressPolicy"></span><code style="font-size: 0.85em">trait EgressPolicy</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

**Supertraits:** `Send + Sync + 'static`

#### Methods

##### <span id="EgressPolicy.authorize"></span><code>authorize</code>

```rust
fn authorize(
    &self) -> Result<(,
    parts: &mut http::request::Parts) -> Result<(,
) -> Result<(), EgressDenied>
```

Authorize (and optionally decorate) one outbound request before dispatch.

<details>
<summary>Source</summary>

```rust
    fn authorize(&self, parts: &mut http::request::Parts) -> Result<(), EgressDenied>;
```

</details>

##### <span id="EgressPolicy.authorize_tcp"></span><code>authorize_​tcp</code>

```rust
fn authorize_tcp(&self, _addr: &SocketAddr) -> Result<(), EgressDenied>
```

Authorize one outbound **TCP** connection before `connect` (FIDIUS-I-0033). The second key of the same two-key gate as `authorize`: the guest's package must declare the `tcp` capability **and** the host must supply a policy whose `authorize_tcp` returns `Ok` for the target.

This is the seam a database/warehouse connector reaches the wire through —
a pure-Rust sync driver over `std::net::TcpStream` (which on
`wasm32-wasip2` is `wasi:sockets`) becomes reachable only for the
`host:port`s this method allows.

`addr` is the **resolved** peer (`IP:port`). A hostname the guest dialed
(`std::net::TcpStream::connect(("db.internal", 5432))`) is resolved via
`wasi:sockets` name-lookup *first*; fidius pins that lookup and hands the
dialed name to `authorize_tcp_target`
(FIDIUS-I-0034), so a policy that cares about names overrides that method
instead of re-deriving resolve-and-pin itself. Only `TcpConnect` reaches
here; bind/listen and UDP are denied outright.

**Defaults to deny.** An existing HTTP-only [`EgressPolicy`](../../../../rust/fidius-host/executor/wasm/#trait.EgressPolicy) therefore
never silently grants raw TCP; an embedder opts in by overriding this.

<details>
<summary>Source</summary>

```rust
    fn authorize_tcp(&self, _addr: &SocketAddr) -> Result<(), EgressDenied> {
        Err(EgressDenied::new("tcp egress not permitted by this policy"))
    }
```

</details>

##### <span id="EgressPolicy.authorize_tcp_target"></span><code>authorize_​tcp_​target</code>

```rust
fn authorize_tcp_target(
    &self) -> Result<(,
    target: &TcpTarget<'_>) -> Result<(,
) -> Result<(), EgressDenied>
```

Name-aware TCP authorization (FIDIUS-I-0034): like `authorize_tcp`, but the target carries the hostname the guest actually dialed, recovered from fidius's resolve-and-pin of the guest's `wasi:sockets` name lookups. This is the method a hostname allow-list implements — IP allow-lists are operationally broken for managed endpoints that rotate IPs.

`target.host` is `Some(name)` only when the guest dialed by name and
this instance's lookup was pinned; an IP-literal dial arrives as
`None` (deny it if your policy speaks names). The pin narrows
lookup→connect TOCTOU to "an address this instance was actually given
for that name" — it cannot eliminate it.

**Default delegates to `authorize_tcp(&target.addr)`**, so an embedder
overriding only that method observes byte-identical behavior.

<details>
<summary>Source</summary>

```rust
    fn authorize_tcp_target(&self, target: &TcpTarget<'_>) -> Result<(), EgressDenied> {
        self.authorize_tcp(&target.addr)
    }
```

</details>

##### <span id="EgressPolicy.authorize_dns"></span><code>authorize_​dns</code>

```rust
fn authorize_dns(&self, _name: &str) -> Result<(), EgressDenied>
```

Authorize one guest DNS lookup, **before** resolution (FIDIUS-I-0034). Without this hook a guest granted the `tcp`/`udp` tier can probe arbitrary DNS even when every connect would be denied; a denial here fails the guest's lookup (it sees the same resolver failure as a lookup denied outright), resolves nothing, and pins nothing.

**Defaults to allow** — deliberately the opposite polarity of
`authorize_tcp`'s default-deny: name lookup is
already open whenever the tcp/udp tier is granted, so a deny default
would break every existing embedder's hostname dials. The connect
itself is still gated by
`authorize_tcp_target`.

<details>
<summary>Source</summary>

```rust
    fn authorize_dns(&self, _name: &str) -> Result<(), EgressDenied> {
        Ok(())
    }
```

</details>

##### <span id="EgressPolicy.authorize_udp"></span><code>authorize_​udp</code>

```rust
fn authorize_udp(&self, _addr: &SocketAddr) -> Result<(), EgressDenied>
```

Authorize one outbound **UDP** datagram before it leaves (FIDIUS-I-0033) — the symmetric counterpart of `authorize_tcp`. The second key of the same two-key gate: the guest's package must declare the `udp` capability **and** the host must supply a policy whose `authorize_udp` returns `Ok` for the target.

`addr` is the **resolved** remote peer (`IP:port`) of a `connect` or a
one-shot `send_to` — name resolution happens *first*, exactly as for TCP, so
the policy sees the IP the datagram will actually reach (resolve-and-pin
closes DNS-rebinding if the embedder cares). Binding the local source socket
is permitted as setup and never reaches this method; inbound and TCP uses
are denied outright.

**Defaults to deny.** An existing policy never silently gains UDP reach; an
embedder opts in by overriding this.

<details>
<summary>Source</summary>

```rust
    fn authorize_udp(&self, _addr: &SocketAddr) -> Result<(), EgressDenied> {
        Err(EgressDenied::new("udp egress not permitted by this policy"))
    }
```

</details>

##### <span id="EgressPolicy.observes_responses"></span><code>observes_​responses</code>

```rust
fn observes_responses(&self) -> bool
```

Opt-in gate for `on_response` (FIDIUS-I-0035).

**Defaults to `false`**, and while it stays `false` the dispatch path
is byte-identical to a fidius without the response hook: no body tee,
no observation, zero overhead. A policy that overrides `on_response`
must also override this to return `true`, accepting the (small) cost
of teeing every outgoing request body up to 64 KiB for possible replay.

<details>
<summary>Source</summary>

```rust
    fn observes_responses(&self) -> bool {
        false
    }
```

</details>

##### <span id="EgressPolicy.on_response"></span><code>on_​response</code>

```rust
fn on_response(
    &self,
    request: &http::request::Parts,
    status: http::StatusCode,
    headers: &http::HeaderMap,
    retry_available: bool,
) -> ResponseDirective
```

Observe the response HEAD for a request this policy authorized, **before** the guest sees it (FIDIUS-I-0035). Called only when `observes_responses` returns `true`.

This is the auth-retry seam: a policy that injects credentials in
`authorize` can match an expired-credential response
here (typically a 401), invalidate its cache, and return
[`ResponseDirective::RetryOnce`](../../../../rust/fidius-host/executor/wasm/#enum.ResponseDirective) — the request is re-authorized (fresh
credential) and dispatched once more, invisibly to the guest.

- `request` is the **as-dispatched** (post-`authorize`) parts — it
  carries whatever this policy injected. The retry's `authorize` runs
  on a clean pre-`authorize` clone instead, so a stale injected header
  never feeds back into re-stamping.
- Head-only: status + headers, never the response body.
- `retry_available` is `false` when the request body was not replayable
  (streamed past 64 KiB, carried trailers, or never finished before the
  response arrived) and on the observation of a retried dispatch — a
  `RetryOnce` returned then is ignored and the response forwards.
- Transport errors never reach this method; the guest sees them as
  today.

**Defaults to [`ResponseDirective::Forward`](../../../../rust/fidius-host/executor/wasm/#enum.ResponseDirective)** — existing policies are
unaffected.

<details>
<summary>Source</summary>

```rust
    fn on_response(
        &self,
        request: &http::request::Parts,
        status: http::StatusCode,
        headers: &http::HeaderMap,
        retry_available: bool,
    ) -> ResponseDirective {
        let _ = (request, status, headers, retry_available);
        ResponseDirective::Forward
    }
```

</details>

## Functions

### <span id="fn.validate_component"></span><code style="font-size: 0.85em">fn validate_​component</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn validate_component(bytes: &[u8]) -> Result<(), CallError>
```

Validate that `bytes` is a well-formed WASM **component** (Component Model), not a core module or a corrupt artifact. This is the pack-time gate; interface-name + `fidius-interface-hash` conformance is enforced at load (`PluginHost::load_wasm`).

<details>
<summary>Source</summary>

```rust
pub fn validate_component(bytes: &[u8]) -> Result<(), CallError> {
    let engine = Engine::default();
    Component::new(&engine, bytes)
        .map(|_| ())
        .map_err(|e| CallError::Backend {
            runtime: "wasm".into(),
            message: format!("not a valid WASM component: {e}"),
        })
}
```

</details>

### <span id="fn.precompile_component"></span><code style="font-size: 0.85em">fn precompile_​component</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn precompile_component(bytes: &[u8]) -> Result<Vec<u8>, CallError>
```

Ahead-of-time compile a component into engine/version-specific `.cwasm` bytes (`Engine::precompile_component`). Written into the package at pack time and consumed by the AOT load path; a stale `.cwasm` is ignored at load (JIT fallback), so this is purely a load-latency optimization.

<details>
<summary>Source</summary>

```rust
pub fn precompile_component(bytes: &[u8]) -> Result<Vec<u8>, CallError> {
    let engine = Engine::default();
    engine
        .precompile_component(bytes)
        .map_err(|e| CallError::Backend {
            runtime: "wasm".into(),
            message: format!("failed to precompile component: {e}"),
        })
}
```

</details>
