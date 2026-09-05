# fidius-host::executor::wasm <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


`WasmComponentExecutor` — the sandboxed WASM (Component Model) backend.

FIDIUS-I-0021 Phase 2, ADR FIDIUS-A-0003 (Path B). The **only** module that
depends on `wasmtime`; it maps the neutral [`fidius_core::Value`] to/from
`wasmtime::component::Val` and dispatches by method index into a loaded
component's exported interface.
Sandbox model (human-ratified, FIDIUS-T-0102 finding): real std-built
components import `wasi:cli/io/clocks/filesystem` even when unused, so an
*empty* `Linker` can't instantiate them. We wire `wasmtime-wasi` into the
`Linker` but give the guest a **zero-grant `WasiCtx`** (no FS preopens, no
env, no inherited stdio, no sockets). T-0104 opens specific capabilities
from the package manifest's allow-list.

## Structs

### `fidius-host::executor::wasm::EgressDenied`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`, `Clone`

Denial returned by an [`EgressPolicy`] to refuse an outbound request.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `reason` | `String` | Human-readable reason (for the embedder's logs; not shown to the guest,
which only sees a generic HTTP "request denied"). |

#### Methods

##### `new` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn new (reason : impl Into < String >) -> Self
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





### `fidius-host::executor::wasm::TcpTarget`<'a>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


#### Fields

| Name | Type | Description |
|------|------|-------------|
| `host` | `Option < & 'a str >` | The hostname the guest dialed, when it dialed by name and fidius could
pin the lookup (lowercased; DNS is case-insensitive). `None` = the
guest dialed an IP literal (no lookup happened), or no pin was
available for the resolved IP. A name-keyed policy should deny `None`
— that is the honest default for an allow-list that speaks names. |
| `addr` | `SocketAddr` | The resolved peer the connect will actually reach (the same value
[`EgressPolicy::authorize_tcp`] has always received). |



### `fidius-host::executor::wasm::EgressHooks`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


fidius's [`WasiHttpHooks`] adapter: routes every outbound request through the embedder's [`EgressPolicy`] before handing off to wasi-http's `default_send_request`. `policy: None` denies everything (defensive — the http imports are never linked without a policy, so this is unreachable in practice).

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `policy` | `Option < Arc < dyn EgressPolicy > >` |  |



### `fidius-host::executor::wasm::HostState`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


Per-store host state. The `WasiCtx` is built from the capability allow-list (deny-all baseline) by `build_wasi_ctx`. `http_ctx`/`hooks` back the optional `wasi:http` egress (FIDIUS-I-0027); they're inert unless egress was enabled.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `ctx` | `WasiCtx` |  |
| `table` | `ResourceTable` |  |
| `http_ctx` | `WasiHttpCtx` |  |
| `hooks` | `EgressHooks` |  |
| `client_stream` | `Option < Box < dyn Iterator < Item = Vec < u8 > > + Send > >` | Client-streaming producer (FIDIUS-I-0030 CS2.3): the host sets this before
a client-streaming call; the guest's `fidius:stream-pull/pull.next` import
pulls bincode items from it. `None` outside such a call. |
| `host_tables` | `HostTables` | Host-function tables bound to this executor (plugin → host callback
channel, wasm variant). Shared with the executor so tables bound after
instantiation (including after `configure`'s persistent store was
created) are visible to the `fidius:host-call` import. |
| `pins` | `PinTable` | Resolve-and-pin table (FIDIUS-I-0034): what this store's name lookups
resolved to, written by the shadowed `ip-name-lookup` and read by the
same store's `socket_addr_check` (which holds a clone of the `Arc`). |
| `resolver` | `Resolver` | Host-side resolution function for the shadowed lookup. The executor's
default matches upstream (std `ToSocketAddrs`); tests may inject one
via `WasmComponentExecutor::set_resolver`. |



### `fidius-host::executor::wasm::HostTableRef`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


**Derives:** `Clone`, `Copy`

A `Send + Sync` wrapper for a bound, process-lifetime [`HostFunctionTable`] pointer (same justification as the loader's static table pointers: the generated binding leaks the table it builds).

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `0` | `* const fidius_core :: host_ffi :: HostFunctionTable` |  |



### `fidius-host::executor::wasm::WasmMethod`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`, `Clone`

A method on the WASM interface, in declaration (vtable) order.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `name` | `String` | Export name within the interface (e.g. `"greet"`). |
| `wire_raw` | `bool` | Whether this method uses `#[wire(raw)]` (bytes in/out). |
| `streaming` | `bool` | Whether this method is server-streaming (`-> fidius::Stream<T>`); the
export returns a `next()`-pollable resource the host pumps (WS.3). |



### `fidius-host::executor::wasm::WasmComponentExecutor`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


WASM component execution backend.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `engine` | `Engine` |  |
| `instance_pre` | `InstancePre < HostState >` | Pre-linked component (Linker + WASI wired in, typechecked) built once at
load. Per call we only create a fresh `Store` and `instance_pre.instantiate`
— instantiation stays per-call (isolation) but the expensive linking is
done once, not on every call (FIDIUS-I-0024). |
| `interface` | `String` | Fully-qualified exported interface name, e.g.
`"fidius:greeter/greeter@1.0.0"`. |
| `methods` | `Vec < WasmMethod >` | Methods in interface order; index = the vtable index callers use. |
| `capabilities` | `Vec < String >` | WASI capability allow-list from `[wasm].capabilities`. Empty = deny-all.
Filesystem is never granted regardless. |
| `egress` | `Option < Arc < dyn EgressPolicy > >` | Embedder egress policy (FIDIUS-I-0027). `Some` + the `http` capability is
the two-key that links `wasi:http`; otherwise egress is impossible. |
| `info` | `PluginInfo` |  |
| `configured` | `Option < std :: sync :: Mutex < ConfiguredStore > >` | FIDIUS-A-0006 / CI.3: when configured, the instance lives in a *persistent*
store (config bound once via the `fidius-configure` export); method calls
dispatch on it instead of a fresh per-call store. `None` = zero-config
(per-call instantiation, the isolation default). |
| `config_bytes` | `Option < Vec < u8 > >` | The config bytes (FIDIUS-A-0006 / CI.3), retained so a *streaming* call can
`fidius-configure` the store it owns for the stream's lifetime (a stream
takes its store by value, so it can't share the unary persistent store — it
just needs the same config set in its own memory first). |
| `host_tables` | `HostTables` | Host-function tables bound to this executor (plugin → host callback
channel, wasm variant), keyed by interface name. Populated by
[`Self::bind_host_table`]; read by the `fidius:host-call` import. |
| `resolver` | `Resolver` | Host-side resolver behind the shadowed `ip-name-lookup`
(FIDIUS-I-0034). Defaults to std `ToSocketAddrs` (upstream parity);
[`Self::set_resolver`] injects one for deterministic tests. |

#### Methods

##### `from_component_bytes` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn from_component_bytes (bytes : & [u8] , interface : String , methods : Vec < WasmMethod > , capabilities : Vec < String > , info : PluginInfo ,) -> Result < Self , CallError >
```

Build an executor from raw component bytes (a `.wasm` component). For the AOT fast path, prefer [`Self::from_cwasm`].

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



##### `from_component_bytes_with_egress` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn from_component_bytes_with_egress (bytes : & [u8] , interface : String , methods : Vec < WasmMethod > , capabilities : Vec < String > , egress : Option < Arc < dyn EgressPolicy > > , info : PluginInfo ,) -> Result < Self , CallError >
```

Like [`Self::from_component_bytes`] but with an embedder [`EgressPolicy`] (FIDIUS-I-0027). `wasi:http` outbound egress is linked only when the package declares the `http` capability **and** `egress` is `Some`.

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



