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

//! `WasmComponentExecutor` — the sandboxed WASM (Component Model) backend.
//!
//! FIDIUS-I-0021 Phase 2, ADR FIDIUS-A-0003 (Path B). The **only** module that
//! depends on `wasmtime`; it maps the neutral [`fidius_core::Value`] to/from
//! `wasmtime::component::Val` and dispatches by method index into a loaded
//! component's exported interface.
//!
//! Sandbox model (human-ratified, FIDIUS-T-0102 finding): real std-built
//! components import `wasi:cli/io/clocks/filesystem` even when unused, so an
//! *empty* `Linker` can't instantiate them. We wire `wasmtime-wasi` into the
//! `Linker` but give the guest a **zero-grant `WasiCtx`** (no FS preopens, no
//! env, no inherited stdio, no sockets). T-0104 opens specific capabilities
//! from the package manifest's allow-list.

use std::sync::Arc;

use fidius_core::Value;
use wasmtime::component::{Component, InstancePre, Linker, Val};
use wasmtime::{Engine, Store};
use wasmtime_wasi::p2::add_to_linker_sync;
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};
use wasmtime_wasi_http::p2::bindings::http::types::ErrorCode;
use wasmtime_wasi_http::p2::body::HyperOutgoingBody;
use wasmtime_wasi_http::p2::types::{HostFutureIncomingResponse, OutgoingRequestConfig};
use wasmtime_wasi_http::p2::{
    add_only_http_to_linker_sync, default_send_request, HttpResult, WasiHttpCtxView, WasiHttpHooks,
    WasiHttpView,
};
use wasmtime_wasi_http::WasiHttpCtx;

use crate::error::CallError;
use crate::executor::{PluginExecutor, ValueExecutor};
use crate::types::PluginInfo;

/// Denial returned by an [`EgressPolicy`] to refuse an outbound request.
#[derive(Debug, Clone)]
pub struct EgressDenied {
    /// Human-readable reason (for the embedder's logs; not shown to the guest,
    /// which only sees a generic HTTP "request denied").
    pub reason: String,
}

impl EgressDenied {
    /// A denial with a reason.
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

/// Embedder-supplied policy governing a sandboxed WASM guest's **outbound HTTP**
/// (FIDIUS-I-0027). This is the *only* egress seam fidius ships — it contains
/// **no** allow-list, SSRF, or credential logic; those are deployment-specific
/// policy the embedder implements here.
///
/// `wasi:http` is enabled for a guest only when its package declares the `http`
/// capability **and** a `PluginHost`/executor was given one of these (two-key,
/// fail-closed). [`authorize`](EgressPolicy::authorize) is then called for
/// **every** outbound request the guest makes — every request is a host call
/// across the sandbox boundary, so this is a true per-request checkpoint, not a
/// one-time gate. Inspect `parts.uri` / `parts.method`, mutate `parts.headers`
/// to inject credentials, or return `Err(EgressDenied)` to refuse (the guest
/// then sees an HTTP error and the request is never dispatched).
pub trait EgressPolicy: Send + Sync + 'static {
    /// Authorize (and optionally decorate) one outbound request before dispatch.
    fn authorize(&self, parts: &mut http::request::Parts) -> Result<(), EgressDenied>;
}

/// fidius's [`WasiHttpHooks`] adapter: routes every outbound request through the
/// embedder's [`EgressPolicy`] before handing off to wasi-http's
/// `default_send_request`. `policy: None` denies everything (defensive — the
/// http imports are never linked without a policy, so this is unreachable in
/// practice).
struct EgressHooks {
    policy: Option<Arc<dyn EgressPolicy>>,
}

impl WasiHttpHooks for EgressHooks {
    fn send_request(
        &mut self,
        request: http::Request<HyperOutgoingBody>,
        config: OutgoingRequestConfig,
    ) -> HttpResult<HostFutureIncomingResponse> {
        let Some(policy) = self.policy.as_ref() else {
            return Err(ErrorCode::HttpRequestDenied.into());
        };
        // Split off the body so the policy works in pure `http`-crate types,
        // then reassemble for dispatch.
        let (mut parts, body) = request.into_parts();
        if policy.authorize(&mut parts).is_err() {
            return Err(ErrorCode::HttpRequestDenied.into());
        }
        Ok(default_send_request(
            http::Request::from_parts(parts, body),
            config,
        ))
    }
}

