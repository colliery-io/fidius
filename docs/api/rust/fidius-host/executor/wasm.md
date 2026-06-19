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
            None,
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
        })
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
        let host = HostState {
            ctx: build_wasi_ctx(&self.capabilities),
            table: ResourceTable::new(),
            http_ctx: WasiHttpCtx::new(),
            hooks: EgressHooks {
                policy: self.egress.clone(),
            },
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





## Functions

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
fn build_wasi_ctx (caps : & [String]) -> WasiCtx
```

Build a `WasiCtx` from the allow-list. Starts deny-all (a fresh builder inherits nothing and has no preopens) and grants only what's listed. Filesystem is never granted.

<details>
<summary>Source</summary>

```rust
fn build_wasi_ctx(caps: &[String]) -> WasiCtx {
    let mut b = WasiCtxBuilder::new();
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
            // Scoped env (FIDIUS-T-0142): `env:VAR_NAME` exposes exactly that one
            // host variable (skipped silently if unset on the host) — never the
            // whole environment. Bare `env` is rejected in `validate_capabilities`.
            _ if c.starts_with("env:") => {
                let name = &c["env:".len()..];
                if let Ok(val) = std::env::var(name) {
                    b.env(name, val);
                }
            }
            _ => {}
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
        Value::Map(_) => {
            return Err(CallError::Serialization(
                "non-string-keyed maps are not yet supported across the WASM boundary".into(),
            ))
        }
    })
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