##### `from_cwasm` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>
 <span class="plissken-badge plissken-badge-unsafe" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #f44336; color: white;">unsafe</span>


```rust
unsafe fn from_cwasm (cwasm : & [u8] , interface : String , methods : Vec < WasmMethod > , capabilities : Vec < String > , info : PluginInfo ,) -> Result < Self , CallError >
```

Build from a precompiled `.cwasm` (engine/version-specific). ~83 µs load per the spike vs ~6.6 ms JIT.

# Safety
The bytes must have been produced by `Engine::precompile_component` with a compatible engine; wasmtime validates the header and refuses a mismatch.

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



##### `from_cwasm_with_egress` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>
 <span class="plissken-badge plissken-badge-unsafe" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #f44336; color: white;">unsafe</span>


```rust
unsafe fn from_cwasm_with_egress (cwasm : & [u8] , interface : String , methods : Vec < WasmMethod > , capabilities : Vec < String > , egress : Option < Arc < dyn EgressPolicy > > , info : PluginInfo ,) -> Result < Self , CallError >
```

Like [`Self::from_cwasm`] but with an embedder [`EgressPolicy`] (FIDIUS-I-0027) — the AOT counterpart of [`Self::from_component_bytes_with_egress`].

# Safety
Same as [`Self::from_cwasm`].

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



##### `build` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn build (engine : Engine , component : & Component , interface : String , methods : Vec < WasmMethod > , capabilities : Vec < String > , egress : Option < Arc < dyn EgressPolicy > > , info : PluginInfo ,) -> Result < Self , CallError >
```

Shared constructor: wire WASI into a `Linker` and pre-instantiate the component **once**. The resulting `InstancePre` is reused for every call.

<details>
<summary>Source</summary>

```rust
    fn build(
        engine: Engine,
        component: &Component,
        interface: String,
        methods: Vec<WasmMethod>,
        capabilities: Vec<String>,
        egress: Option<Arc<dyn EgressPolicy>>,
        info: PluginInfo,
    ) -> Result<Self, CallError> {
        // Fail loud on a wasi:http version the host can't satisfy (FIDIUS-A-0005),
        // ahead of the cryptic wasmtime instantiate error.
        let import_names: Vec<String> = component
            .component_type()
            .imports(&engine)
            .map(|(name, _)| name.to_string())
            .collect();
        if let Some(message) = wasi_http_incompatibility(import_names.iter().map(String::as_str)) {
            return Err(CallError::Backend {
                runtime: "wasm".into(),
                message,
            });
        }

        let mut linker: Linker<HostState> = Linker::new(&engine);
        // WASI present, zero grants (the deny-all/allow-list `WasiCtx` is built
        // fresh per call in `instantiate`).
        add_to_linker_sync(&mut linker).map_err(|e| CallError::Backend {
            runtime: "wasm".into(),
            message: e.to_string(),
        })?;
        // FIDIUS-I-0027 two-key gating: link `wasi:http` ONLY when the package
        // declares the `http` capability AND the embedder supplied an
        // `EgressPolicy`. Missing either → the http imports are absent, so a guest
        // that imports `wasi:http/outgoing-handler` fails closed at instantiate.
        let http_enabled = capabilities.iter().any(|c| c == "http") && egress.is_some();
        if http_enabled {
            add_only_http_to_linker_sync(&mut linker).map_err(|e| CallError::Backend {
                runtime: "wasm".into(),
                message: e.to_string(),
            })?;
        }
        // FIDIUS-I-0034 resolve-and-pin: shadow `wasi:sockets/ip-name-lookup`
        // with fidius's pinning implementation, under exactly the two-key
        // condition that turns name lookup on at all (a `tcp`/`udp` grant AND
        // an embedder policy — the same gate as `allow_ip_name_lookup(true)`
        // in `build_wasi_ctx`). Without it, upstream's implementation stands
        // and its default-off lookup flag keeps resolution dead, so
        // non-granted guests are byte-identical to pre-0034 behavior.
        let sockets_enabled = capabilities.iter().any(|c| c == "tcp" || c == "udp");
        if sockets_enabled && egress.is_some() {
            linker.allow_shadowing(true);
            ip_name_lookup::add_to_linker::<HostState, FidiusNameLookup>(
                &mut linker,
                name_lookup_view,
            )
            .map_err(|e| CallError::Backend {
                runtime: "wasm".into(),
                message: e.to_string(),
            })?;
            linker.allow_shadowing(false);
        }
        // Client-streaming (FIDIUS-I-0030 CS2.3): provide the `fidius:stream-pull`
        // import the guest pulls its `Stream<T>` argument through. Always linked
        // (harmless for components that don't import it); backed per call by
        // `HostState::client_stream`.
        linker
            .instance("fidius:stream-pull/pull@0.1.0")
            .and_then(|mut pull| {
                pull.func_wrap(
                    "next",
                    |mut store: wasmtime::StoreContextMut<'_, HostState>,
                     (): ()|
                     -> wasmtime::Result<(Option<Vec<u8>>,)> {
                        let item = store
                            .data_mut()
                            .client_stream
                            .as_mut()
                            .and_then(|p| p.next());
                        Ok((item,))
                    },
                )?;
                Ok(())
            })
            .map_err(|e| CallError::Backend {
                runtime: "wasm".into(),
                message: e.to_string(),
            })?;

        // Host functions (plugin → host callback channel, wasm variant):
        // provide the `fidius:host-call` import the guest dispatches host
        // functions through. Always linked (harmless for components that
        // don't import it); backed by the executor's bound-table registry.
        // The identity triple the guest sends with each call is gated here
        // against the bound table before any dispatch — the wasm counterpart
        // of the dylib bind-time gate (never a bincode mis-dispatch).
        let host_tables: HostTables = Arc::new(std::sync::RwLock::new(Default::default()));
        linker
            .instance("fidius:host-call/host@0.1.0")
            .and_then(|mut host| {
                host.func_wrap(
                    "call",
                    |store: wasmtime::StoreContextMut<'_, HostState>,
                     (interface, expected_version, expected_hash, index, args): (
                        String,
                        u32,
                        u64,
                        u32,
                        Vec<u8>,
                    )|
                     -> wasmtime::Result<((i32, Vec<u8>),)> {
                        use fidius_core::host_ffi::{
                            HOST_CALL_PROBE_INDEX, HOST_CALL_STATUS_HASH_MISMATCH,
                            HOST_CALL_STATUS_NOT_BOUND, HOST_CALL_STATUS_VERSION_MISMATCH,
                        };
                        let tables = store.data().host_tables.clone();
                        let guard = tables
                            .read()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        let Some(entry) = guard.get(&interface) else {
                            return Ok(((HOST_CALL_STATUS_NOT_BOUND, Vec::new()),));
                        };
                        // SAFETY: process-lifetime table per the bind contract.
                        let table = unsafe { &*entry.0 };
                        if table.interface_version != expected_version {
                            let payload = fidius_core::wire::serialize(&(
                                expected_version,
                                table.interface_version,
                            ))
                            .unwrap_or_default();
                            return Ok(((HOST_CALL_STATUS_VERSION_MISMATCH, payload),));
                        }
                        if table.interface_hash != expected_hash {
                            let payload = fidius_core::wire::serialize(&(
                                expected_hash,
                                table.interface_hash,
                            ))
                            .unwrap_or_default();
                            return Ok(((HOST_CALL_STATUS_HASH_MISMATCH, payload),));
                        }
                        if index == HOST_CALL_PROBE_INDEX {
                            // Bind-probe: the gate passed; nothing to dispatch.
                            return Ok(((fidius_core::status::STATUS_OK, Vec::new()),));
                        }
                        Ok((dispatch_host_table(table, index, &args),))
                    },
                )?;
                Ok(())
            })
            .map_err(|e| CallError::Backend {
                runtime: "wasm".into(),
                message: e.to_string(),
            })?;

        let instance_pre = linker
            .instantiate_pre(component)
            .map_err(|e| CallError::Backend {
                runtime: "wasm".into(),
                message: e.to_string(),
            })?;
        Ok(Self {
            engine,
            instance_pre,
            interface,
            methods,
            capabilities,
            egress,
            info,
            configured: None,
            config_bytes: None,
            host_tables,
            resolver: default_resolver(),
        })
    }