/// Per-store host state. The `WasiCtx` is built from the capability allow-list
/// (deny-all baseline) by `build_wasi_ctx`. `http_ctx`/`hooks` back the optional
/// `wasi:http` egress (FIDIUS-I-0027); they're inert unless egress was enabled.
struct HostState {
    ctx: WasiCtx,
    table: ResourceTable,
    http_ctx: WasiHttpCtx,
    hooks: EgressHooks,
}

impl WasiHttpView for HostState {
    fn http(&mut self) -> WasiHttpCtxView<'_> {
        WasiHttpCtxView {
            ctx: &mut self.http_ctx,
            table: &mut self.table,
            hooks: &mut self.hooks,
        }
    }
}

/// Capabilities the host knows how to grant. **Filesystem is intentionally
/// absent** — it is never granted in v1 (no preopens, ever). `clocks`/`random`
/// are always available in WASI and are accepted as no-ops so manifests can
/// declare intent without error.
const KNOWN_CAPABILITIES: &[&str] = &[
    "env", "args", "stdout", "stderr", "stdin", "network", "sockets", "clocks", "random",
    // FIDIUS-I-0027: declares the guest *wants* brokered outbound HTTP. Actual
    // egress also requires the embedder to supply an `EgressPolicy` (two-key);
    // handled in `build`, not `build_wasi_ctx`.
    "http",
];

/// Reject unknown capability names early (at load) so a typo fails closed and
/// loud rather than silently granting nothing.
fn validate_capabilities(caps: &[String]) -> Result<(), CallError> {
    for c in caps {
        if !KNOWN_CAPABILITIES.contains(&c.as_str()) {
            return Err(CallError::Backend {
                runtime: "wasm".into(),
                message: format!(
                    "unknown wasm capability '{c}'; allowed: {}",
                    KNOWN_CAPABILITIES.join(", ")
                ),
            });
        }
    }
    Ok(())
}

/// Build a `WasiCtx` from the allow-list. Starts deny-all (a fresh builder
/// inherits nothing and has no preopens) and grants only what's listed.
/// Filesystem is never granted.
fn build_wasi_ctx(caps: &[String]) -> WasiCtx {
    let mut b = WasiCtxBuilder::new();
    for c in caps {
        match c.as_str() {
            "env" => {
                b.inherit_env();
            }
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
            // Network egress. Coarse in v1 (per-host:port filtering is a
            // documented follow-on); still opt-in and off by default.
            "network" | "sockets" => {
                b.inherit_network();
                b.allow_ip_name_lookup(true);
            }
            // Always available in WASI; accepted as a no-op (intent marker).
            "clocks" | "random" => {}
            // Egress is wired at the linker level (two-key with the embedder's
            // EgressPolicy), not via the WasiCtx — no-op here.
            "http" => {}
            _ => {}
        }
    }
    b.build()
}

// wasmtime-wasi 45: `IoView` was merged into `WasiView`, whose `ctx` returns a
// `WasiCtxView<'_>` borrowing both the ctx and the resource table.
impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

/// A method on the WASM interface, in declaration (vtable) order.
#[derive(Debug, Clone)]
pub struct WasmMethod {
    /// Export name within the interface (e.g. `"greet"`).
    pub name: String,
    /// Whether this method uses `#[wire(raw)]` (bytes in/out).
    pub wire_raw: bool,
    /// Whether this method is server-streaming (`-> fidius::Stream<T>`); the
    /// export returns a `next()`-pollable resource the host pumps (WS.3).
    pub streaming: bool,
}