```

</details>



##### `set_resolver` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn set_resolver (& mut self , resolver : Arc < dyn Fn (& str) -> std :: io :: Result < Vec < IpAddr > > + Send + Sync > ,)
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



##### `bind_host_table` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>
 <span class="plissken-badge plissken-badge-unsafe" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #f44336; color: white;">unsafe</span>


```rust
unsafe fn bind_host_table (& self , table : * const fidius_core :: host_ffi :: HostFunctionTable ,) -> Result < () , crate :: error :: LoadError >
```

Bind a host-function table (plugin → host callback channel) to this executor. Its identity fields are read here and gated against the guest's expectation on every `fidius:host-call` dispatch. Once-only per interface: a second bind fails rather than swapping the table under in-flight calls.

# Safety
`table` must be null or a valid, **process-lifetime** [`HostFunctionTable`](fidius_core::host_ffi::HostFunctionTable) — e.g. the leaked table a `#[host_interface]`-generated `<Trait>Binding::table` builds. The executor retains the pointer and dispatches through it for its remaining lifetime.

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



##### `configure` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn configure (& mut self , cfg : & [u8]) -> Result < () , CallError >
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



##### `call_client_streaming` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn call_client_streaming (& self , method : usize , producer : Box < dyn Iterator < Item = Vec < u8 > > + Send > , args : Value ,) -> Result < Value , CallError >
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



##### `call_bidi_streaming` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>
 <span class="plissken-badge plissken-badge-async" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-primary-fg-color); color: white;">async</span>


```rust
async fn call_bidi_streaming (& self , method : usize , producer : Box < dyn Iterator < Item = Vec < u8 > > + Send > , args : Value ,) -> Result < crate :: stream :: ChunkStream , CallError >
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



##### `with_store` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn with_store < R > (& self , f : impl FnOnce (& mut Store < HostState > , & wasmtime :: component :: Instance) -> Result < R , CallError > ,) -> Result < R , CallError >
```

Run `f` with a `(store, instance)`: the persistent configured store if configured (FIDIUS-A-0006 / CI.3), else a fresh per-call one (isolation).

<details>
<summary>Source</summary>

```rust
    fn with_store<R>(
        &self,
        f: impl FnOnce(&mut Store<HostState>, &wasmtime::component::Instance) -> Result<R, CallError>,
    ) -> Result<R, CallError> {
        if let Some(cfg) = &self.configured {
            let mut guard = cfg.lock().map_err(|_| CallError::Backend {
                runtime: "wasm".into(),
                message: "configured store mutex poisoned".into(),
            })?;
            let ConfiguredStore { store, instance } = &mut *guard;
            f(store, instance)
        } else {
            let (mut store, instance) = self.instantiate()?;
            f(&mut store, &instance)
        }
    }
```

</details>



##### `instantiate` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn instantiate (& self) -> Result < (Store < HostState > , wasmtime :: component :: Instance) , CallError >
```

Instantiate a fresh sandboxed `Store` + component instance from the cached `InstancePre`. Per-call instantiation gives isolation; the linking cost is already paid in `build` (FIDIUS-I-0024).

<details>
<summary>Source</summary>

```rust
    fn instantiate(&self) -> Result<(Store<HostState>, wasmtime::component::Instance), CallError> {
        // One pin table per store (FIDIUS-I-0034): the shadowed lookup writes
        // through `HostState.pins`, the `socket_addr_check` closure inside the
        // `WasiCtx` reads through its own clone. Lifetime = this store —
        // per-call for unary dispatch, the persistent store's lifetime for
        // configured instances (replace-on-re-resolve is the eviction policy).
        let pins = PinTable::default();
        let host = HostState {
            ctx: build_wasi_ctx(&self.capabilities, self.egress.clone(), pins.clone()),
            table: ResourceTable::new(),
            http_ctx: WasiHttpCtx::new(),
            hooks: EgressHooks {
                policy: self.egress.clone(),
            },
            client_stream: None,
            host_tables: self.host_tables.clone(),
            pins,
            resolver: self.resolver.clone(),
        };
        let mut store = Store::new(&self.engine, host);
        let instance =
            self.instance_pre
                .instantiate(&mut store)
                .map_err(|e| CallError::Backend {
                    runtime: "wasm".into(),
                    message: e.to_string(),
                })?;
        Ok((store, instance))
    }
```

</details>



##### `func` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn func (& self , store : & mut Store < HostState > , instance : & wasmtime :: component :: Instance , name : & str ,) -> Result < wasmtime :: component :: Func , CallError >
```

Resolve an exported function within the plugin's interface by name.

<details>
<summary>Source</summary>

```rust
    fn func(
        &self,
        store: &mut Store<HostState>,
        instance: &wasmtime::component::Instance,
        name: &str,
    ) -> Result<wasmtime::component::Func, CallError> {
        // wasmtime 45: `get_export` returns `(ComponentItem, ComponentExportIndex)`;
        // the index impls `InstanceExportLookup` for `get_func` and is the parent
        // for nested lookups.
        let (_, iface_idx) = instance
            .get_export(&mut *store, None, &self.interface)
            .ok_or_else(|| CallError::Backend {
                runtime: "wasm".into(),
                message: format!("component does not export interface '{}'", self.interface),
            })?;
        let (_, func_idx) = instance
            .get_export(&mut *store, Some(&iface_idx), name)
            .ok_or_else(|| CallError::Backend {
                runtime: "wasm".into(),
                message: format!("interface '{}' does not export '{name}'", self.interface),
            })?;
        instance
            .get_func(&mut *store, func_idx)
            .ok_or_else(|| CallError::Backend {
                runtime: "wasm".into(),
                message: format!("export '{name}' is not a function"),
            })
    }
```

</details>



##### `method` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn method (& self , index : usize , want_raw : bool) -> Result < & WasmMethod , CallError >
```

<details>
<summary>Source</summary>

```rust
    fn method(&self, index: usize, want_raw: bool) -> Result<&WasmMethod, CallError> {
        let m = self
            .methods
            .get(index)
            .ok_or(CallError::InvalidMethodIndex {
                index,
                count: self.methods.len() as u32,
            })?;
        if m.wire_raw != want_raw {
            return Err(CallError::WireModeMismatch {
                method: m.name.clone(),
                declared: m.wire_raw,
                attempted: want_raw,
            });
        }
        Ok(m)
    }
```

</details>



##### `interface_hash` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn interface_hash (& self) -> Result < u64 , CallError >
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



##### `stream_with_producer` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>
 <span class="plissken-badge plissken-badge-async" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-primary-fg-color); color: white;">async</span>


```rust
async fn stream_with_producer (& self , method : usize , args : Value , producer : Option < Box < dyn Iterator < Item = Vec < u8 > > + Send > > ,) -> Result < crate :: stream :: ChunkStream , CallError >
```

Shared server-streaming / bidirectional output pump. `producer = Some(items)` sets the client-streaming **input** producer in the (pump-owned) store before the export call, so the output resource's `next()` re-enters the `fidius:stream-pull` import on demand — the bidirectional synchronous lazy-pull composition (FIDIUS-I-0032 / ADR-0010). `None` = plain server-streaming (WS).

<details>
<summary>Source</summary>

```rust
    async fn stream_with_producer(
        &self,
        method: usize,
        args: Value,
        producer: Option<Box<dyn Iterator<Item = Vec<u8>> + Send>>,
    ) -> Result<crate::stream::ChunkStream, CallError> {
        let m = self.method(method, false)?.clone();
        if !m.streaming {
            return Err(CallError::Backend {
                runtime: "wasm".into(),
                message: format!("method '{}' is not a server-streaming method", m.name),
            });
        }
        let (mut store, instance) = self.instantiate()?;
        // Bidirectional: seed the input producer the output resource pulls through (lazy,
        // FIDIUS-T-0172). The store moves to the pump thread, so it lives for the stream.
        if let Some(producer) = producer {
            store.data_mut().client_stream = Some(producer);
        }
        // FIDIUS-A-0006 / CI.3: a stream takes its store by value (the pump owns it
        // for the stream's lifetime), so it can't share the unary persistent store —
        // it just needs the same config set in its own memory first. Bind config
        // into this store (once, at stream start) before the streaming export reads it.
        if let Some(cfg) = &self.config_bytes {
            let cfunc = self.func(&mut store, &instance, "fidius-configure")?;
            let typed = cfunc
                .typed::<(Vec<u8>,), ()>(&store)
                .map_err(|e| CallError::Backend {
                    runtime: "wasm".into(),
                    message: format!("fidius-configure signature: {e}"),
                })?;
            typed
                .call(&mut store, (cfg.clone(),))
                .map_err(|e| CallError::Backend {
                    runtime: "wasm".into(),
                    message: e.to_string(),
                })?;
        }
        let params: Vec<Val> = match args {
            Value::List(items) => items.iter().map(value_to_val).collect::<Result<_, _>>()?,
            Value::Unit => Vec::new(),
            single => vec![value_to_val(&single)?],
        };

        // Call the streaming export: it returns an owned stream `resource`.
        let start = self.func(&mut store, &instance, &m.name)?;
        let mut out = [Val::Bool(false)];
        start
            .call(&mut store, &params, &mut out)
            .map_err(|e| CallError::Backend {
                runtime: "wasm".into(),
                message: e.to_string(),
            })?;
        // (wasmtime 45: `post_return` is a no-op and deprecated — not called.)
        let resource = match out.into_iter().next() {
            Some(Val::Resource(r)) => r,
            other => {
                return Err(CallError::Backend {
                    runtime: "wasm".into(),
                    message: format!(
                        "streaming method '{}' did not return a resource: {other:?}",
                        m.name
                    ),
                })
            }
        };

        // The poll method on the returned resource: `[method]<m>-stream.next`
        // (WS.1/WS.2 naming convention: the resource for method `m` is `m-stream`).
        let next_name = format!("[method]{}-stream.next", m.name);
        let next_func = self.func(&mut store, &instance, &next_name)?;

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<Value, CallError>>(STREAM_CHANNEL_CAP);

        // Dedicated pump thread owns the Store + resource (mirrors the Python GIL
        // thread). Sync wasmtime `next()` calls, bounded channel = backpressure.
        std::thread::spawn(move || {
            loop {
                let mut nout = [Val::Bool(false)];
                if let Err(e) = next_func.call(&mut store, &[Val::Resource(resource)], &mut nout) {
                    let _ = tx.blocking_send(Err(CallError::Backend {
                        runtime: "wasm".into(),
                        message: e.to_string(),
                    }));
                    break;
                }
                // (wasmtime 45: `post_return` is a deprecated no-op — not called.)

                // nout[0] = result<option<u64>, plugin-error>
                let step: Option<Result<Value, CallError>> = match &nout[0] {
                    Val::Result(Ok(inner)) => match inner.as_deref() {
                        Some(Val::Option(Some(v))) => Some(Ok(val_to_value(v))),
                        // none → clean end of stream
                        Some(Val::Option(None)) | None => None,
                        Some(other) => Some(Ok(val_to_value(other))),
                    },
                    Val::Result(Err(payload)) => {
                        Some(Err(plugin_error_from_val(payload.as_deref())))
                    }
                    other => Some(Ok(val_to_value(other))),
                };

                match step {
                    None => break,
                    Some(item) => {
                        let is_err = item.is_err();
                        if tx.blocking_send(item).is_err() {
                            // Consumer dropped the stream → cancel.
                            break;
                        }
                        if is_err {
                            break;
                        }
                    }
                }
            }
            // Drop the resource (runs the guest destructor = D3 cancel), then the Store.
            let _ = resource.resource_drop(&mut store);
            drop(store);
        });

        let body = futures::stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|item| (item, rx))
        });
        Ok(crate::stream::ChunkStream::new(body))
    }
```

</details>





### `fidius-host::executor::wasm::ConfiguredStore`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


A configured instance's persistent store + instance (FIDIUS-A-0006 / CI.3).

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `store` | `Store < HostState >` |  |
| `instance` | `wasmtime :: component :: Instance` |  |



## Enums

### `fidius-host::executor::wasm::ResponseDirective` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Embedder-supplied policy governing a sandboxed WASM guest's **outbound HTTP** (FIDIUS-I-0027). This is the *only* egress seam fidius ships — it contains **no** allow-list, SSRF, or credential logic; those are deployment-specific policy the embedder implements here.

`wasi:http` is enabled for a guest only when its package declares the `http`
capability **and** a `PluginHost`/executor was given one of these (two-key,
fail-closed). [`authorize`](EgressPolicy::authorize) is then called for
**every** outbound request the guest makes — every request is a host call
across the sandbox boundary, so this is a true per-request checkpoint, not a
one-time gate. Inspect `parts.uri` / `parts.method`, mutate `parts.headers`
to inject credentials, or return `Err(EgressDenied)` to refuse (the guest
then sees an HTTP error and the request is never dispatched).
The target of an outbound TCP connect, as the guest expressed it
(FIDIUS-I-0034). Handed to [`EgressPolicy::authorize_tcp_target`].
Plain public fields on purpose: an embedder constructs these literally in
policy tests. Additional context (e.g. exhaustive name candidates for an
IP) would land as new fields in a breaking rev, not `#[non_exhaustive]`.
What an [`EgressPolicy`] wants done with a response it has observed via
[`on_response`](EgressPolicy::on_response) (FIDIUS-I-0035).

#### Variants

- **`Forward`** - Hand the response to the guest unchanged (the default).
- **`RetryOnce`** - Discard this response; re-run [`authorize`](EgressPolicy::authorize) on
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



## Functions

### `fidius-host::executor::wasm::copy_config`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn copy_config (c : & OutgoingRequestConfig) -> OutgoingRequestConfig
```

Copy an [`OutgoingRequestConfig`] (all-`Copy` fields; the type itself isn't `Clone` upstream). The retry attempt reuses the same budget, so connect and first-byte timeouts apply **per attempt**.

<details>
<summary>Source</summary>

```rust
fn copy_config(c: &OutgoingRequestConfig) -> OutgoingRequestConfig {
    OutgoingRequestConfig {
        use_tls: c.use_tls,
        connect_timeout: c.connect_timeout,
        first_byte_timeout: c.first_byte_timeout,
        between_bytes_timeout: c.between_bytes_timeout,
    }
}
```

</details>



### `fidius-host::executor::wasm::dispatch_observed`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
async fn dispatch_observed (policy : Arc < dyn EgressPolicy > , original : http :: request :: Parts , dispatched : http :: request :: Parts , body : HyperOutgoingBody , capture : CaptureHandle , config : OutgoingRequestConfig ,) -> Result < IncomingResponse , ErrorCode >
```