/// WASM component execution backend.
pub struct WasmComponentExecutor {
    engine: Engine,
    /// Pre-linked component (Linker + WASI wired in, typechecked) built once at
    /// load. Per call we only create a fresh `Store` and `instance_pre.instantiate`
    /// — instantiation stays per-call (isolation) but the expensive linking is
    /// done once, not on every call (FIDIUS-I-0024).
    instance_pre: InstancePre<HostState>,
    /// Fully-qualified exported interface name, e.g.
    /// `"fidius:greeter/greeter@1.0.0"`.
    interface: String,
    /// Methods in interface order; index = the vtable index callers use.
    methods: Vec<WasmMethod>,
    /// WASI capability allow-list from `[wasm].capabilities`. Empty = deny-all.
    /// Filesystem is never granted regardless.
    capabilities: Vec<String>,
    /// Embedder egress policy (FIDIUS-I-0027). `Some` + the `http` capability is
    /// the two-key that links `wasi:http`; otherwise egress is impossible.
    egress: Option<Arc<dyn EgressPolicy>>,
    info: PluginInfo,
}

impl WasmComponentExecutor {
    /// Build an executor from raw component bytes (a `.wasm` component). For the
    /// AOT fast path, prefer [`Self::from_cwasm`].
    pub fn from_component_bytes(
        bytes: &[u8],
        interface: String,
        methods: Vec<WasmMethod>,
        capabilities: Vec<String>,
        info: PluginInfo,
    ) -> Result<Self, CallError> {
        Self::from_component_bytes_with_egress(bytes, interface, methods, capabilities, None, info)
    }

    /// Like [`Self::from_component_bytes`] but with an embedder [`EgressPolicy`]
    /// (FIDIUS-I-0027). `wasi:http` outbound egress is linked only when the
    /// package declares the `http` capability **and** `egress` is `Some`.
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

    /// Build from a precompiled `.cwasm` (engine/version-specific). ~83 µs load
    /// per the spike vs ~6.6 ms JIT.
    ///
    /// # Safety
    /// The bytes must have been produced by `Engine::precompile_component` with a
    /// compatible engine; wasmtime validates the header and refuses a mismatch.
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

    /// Shared constructor: wire WASI into a `Linker` and pre-instantiate the
    /// component **once**. The resulting `InstancePre` is reused for every call.
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

    /// Instantiate a fresh sandboxed `Store` + component instance from the cached
    /// `InstancePre`. Per-call instantiation gives isolation; the linking cost is
    /// already paid in `build` (FIDIUS-I-0024).
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

    /// Resolve an exported function within the plugin's interface by name.
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

    /// Call the `fidius-interface-hash` export — the integrity check the loader
    /// (T-0103) runs against the expected interface hash.
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
}

impl PluginExecutor for WasmComponentExecutor {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn method_count(&self) -> u32 {
        self.methods.len() as u32
    }

    fn call_raw(&self, method: usize, input: &[u8]) -> Result<Vec<u8>, CallError> {
        let m = self.method(method, true)?.clone();
        let (mut store, instance) = self.instantiate()?;
        let func = self.func(&mut store, &instance, &m.name)?;
        // `#[wire(raw)]` is always `list<u8> -> list<u8>`. Use the *typed* call so
        // wasmtime lowers/lifts the bytes as a bulk memcpy instead of building a
        // `Val::List` of one `Val::U8` per byte (the dynamic path turned a 256 KiB
        // payload into milliseconds — FIDIUS-I-0024).
        let typed =
            func.typed::<(Vec<u8>,), (Vec<u8>,)>(&store)
                .map_err(|e| CallError::Backend {
                    runtime: "wasm".into(),
                    message: format!("raw method '{}' is not list<u8> -> list<u8>: {e}", m.name),
                })?;
        let (out,) = typed
            .call(&mut store, (input.to_vec(),))
            .map_err(|e| CallError::Backend {
                runtime: "wasm".into(),
                message: e.to_string(),
            })?;
        typed
            .post_return(&mut store)
            .map_err(|e| CallError::Backend {
                runtime: "wasm".into(),
                message: e.to_string(),
            })?;
        Ok(out)
    }
}