The observing dispatch (FIDIUS-I-0035): send the request, show the policy the response head, and honor at most one `RetryOnce` — structurally, a straight-line function with a single possible second dispatch; no loop exists for a policy to drive.

`original` is the pre-`authorize` parts; `dispatched` the as-sent parts.

<details>
<summary>Source</summary>

```rust
async fn dispatch_observed(
    policy: Arc<dyn EgressPolicy>,
    original: http::request::Parts,
    dispatched: http::request::Parts,
    body: HyperOutgoingBody,
    capture: CaptureHandle,
    config: OutgoingRequestConfig,
) -> Result<IncomingResponse, ErrorCode> {
    let retry_config = copy_config(&config);
    // A transport error (`Err`) propagates to the guest exactly as today —
    // `on_response` observes only actual responses.
    let first =
        default_send_request_handler(http::Request::from_parts(dispatched.clone(), body), config)
            .await?;

    let replay = capture.replayable();
    let directive = policy.on_response(
        &dispatched,
        first.resp.status(),
        first.resp.headers(),
        replay.is_some(),
    );
    let (ResponseDirective::RetryOnce, Some(bytes)) = (directive, replay) else {
        return Ok(first);
    };

    // Discard the observed response (dropping it aborts its body worker) and
    // re-authorize a clean clone of the original parts — the policy injects
    // its fresh credential here. A denial is terminal: the policy consumed
    // the response and then refused to re-stamp.
    drop(first);
    let mut retry_parts = original;
    if policy.authorize(&mut retry_parts).is_err() {
        return Err(ErrorCode::HttpRequestDenied);
    }
    let second = default_send_request_handler(
        http::Request::from_parts(retry_parts.clone(), replay_body(bytes)),
        retry_config,
    )
    .await?;
    // Observability only: `retry_available = false`, directive ignored — the
    // second response forwards unconditionally.
    let _ = policy.on_response(
        &retry_parts,
        second.resp.status(),
        second.resp.headers(),
        false,
    );
    Ok(second)
}
```

</details>



### `fidius-host::executor::wasm::name_lookup_view`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn name_lookup_view (state : & mut HostState) -> NameLookupView < '_ >
```

Accessor handed to `ip_name_lookup::add_to_linker` for the shadowed instance: project the store's state into the lookup view (FIDIUS-I-0034). The policy rides in via the (always-present) `EgressHooks`.

<details>
<summary>Source</summary>

```rust
fn name_lookup_view(state: &mut HostState) -> NameLookupView<'_> {
    NameLookupView {
        table: &mut state.table,
        pins: &state.pins,
        policy: state.hooks.policy.as_ref(),
        resolver: &state.resolver,
    }
}
```

</details>



### `fidius-host::executor::wasm::dispatch_host_table`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn dispatch_host_table (table : & fidius_core :: host_ffi :: HostFunctionTable , index : u32 , args : & [u8] ,) -> (i32 , Vec < u8 >)
```

Run one host-function dispatch through a bound table and return the raw `(status, payload)` pair for the guest, copying the host-owned output buffer and releasing it via the table's `free_buffer`.

<details>
<summary>Source</summary>

```rust
fn dispatch_host_table(
    table: &fidius_core::host_ffi::HostFunctionTable,
    index: u32,
    args: &[u8],
) -> (i32, Vec<u8>) {
    let mut out_ptr: *mut u8 = std::ptr::null_mut();
    let mut out_len: u32 = 0;
    // SAFETY: the table was validated at bind time; dispatch/free_buffer are
    // process-lifetime function pointers per the HostFunctionTable contract.
    let status = unsafe {
        (table.dispatch)(
            table.ctx,
            index,
            args.as_ptr(),
            args.len() as u32,
            &mut out_ptr,
            &mut out_len,
        )
    };
    let payload = if out_ptr.is_null() || out_len == 0 {
        Vec::new()
    } else {
        // SAFETY: the host wrote a valid buffer of out_len bytes; copy it out
        // and hand it straight back to the host's free_buffer.
        unsafe {
            let bytes = std::slice::from_raw_parts(out_ptr, out_len as usize).to_vec();
            (table.free_buffer)(out_ptr, out_len as usize);
            bytes
        }
    };
    (status, payload)
}
```

</details>



### `fidius-host::executor::wasm::validate_capabilities`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn validate_capabilities (caps : & [String]) -> Result < () , CallError >
```

Reject unknown capability names early (at load) so a typo fails closed and loud rather than silently granting nothing.

<details>
<summary>Source</summary>

```rust
fn validate_capabilities(caps: &[String]) -> Result<(), CallError> {
    // FIDIUS-I-0033: the policy-gated egress tier (`tcp`/`udp`) and the coarse
    // `network`/`sockets` grant both install a single `socket_addr_check` in
    // `build_wasi_ctx`, and that builder field is last-call-wins. Declaring both
    // tiers would silently keep only one check (which one depends on capability
    // order) — either discarding the embedder's per-peer policy or the SSRF floor.
    // Reject the combination outright so the gate an operator vets is the gate that
    // runs. (`tcp` and `udp` *do* compose — they share one dispatching check.)
    let wants_policy_egress = caps.iter().any(|c| c == "tcp" || c == "udp");
    let wants_network = caps.iter().any(|c| c == "network" || c == "sockets");
    if wants_policy_egress && wants_network {
        return Err(CallError::Backend {
            runtime: "wasm".into(),
            message: "wasm capabilities 'tcp'/'udp' and 'network'/'sockets' are \
                      mutually exclusive: 'tcp'/'udp' are per-peer policy-gated \
                      (EgressPolicy::authorize_tcp/authorize_udp) while \
                      'network'/'sockets' is coarse SSRF-floored access — granting \
                      both would silently keep only one gate. Pick one tier."
                .into(),
        });
    }
    for c in caps {
        // Bare `env` (inherit ALL host env vars — i.e. every secret) is no longer
        // grantable (FIDIUS-T-0142). Point the author at the scoped form.
        if c == "env" {
            return Err(CallError::Backend {
                runtime: "wasm".into(),
                message: "wasm capability 'env' grants ALL host environment variables (every \
                          secret) and is not allowed; grant specific variables with \
                          'env:VAR_NAME' instead"
                    .into(),
            });
        }
        // Scoped env grant: `env:VAR_NAME` exposes exactly that one variable.
        if let Some(name) = c.strip_prefix("env:") {
            if name.is_empty() {
                return Err(CallError::Backend {
                    runtime: "wasm".into(),
                    message: "wasm capability 'env:' requires a variable name (e.g. \
                              'env:STRIPE_API_BASE')"
                        .into(),
                });
            }
            continue;
        }
        // Path-scoped filesystem (FIDIUS-A-0008): `fs:ro:<path>` / `fs:rw:<path>`
        // preopen exactly that directory. Bare `fs`/`filesystem` (whole-FS) is
        // rejected — like bare `env`, a coarse grant is a footgun.
        if c == "fs" || c == "filesystem" {
            return Err(CallError::Backend {
                runtime: "wasm".into(),
                message: "wasm filesystem is path-scoped; grant a directory with \
                          'fs:ro:<path>' (read-only) or 'fs:rw:<path>' — bare \
                          'fs'/'filesystem' (whole filesystem) is not allowed"
                    .into(),
            });
        }
        if let Some(path) = c
            .strip_prefix("fs:ro:")
            .or_else(|| c.strip_prefix("fs:rw:"))
        {
            if path.is_empty() {
                return Err(CallError::Backend {
                    runtime: "wasm".into(),
                    message: "wasm capability 'fs:ro:'/'fs:rw:' requires a path (e.g. \
                              'fs:ro:/data')"
                        .into(),
                });
            }
            continue;
        }
        if !KNOWN_CAPABILITIES.contains(&c.as_str()) {
            return Err(CallError::Backend {
                runtime: "wasm".into(),
                message: format!(
                    "unknown wasm capability '{c}'; allowed: {}, env:VAR_NAME",
                    KNOWN_CAPABILITIES.join(", ")
                ),
            });
        }
    }
    Ok(())
}
```

</details>



### `fidius-host::executor::wasm::build_wasi_ctx`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn build_wasi_ctx (caps : & [String] , egress : Option < Arc < dyn EgressPolicy > > , pins : PinTable ,) -> WasiCtx
```

Build a `WasiCtx` from the allow-list. Starts deny-all (a fresh builder inherits nothing and has no preopens) and grants only what's listed. Filesystem is granted only per `fs:ro:<path>` / `fs:rw:<path>` — a path-scoped preopen, never the whole filesystem (FIDIUS-A-0008).

`pins` is the same table the store's shadowed `ip-name-lookup` writes
(FIDIUS-I-0034) — the `socket_addr_check` installed here reads it to
recover the hostname the guest dialed for a connect's IP.

<details>
<summary>Source</summary>

```rust
fn build_wasi_ctx(
    caps: &[String],
    egress: Option<Arc<dyn EgressPolicy>>,
    pins: PinTable,
) -> WasiCtx {
    let mut b = WasiCtxBuilder::new();
    // FIDIUS-I-0033: the policy-gated egress tier (`tcp`/`udp`) shares ONE
    // `socket_addr_check`, which is last-call-wins on the builder. We accumulate
    // the grants here and install a single dispatching check after the loop, so
    // declaring both `tcp` and `udp` composes instead of one silently clobbering
    // the other. (The coarse `network`/`sockets` tier installs its own check and
    // is mutually exclusive with these — rejected in `validate_capabilities`.)
    let mut wants_tcp = false;
    let mut wants_udp = false;
    for c in caps {
        let c = c.as_str();
        match c {
            "args" => {
                b.inherit_args();
            }
            "stdout" => {
                b.inherit_stdout();
            }
            "stderr" => {
                b.inherit_stderr();
            }
            "stdin" => {
                b.inherit_stdin();
            }
            // Raw outbound sockets (coarse — no per-host policy). FIDIUS-T-0143:
            // a baseline SSRF floor rejects loopback/link-local/private/metadata
            // targets. The check runs on the *resolved* `SocketAddr`, so it also
            // catches a hostname that resolves (or rebinds) to an internal IP.
            // For host-brokered, per-host-policied egress prefer `http`.
            "network" | "sockets" => {
                b.inherit_network();
                b.allow_ip_name_lookup(true);
                b.socket_addr_check(|addr, _use| {
                    let ok = !is_blocked_ip(&addr.ip());
                    Box::pin(async move { ok }) as Pin<Box<dyn Future<Output = bool> + Send + Sync>>
                });
            }
            // Always available in WASI; accepted as a no-op (intent marker).
            "clocks" | "random" => {}
            // Egress is wired at the linker level (two-key with the embedder's
            // EgressPolicy), not via the WasiCtx — no-op here.
            "http" => {}
            // FIDIUS-I-0033: policy-gated outbound TCP — the second key is the
            // embedder's `EgressPolicy::authorize_tcp`. We grant `wasi:sockets`
            // narrowly: TCP only (no UDP), name-lookup on (so the guest can dial a
            // hostname — `std::net::TcpStream::connect(("db", 5432))` resolves via
            // wasi:sockets first), and a per-connection check that routes every
            // *resolved* peer through the policy. Bind/listen and UDP are rejected
            // outright, so this is strictly outbound `tcp.connect`.
            //
            // Two-key, fail-closed: with no policy we install NO socket check, so
            // the deny-all default stands and every connect is refused — granting
            // `tcp` without a host policy reaches nothing (the `wasi:http` analog:
            // there the imports are simply absent).
            "tcp" => {
                wants_tcp = true;
            }
            // FIDIUS-I-0033: policy-gated outbound UDP — the symmetric counterpart
            // of `tcp`. The second key is the embedder's
            // `EgressPolicy::authorize_udp`, consulted on the resolved peer of every
            // outbound datagram (`UdpConnect`/`UdpOutgoingDatagram`). The local
            // `UdpBind` (binding an ephemeral source socket, the addr is local, not
            // a peer) is permitted as setup; inbound and TCP uses are refused. Same
            // two-key fail-closed shape: no policy → no check installed → deny-all.
            "udp" => {
                wants_udp = true;
            }
            // Scoped env (FIDIUS-T-0142, ADR FIDIUS-A-0009): `env:VAR_NAME` exposes
            // exactly that one host variable (skipped silently if unset on the host)
            // — never the whole environment. Bare `env` is rejected in
            // `validate_capabilities`.
            _ if c.starts_with("env:") => {
                let name = &c["env:".len()..];
                if let Ok(val) = std::env::var(name) {
                    b.env(name, val);
                }
            }
            // Path-scoped filesystem (FIDIUS-A-0008): preopen exactly the granted
            // host directory at the same guest path. WASI's capability model scopes
            // the guest to the preopen (no traversal escape). A non-existent path is
            // skipped — the guest's open() then fails with a normal WASI error.
            _ if c.starts_with("fs:ro:") => {
                let path = &c["fs:ro:".len()..];
                let _ = b.preopened_dir(path, path, DirPerms::READ, FilePerms::READ);
            }
            _ if c.starts_with("fs:rw:") => {
                let path = &c["fs:rw:".len()..];
                let _ = b.preopened_dir(path, path, DirPerms::all(), FilePerms::all());
            }
            _ => {}
        }
    }
    // FIDIUS-I-0033: install the single policy-gated egress check for `tcp`/`udp`.
    // Two-key, fail-closed: only when a grant is present AND the embedder supplied
    // an `EgressPolicy`. With no policy we install NO check, so the deny-all
    // default stands and every connect/datagram is refused — granting `tcp`/`udp`
    // without a host policy reaches nothing.
    if wants_tcp || wants_udp {
        if let Some(policy) = egress.clone() {
            // Set the use-flags precisely (both default to `true` in wasmtime-wasi,
            // so a `tcp`-only grant must explicitly turn UDP off, and vice versa).
            b.allow_tcp(wants_tcp);
            b.allow_udp(wants_udp);
            // The guest may dial a hostname — std resolves via wasi:sockets first,
            // then the resolved peer is gated below.
            b.allow_ip_name_lookup(true);
            b.socket_addr_check(move |addr, use_| {
                // Route each resolved peer through the matching hook; fidius ships
                // the mechanism, the embedder's policy is the allow-list.
                let allowed = match use_ {
                    // Outbound TCP connect → authorize_tcp_target (FIDIUS-I-0034),
                    // with the dialed hostname recovered from this store's
                    // resolve-and-pin table. An IP-literal dial (no lookup, no
                    // pin) arrives as `host: None`, and the default delegation
                    // to `authorize_tcp` keeps host-unaware policies
                    // byte-identical.
                    SocketAddrUse::TcpConnect => {
                        let host = pins.lock().unwrap().host_for(&addr.ip().to_canonical());
                        wants_tcp
                            && policy
                                .authorize_tcp_target(&TcpTarget {
                                    host: host.as_deref(),
                                    addr,
                                })
                                .is_ok()
                    }
                    // Outbound UDP (connected send or one-shot datagram) →
                    // authorize_udp, on the remote peer.
                    SocketAddrUse::UdpConnect | SocketAddrUse::UdpOutgoingDatagram => {
                        wants_udp && policy.authorize_udp(&addr).is_ok()
                    }
                    // Binding the local UDP source socket is setup, not a peer; allow
                    // it only as part of an active `udp` grant (the actual peer is
                    // gated on connect/send above).
                    SocketAddrUse::UdpBind => wants_udp,
                    // Inbound TCP (bind/listen) is never reachable through this tier.
                    SocketAddrUse::TcpBind => false,
                };
                Box::pin(async move { allowed })
                    as Pin<Box<dyn Future<Output = bool> + Send + Sync>>
            });
        }
    }
    b.build()
}
```