impl ValueExecutor for WasmComponentExecutor {
    fn call(&self, method: usize, args: Value) -> Result<Value, CallError> {
        let m = self.method(method, false)?.clone();
        let (mut store, instance) = self.instantiate()?;
        let func = self.func(&mut store, &instance, &m.name)?;

        // The host tuple-packs args into a `Value::List` of positional args.
        let params: Vec<Val> = match args {
            Value::List(items) => items.iter().map(value_to_val).collect::<Result<_, _>>()?,
            // Unit / no args.
            Value::Unit => Vec::new(),
            // Single non-list arg — treat as one positional param.
            single => vec![value_to_val(&single)?],
        };

        let mut out = [Val::Bool(false)];
        func.call(&mut store, &params, &mut out)
            .map_err(|e| CallError::Backend {
                runtime: "wasm".into(),
                message: e.to_string(),
            })?;

        // A `result<_, plugin-error>` err arm becomes CallError::Plugin.
        if let Val::Result(Err(payload)) = &out[0] {
            return Err(plugin_error_from_val(payload.as_deref()));
        }
        let ret = match &out[0] {
            Val::Result(Ok(inner)) => inner.as_deref().map(val_to_value).unwrap_or(Value::Unit),
            other => val_to_value(other),
        };
        Ok(ret)
    }
}

/// Bounded channel depth between the wasmtime pump thread and the async
/// consumer — the backpressure/memory window (REQ-003/NFR-003), like the Python
/// backend's.
#[cfg(feature = "streaming")]
const STREAM_CHANNEL_CAP: usize = 4;

#[cfg(feature = "streaming")]
#[async_trait::async_trait]
impl crate::stream::StreamExecutor for WasmComponentExecutor {
    async fn call_streaming(
        &self,
        method: usize,
        args: Value,
    ) -> Result<crate::stream::ChunkStream, CallError> {
        let m = self.method(method, false)?.clone();
        if !m.streaming {
            return Err(CallError::Backend {
                runtime: "wasm".into(),
                message: format!("method '{}' is not a server-streaming method", m.name),
            });
        }

        let (mut store, instance) = self.instantiate()?;
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
}

/// Map a `result::err` payload (expected: a record with `code`/`message`/
/// `details`) into a `PluginError`.
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

/// fidius `Value` → wasmtime `Val`. Mirrors the Phase-1 serde bridge shapes.
/// Rust identifier (snake_case / PascalCase) → kebab-case, matching the WIT
/// naming the generator uses. `y_pos`→`y-pos`, `Circle`→`circle`.
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

/// kebab-case → snake_case (WIT record field → serde struct field).
fn kebab_to_snake(s: &str) -> String {
    s.replace('-', "_")
}

/// kebab-case → PascalCase (WIT variant case → serde enum variant).
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

/// wasmtime `Val` → fidius `Value` (structural; self-describing).
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

// ── Pack-time helpers (FIDIUS-T-0107) ───────────────────────────────────────
// Used by `fidius pack` to validate and (optionally) precompile a component
// without constructing a full executor (pack has no descriptor/method list).

/// Validate that `bytes` is a well-formed WASM **component** (Component Model),
/// not a core module or a corrupt artifact. This is the pack-time gate;
/// interface-name + `fidius-interface-hash` conformance is enforced at load
/// (`PluginHost::load_wasm`).
pub fn validate_component(bytes: &[u8]) -> Result<(), CallError> {
    let engine = Engine::default();
    Component::new(&engine, bytes)
        .map(|_| ())
        .map_err(|e| CallError::Backend {
            runtime: "wasm".into(),
            message: format!("not a valid WASM component: {e}"),
        })
}

/// Ahead-of-time compile a component into engine/version-specific `.cwasm`
/// bytes (`Engine::precompile_component`). Written into the package at pack time
/// and consumed by the AOT load path; a stale `.cwasm` is ignored at load (JIT
/// fallback), so this is purely a load-latency optimization.
pub fn precompile_component(bytes: &[u8]) -> Result<Vec<u8>, CallError> {
    let engine = Engine::default();
    engine
        .precompile_component(bytes)
        .map_err(|e| CallError::Backend {
            runtime: "wasm".into(),
            message: format!("failed to precompile component: {e}"),
        })
}