</details>



### `fidius-host::executor::wasm::is_blocked_ip`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn is_blocked_ip (ip : & IpAddr) -> bool
```

Baseline SSRF denylist for the raw-socket grant (FIDIUS-T-0143): an address a sandboxed guest must never reach — loopback, link-local (incl. the cloud metadata IP `169.254.169.254`), private (RFC-1918), unique-local, unspecified, or broadcast. This is a safety *floor* (like deny-all), not a full egress policy; per-host policy is the embedder's job via the `http` capability.

<details>
<summary>Source</summary>

```rust
fn is_blocked_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_link_local()
                || v4.is_private()
                || v4.is_unspecified()
                || v4.is_broadcast()
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // link-local fe80::/10
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // unique-local fc00::/7
                || v6
                    .to_ipv4_mapped()
                    .is_some_and(|m| is_blocked_ip(&IpAddr::V4(m)))
        }
    }
}
```

</details>



### `fidius-host::executor::wasm::wasi_http_incompatibility`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn wasi_http_incompatibility < 'a > (import_names : impl Iterator < Item = & 'a str >) -> Option < String >
```

Scan a component's import names for a `wasi:http` version this host can't satisfy, returning a clear, actionable message if so (FIDIUS-A-0005, fail loud — the same discipline as the `ABI_VERSION` check, on a new axis).

Compatible iff the import is on the host's `major.minor` line and the host's
patch is `>=` the plugin's (WASI 0.2 is forward-compatible: a newer host
satisfies an older import, never the reverse). A host *behind* the plugin, or
a different line (`0.2`→`0.3`), is rejected up front instead of surfacing as a
cryptic instantiate trap. Pulled out as a free fn so it unit-tests without a
real component.

<details>
<summary>Source</summary>

```rust
fn wasi_http_incompatibility<'a>(import_names: impl Iterator<Item = &'a str>) -> Option<String> {
    let (hmaj, hmin, hpat) = HOST_WASI_HTTP;
    for name in import_names {
        let Some(rest) = name.strip_prefix("wasi:http/") else {
            continue;
        };
        let Some(ver) = rest.split('@').nth(1) else {
            continue;
        };
        let parts: Vec<&str> = ver.split('.').collect();
        if parts.len() != 3 {
            continue;
        }
        let (Ok(maj), Ok(min), Ok(pat)) = (
            parts[0].parse::<u32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<u32>(),
        ) else {
            continue;
        };
        if maj == hmaj && min == hmin && pat <= hpat {
            return None; // a compatible wasi:http import — nothing to flag
        }
        return Some(format!(
            "plugin requires wasi:http {maj}.{min}.{pat}, but this host provides \
             {hmaj}.{hmin}.{hpat} — upgrade the host (newer wasmtime) or rebuild the \
             plugin against an older fidius-guest"
        ));
    }
    None
}
```

</details>



### `fidius-host::executor::wasm::plugin_error_from_val`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn plugin_error_from_val (payload : Option < & Val >) -> CallError
```

Map a `result::err` payload (expected: a record with `code`/`message`/ `details`) into a `PluginError`.

<details>
<summary>Source</summary>

```rust
fn plugin_error_from_val(payload: Option<&Val>) -> CallError {
    use fidius_core::PluginError;
    let mut code = "WASM_ERROR".to_string();
    let mut message = String::new();
    let mut details: Option<String> = None;
    if let Some(Val::Record(fields)) = payload {
        for (k, v) in fields {
            match (k.as_str(), v) {
                ("code", Val::String(s)) => code = s.clone(),
                ("message", Val::String(s)) => message = s.clone(),
                ("details", Val::Option(Some(b))) => {
                    if let Val::String(s) = b.as_ref() {
                        details = Some(s.clone());
                    }
                }
                _ => {}
            }
        }
    } else if let Some(other) = payload {
        message = format!("{other:?}");
    }
    let mut err = PluginError::new(code, message);
    if let Some(d) = details {
        err.details = Some(d);
    }
    CallError::Plugin(err)
}
```

</details>



### `fidius-host::executor::wasm::to_kebab`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn to_kebab (s : & str) -> String
```

fidius `Value` → wasmtime `Val`. Mirrors the Phase-1 serde bridge shapes. Rust identifier (snake_case / PascalCase) → kebab-case, matching the WIT naming the generator uses. `y_pos`→`y-pos`, `Circle`→`circle`.

<details>
<summary>Source</summary>

```rust
fn to_kebab(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch == '_' {
            out.push('-');
        } else if ch.is_uppercase() {
            if i != 0 {
                out.push('-');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}
```

</details>



### `fidius-host::executor::wasm::kebab_to_snake`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn kebab_to_snake (s : & str) -> String
```

kebab-case → snake_case (WIT record field → serde struct field).

<details>
<summary>Source</summary>

```rust
fn kebab_to_snake(s: &str) -> String {
    s.replace('-', "_")
}
```

</details>



### `fidius-host::executor::wasm::kebab_to_pascal`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn kebab_to_pascal (s : & str) -> String
```

kebab-case → PascalCase (WIT variant case → serde enum variant).

<details>
<summary>Source</summary>

```rust
fn kebab_to_pascal(s: &str) -> String {
    s.split('-')
        .map(|seg| {
            let mut c = seg.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect()
}
```

</details>



### `fidius-host::executor::wasm::value_to_val`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn value_to_val (v : & Value) -> Result < Val , CallError >
```

<details>
<summary>Source</summary>

```rust
fn value_to_val(v: &Value) -> Result<Val, CallError> {
    Ok(match v {
        Value::Bool(b) => Val::Bool(*b),
        Value::S8(x) => Val::S8(*x),
        Value::S16(x) => Val::S16(*x),
        Value::S32(x) => Val::S32(*x),
        Value::S64(x) => Val::S64(*x),
        Value::U8(x) => Val::U8(*x),
        Value::U16(x) => Val::U16(*x),
        Value::U32(x) => Val::U32(*x),
        Value::U64(x) => Val::U64(*x),
        Value::F32(x) => Val::Float32(*x),
        Value::F64(x) => Val::Float64(*x),
        Value::Char(c) => Val::Char(*c),
        Value::String(s) => Val::String(s.clone()),
        Value::Bytes(b) => Val::List(b.iter().map(|x| Val::U8(*x)).collect()),
        Value::List(items) => Val::List(items.iter().map(value_to_val).collect::<Result<_, _>>()?),
        // Record/variant names cross as kebab-case (the WIT convention) — serde
        // produces snake/PascalCase, so normalize here and un-normalize on the
        // way back (see `val_to_value`).
        Value::Record(fields) => Val::Record(
            fields
                .iter()
                .map(|(k, v)| Ok::<_, CallError>((to_kebab(k), value_to_val(v)?)))
                .collect::<Result<_, _>>()?,
        ),
        Value::Option(None) => Val::Option(None),
        Value::Option(Some(inner)) => Val::Option(Some(Box::new(value_to_val(inner)?))),
        Value::Variant { name, value } => {
            // Unit-payload variant → no payload; else carry the lowered value.
            let payload = match value.as_ref() {
                Value::Unit => None,
                other => Some(Box::new(value_to_val(other)?)),
            };
            Val::Variant(to_kebab(name), payload)
        }
        Value::Unit => Val::Tuple(Vec::new()),
        // A map has no native WIT type — it projects to `list<tuple<k, v>>`
        // (FIDIUS-A-0008/PC.1), which is unambiguous from a `Value::Map`.
        Value::Map(pairs) => Val::List(
            pairs
                .iter()
                .map(|(k, v)| {
                    Ok::<_, CallError>(Val::Tuple(vec![value_to_val(k)?, value_to_val(v)?]))
                })
                .collect::<Result<_, _>>()?,
        ),
    })
}
```

</details>



### `fidius-host::executor::wasm::value_to_val_typed`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn value_to_val_typed (v : & Value , ty : & wasmtime :: component :: Type) -> Result < Val , CallError >
```

Type-directed lowering for the **argument** path. The structural [`value_to_val`] can't tell a Rust tuple (a `Value::List`) from a real list, so when the target WIT type is a `tuple<…>` we use the wasmtime [`Type`] to emit `Val::Tuple`. Lists, options, and maps recurse with their element type so nested tuples are caught; everything else falls back to the structural lowering.

<details>
<summary>Source</summary>

```rust
fn value_to_val_typed(v: &Value, ty: &wasmtime::component::Type) -> Result<Val, CallError> {
    use wasmtime::component::Type;
    match ty {
        Type::Tuple(tt) => {
            let types: Vec<Type> = tt.types().collect();
            let items: Vec<Value> = match v {
                Value::List(items) => items.clone(),
                Value::Unit if types.is_empty() => Vec::new(),
                other => {
                    return Err(CallError::Serialization(format!(
                        "expected a tuple value (got {other:?}) for a WIT tuple<…>"
                    )))
                }
            };
            if items.len() != types.len() {
                return Err(CallError::Serialization(format!(
                    "tuple arity mismatch: value has {}, WIT tuple has {}",
                    items.len(),
                    types.len()
                )));
            }
            Ok(Val::Tuple(
                items
                    .iter()
                    .zip(types.iter())
                    .map(|(it, t)| value_to_val_typed(it, t))
                    .collect::<Result<_, _>>()?,
            ))
        }
        Type::List(lt) => {
            let elem = lt.ty();
            match v {
                Value::List(items) => Ok(Val::List(
                    items
                        .iter()
                        .map(|i| value_to_val_typed(i, &elem))
                        .collect::<Result<_, _>>()?,
                )),
                Value::Bytes(b) => Ok(Val::List(b.iter().map(|x| Val::U8(*x)).collect())),
                // A map lowered to `list<tuple<k, v>>`: each pair becomes a 2-tuple.
                Value::Map(pairs) => Ok(Val::List(
                    pairs
                        .iter()
                        .map(|(k, val)| {
                            value_to_val_typed(&Value::List(vec![k.clone(), val.clone()]), &elem)
                        })
                        .collect::<Result<_, _>>()?,
                )),
                // A string-keyed map serializes to `Value::Record`; its field names
                // are the (string) keys. Project to the same list-of-pairs.
                Value::Record(fields) => Ok(Val::List(
                    fields
                        .iter()
                        .map(|(k, val)| {
                            value_to_val_typed(
                                &Value::List(vec![Value::String(k.clone()), val.clone()]),
                                &elem,
                            )
                        })
                        .collect::<Result<_, _>>()?,
                )),
                other => Err(CallError::Serialization(format!(
                    "expected a list/map value (got {other:?}) for a WIT list<…>"
                ))),
            }
        }
        Type::Option(ot) => match v {
            Value::Option(None) => Ok(Val::Option(None)),
            Value::Option(Some(inner)) => Ok(Val::Option(Some(Box::new(value_to_val_typed(
                inner,
                &ot.ty(),
            )?)))),
            _ => value_to_val(v),
        },
        // Record: thread each field's declared type through, so a tuple (or map, or
        // nested record) inside a record field lowers correctly — `Value::List` alone
        // can't distinguish a tuple from a list, so the structural path would mis-lower a
        // tuple-valued field as `Val::List` and wasmtime would reject it (FIDIUS-T-0160).
        Type::Record(rt) => match v {
            Value::Record(fields) => {
                // Value field names are serde (snake/Pascal); WIT fields are kebab.
                let mut by_kebab: std::collections::HashMap<String, &Value> =
                    fields.iter().map(|(k, val)| (to_kebab(k), val)).collect();
                let lowered = rt
                    .fields()
                    .map(|f| {
                        let val = by_kebab.remove(f.name).ok_or_else(|| {
                            CallError::Serialization(format!(
                                "record value is missing field '{}' (for a WIT record)",
                                f.name
                            ))
                        })?;
                        Ok::<_, CallError>((f.name.to_string(), value_to_val_typed(val, &f.ty)?))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Val::Record(lowered))
            }
            other => Err(CallError::Serialization(format!(
                "expected a record value (got {other:?}) for a WIT record {{ … }}"
            ))),
        },
        // Primitives, variants, results: structural lowering is unambiguous.
        _ => value_to_val(v),
    }
}
```

</details>



### `fidius-host::executor::wasm::val_to_value`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn val_to_value (v : & Val) -> Value
```

wasmtime `Val` → fidius `Value` (structural; self-describing).

<details>
<summary>Source</summary>

```rust
fn val_to_value(v: &Val) -> Value {
    match v {
        Val::Bool(b) => Value::Bool(*b),
        Val::S8(x) => Value::S8(*x),
        Val::S16(x) => Value::S16(*x),
        Val::S32(x) => Value::S32(*x),
        Val::S64(x) => Value::S64(*x),
        Val::U8(x) => Value::U8(*x),
        Val::U16(x) => Value::U16(*x),
        Val::U32(x) => Value::U32(*x),
        Val::U64(x) => Value::U64(*x),
        Val::Float32(x) => Value::F32(*x),
        Val::Float64(x) => Value::F64(*x),
        Val::Char(c) => Value::Char(*c),
        Val::String(s) => Value::String(s.clone()),
        Val::List(items) => Value::List(items.iter().map(val_to_value).collect()),
        Val::Record(fields) => Value::Record(
            fields
                .iter()
                .map(|(k, v)| (kebab_to_snake(k), val_to_value(v)))
                .collect(),
        ),
        Val::Tuple(items) => Value::List(items.iter().map(val_to_value).collect()),
        Val::Option(None) => Value::Option(None),
        Val::Option(Some(inner)) => Value::Option(Some(Box::new(val_to_value(inner)))),
        Val::Variant(name, payload) => Value::Variant {
            name: kebab_to_pascal(name),
            value: Box::new(payload.as_deref().map(val_to_value).unwrap_or(Value::Unit)),
        },
        Val::Enum(name) => Value::Variant {
            name: kebab_to_pascal(name),
            value: Box::new(Value::Unit),
        },
        Val::Result(Ok(inner)) => inner.as_deref().map(val_to_value).unwrap_or(Value::Unit),
        Val::Result(Err(inner)) => inner.as_deref().map(val_to_value).unwrap_or(Value::Unit),
        // Flags / Resource have no fidius Value equivalent in v1.
        other => Value::String(format!("{other:?}")),
    }
}
```

</details>



### `fidius-host::executor::wasm::validate_component`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn validate_component (bytes : & [u8]) -> Result < () , CallError >
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



### `fidius-host::executor::wasm::precompile_component`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn precompile_component (bytes : & [u8]) -> Result < Vec < u8 > , CallError >
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



