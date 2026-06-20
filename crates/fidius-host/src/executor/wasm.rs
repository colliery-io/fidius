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

use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::Arc;

use fidius_core::Value;
use wasmtime::component::{Component, InstancePre, Linker, Val};
use wasmtime::{Engine, Store};
use wasmtime_wasi::p2::add_to_linker_sync;
use wasmtime_wasi::{
    DirPerms, FilePerms, ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView,
};
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
    /// Client-streaming producer (FIDIUS-I-0030 CS2.3): the host sets this before
    /// a client-streaming call; the guest's `fidius:stream-pull/pull.next` import
    /// pulls bincode items from it. `None` outside such a call.
    client_stream: Option<Box<dyn Iterator<Item = Vec<u8>> + Send>>,
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

/// Capabilities the host knows how to grant. Filesystem is absent here because
/// it is grantable ONLY in the scoped form `fs:ro:<path>` / `fs:rw:<path>`
/// (FIDIUS-A-0008) — a path-scoped preopen, never the whole filesystem; handled
/// in `validate_capabilities`/`build_wasi_ctx`. `clocks`/`random` are always
/// available in WASI and are accepted as no-ops so manifests can declare intent
/// without error.
const KNOWN_CAPABILITIES: &[&str] = &[
    "args", "stdout", "stderr", "stdin", "network", "sockets", "clocks", "random",
    // FIDIUS-I-0027: declares the guest *wants* brokered outbound HTTP. Actual
    // egress also requires the embedder to supply an `EgressPolicy` (two-key);
    // handled in `build`, not `build_wasi_ctx`.
    "http",
    // NOTE: `env` is intentionally absent — it is grantable ONLY in the scoped
    // form `env:VAR_NAME` (FIDIUS-T-0142). Bare `env` (inherit ALL host env vars,
    // i.e. all secrets) is rejected by `validate_capabilities`.
];

/// Reject unknown capability names early (at load) so a typo fails closed and
/// loud rather than silently granting nothing.
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

/// Build a `WasiCtx` from the allow-list. Starts deny-all (a fresh builder
/// inherits nothing and has no preopens) and grants only what's listed.
/// Filesystem is granted only per `fs:ro:<path>` / `fs:rw:<path>` — a path-scoped
/// preopen, never the whole filesystem (FIDIUS-A-0008).
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
    b.build()
}

/// Baseline SSRF denylist for the raw-socket grant (FIDIUS-T-0143): an address a
/// sandboxed guest must never reach — loopback, link-local (incl. the cloud
/// metadata IP `169.254.169.254`), private (RFC-1918), unique-local, unspecified,
/// or broadcast. This is a safety *floor* (like deny-all), not a full egress
/// policy; per-host policy is the embedder's job via the `http` capability.
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

/// The `wasi:http` version this host provides — matched to `wasmtime-wasi-http`
/// and the vendored guest WIT (FIDIUS-A-0005). Bump together with a wasmtime
/// upgrade; the `fidius-guest` pin tripwire + the macro-egress E2E guard the match.
const HOST_WASI_HTTP: (u32, u32, u32) = (0, 2, 6);

/// Scan a component's import names for a `wasi:http` version this host can't
/// satisfy, returning a clear, actionable message if so (FIDIUS-A-0005, fail
/// loud — the same discipline as the `ABI_VERSION` check, on a new axis).
///
/// Compatible iff the import is on the host's `major.minor` line and the host's
/// patch is `>=` the plugin's (WASI 0.2 is forward-compatible: a newer host
/// satisfies an older import, never the reverse). A host *behind* the plugin, or
/// a different line (`0.2`→`0.3`), is rejected up front instead of surfacing as a
/// cryptic instantiate trap. Pulled out as a free fn so it unit-tests without a
/// real component.
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
    /// FIDIUS-A-0006 / CI.3: when configured, the instance lives in a *persistent*
    /// store (config bound once via the `fidius-configure` export); method calls
    /// dispatch on it instead of a fresh per-call store. `None` = zero-config
    /// (per-call instantiation, the isolation default).
    configured: Option<std::sync::Mutex<ConfiguredStore>>,
    /// The config bytes (FIDIUS-A-0006 / CI.3), retained so a *streaming* call can
    /// `fidius-configure` the store it owns for the stream's lifetime (a stream
    /// takes its store by value, so it can't share the unary persistent store — it
    /// just needs the same config set in its own memory first).
    config_bytes: Option<Vec<u8>>,
}

/// A configured instance's persistent store + instance (FIDIUS-A-0006 / CI.3).
struct ConfiguredStore {
    store: Store<HostState>,
    instance: wasmtime::component::Instance,
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
        Self::from_cwasm_with_egress(cwasm, interface, methods, capabilities, None, info)
    }

    /// Like [`Self::from_cwasm`] but with an embedder [`EgressPolicy`]
    /// (FIDIUS-I-0027) — the AOT counterpart of
    /// [`Self::from_component_bytes_with_egress`].
    ///
    /// # Safety
    /// Same as [`Self::from_cwasm`].
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
        })
    }

    /// Bind config once (FIDIUS-A-0006 / CI.3): instantiate a *persistent* store,
    /// call the guest's `fidius-configure` export with `cfg`, and retain the store
    /// so subsequent method calls dispatch on the configured instance. `cfg` is
    /// the bincode of the plugin's config type (empty = the zero-config no-op).
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
        typed
            .post_return(&mut store)
            .map_err(|e| CallError::Backend {
                runtime: "wasm".into(),
                message: e.to_string(),
            })?;
        self.configured = Some(std::sync::Mutex::new(ConfiguredStore { store, instance }));
        self.config_bytes = Some(cfg.to_vec());
        Ok(())
    }

    /// Client-streaming (FIDIUS-I-0030 CS2.3): call a method whose `Stream<T>`
    /// argument is fed by the host. `producer` is the bincode-encoded items the
    /// guest pulls via the `fidius:stream-pull` import; `args` are the non-stream
    /// args (tuple-packed into a `Value`); returns the method's result as a `Value`.
    #[cfg(feature = "streaming")]
    pub fn call_client_streaming(
        &self,
        method: usize,
        producer: Vec<Vec<u8>>,
        args: Value,
    ) -> Result<Value, CallError> {
        let m = self.method(method, false)?.clone();
        self.with_store(|store, instance| {
            store.data_mut().client_stream = Some(Box::new(producer.into_iter()));
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

    /// Bidirectional streaming (FIDIUS-I-0032 / ADR-0010): the host produces `producer`
    /// (the plugin's `Stream<In>` argument, pulled via the `fidius:stream-pull` import)
    /// and consumes the plugin's `Stream<Out>` output resource as a `ChunkStream`. Pulling
    /// the output drives the plugin, which pulls input on demand. `args` are the
    /// non-stream args (as a `Value`).
    #[cfg(feature = "streaming")]
    pub async fn call_bidi_streaming(
        &self,
        method: usize,
        producer: Vec<Vec<u8>>,
        args: Value,
    ) -> Result<crate::stream::ChunkStream, CallError> {
        self.stream_with_producer(method, args, Some(producer))
            .await
    }

    /// Run `f` with a `(store, instance)`: the persistent configured store if
    /// configured (FIDIUS-A-0006 / CI.3), else a fresh per-call one (isolation).
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
            client_stream: None,
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
        self.with_store(|store, instance| {
            let func = self.func(store, instance, &m.name)?;
            // `#[wire(raw)]` is always `list<u8> -> list<u8>`. Use the *typed* call
            // so wasmtime lowers/lifts the bytes as a bulk memcpy instead of a
            // `Val::List` of one `Val::U8` per byte (FIDIUS-I-0024).
            let typed =
                func.typed::<(Vec<u8>,), (Vec<u8>,)>(&*store)
                    .map_err(|e| CallError::Backend {
                        runtime: "wasm".into(),
                        message: format!(
                            "raw method '{}' is not list<u8> -> list<u8>: {e}",
                            m.name
                        ),
                    })?;
            let (out,) =
                typed
                    .call(&mut *store, (input.to_vec(),))
                    .map_err(|e| CallError::Backend {
                        runtime: "wasm".into(),
                        message: e.to_string(),
                    })?;
            typed
                .post_return(&mut *store)
                .map_err(|e| CallError::Backend {
                    runtime: "wasm".into(),
                    message: e.to_string(),
                })?;
            Ok(out)
        })
    }
}

impl ValueExecutor for WasmComponentExecutor {
    fn call(&self, method: usize, args: Value) -> Result<Value, CallError> {
        let m = self.method(method, false)?.clone();

        self.with_store(|store, instance| {
            let func = self.func(store, instance, &m.name)?;
            // Type-directed lowering: the WIT param types disambiguate a tuple from a
            // list (PC.1). The host tuple-packs args into a `Value::List` of positionals.
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
                            "plugin method takes no parameters but an argument was supplied".into(),
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

            // A `result<_, plugin-error>` err arm becomes CallError::Plugin.
            if let Val::Result(Err(payload)) = &out[0] {
                return Err(plugin_error_from_val(payload.as_deref()));
            }
            let ret = match &out[0] {
                Val::Result(Ok(inner)) => inner.as_deref().map(val_to_value).unwrap_or(Value::Unit),
                other => val_to_value(other),
            };
            Ok(ret)
        })
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
        self.stream_with_producer(method, args, None).await
    }
}

impl WasmComponentExecutor {
    /// Shared server-streaming / bidirectional output pump. `producer = Some(items)`
    /// sets the client-streaming **input** producer in the (pump-owned) store before the
    /// export call, so the output resource's `next()` re-enters the
    /// `fidius:stream-pull` import on demand — the bidirectional synchronous lazy-pull
    /// composition (FIDIUS-I-0032 / ADR-0010). `None` = plain server-streaming (WS).
    #[cfg(feature = "streaming")]
    async fn stream_with_producer(
        &self,
        method: usize,
        args: Value,
        producer: Option<Vec<Vec<u8>>>,
    ) -> Result<crate::stream::ChunkStream, CallError> {
        let m = self.method(method, false)?.clone();
        if !m.streaming {
            return Err(CallError::Backend {
                runtime: "wasm".into(),
                message: format!("method '{}' is not a server-streaming method", m.name),
            });
        }
        let (mut store, instance) = self.instantiate()?;
        // Bidirectional: seed the input producer the output resource pulls through. The
        // store moves to the pump thread, so the producer lives for the whole stream.
        if let Some(items) = producer {
            store.data_mut().client_stream = Some(Box::new(items.into_iter()));
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
            typed
                .post_return(&mut store)
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

/// Type-directed lowering for the **argument** path. The structural [`value_to_val`]
/// can't tell a Rust tuple (a `Value::List`) from a real list, so when the target WIT
/// type is a `tuple<…>` we use the wasmtime [`Type`] to emit `Val::Tuple`. Lists,
/// options, and maps recurse with their element type so nested tuples are caught;
/// everything else falls back to the structural lowering.
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
        // Records, primitives, variants, results: structural lowering is unambiguous.
        _ => value_to_val(v),
    }
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

#[cfg(test)]
mod ssrf_tests {
    use super::is_blocked_ip;
    use std::net::IpAddr;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn blocks_internal_and_metadata_targets() {
        // Cloud metadata, loopback, link-local, RFC-1918, ULA, unspecified.
        for s in [
            "169.254.169.254", // AWS/GCP/Azure metadata (link-local)
            "127.0.0.1",
            "::1",
            "10.1.2.3",
            "172.16.5.5",
            "192.168.1.1",
            "0.0.0.0",
            "fe80::1",          // link-local v6
            "fc00::1",          // unique-local v6
            "::ffff:127.0.0.1", // v4-mapped loopback
        ] {
            assert!(is_blocked_ip(&ip(s)), "{s} must be blocked");
        }
    }

    #[test]
    fn allows_public_targets() {
        for s in [
            "1.1.1.1",
            "93.184.216.34",
            "8.8.8.8",
            "2606:4700:4700::1111",
        ] {
            assert!(!is_blocked_ip(&ip(s)), "{s} must be allowed");
        }
    }
}

#[cfg(test)]
mod fs_capability_tests {
    use super::*;

    fn msg(r: Result<(), CallError>) -> String {
        match r {
            Err(CallError::Backend { message, .. }) => message,
            other => panic!("expected Backend error, got {other:?}"),
        }
    }

    #[test]
    fn path_scoped_fs_grants_are_accepted() {
        assert!(validate_capabilities(&["fs:ro:/data".into()]).is_ok());
        assert!(validate_capabilities(&["fs:rw:/var/out".into()]).is_ok());
        // Composes with other caps.
        assert!(validate_capabilities(&["stdout".into(), "fs:rw:/tmp/x".into()]).is_ok());
    }

    #[test]
    fn bare_filesystem_is_rejected() {
        // Whole-FS grants are a footgun — must fail loud, like bare `env`.
        assert!(msg(validate_capabilities(&["fs".into()])).contains("path-scoped"));
        assert!(msg(validate_capabilities(&["filesystem".into()])).contains("path-scoped"));
    }

    #[test]
    fn fs_grant_without_a_path_is_rejected() {
        assert!(msg(validate_capabilities(&["fs:ro:".into()])).contains("requires a path"));
        assert!(msg(validate_capabilities(&["fs:rw:".into()])).contains("requires a path"));
    }

    #[test]
    fn build_wasi_ctx_with_an_fs_grant_does_not_panic() {
        // A read-write preopen of a real temp dir builds a ctx (the guest would
        // then see exactly that dir).
        let tmp = tempfile::TempDir::new().unwrap();
        let cap = format!("fs:rw:{}", tmp.path().display());
        let _ctx = build_wasi_ctx(&[cap]);
    }
}

#[cfg(test)]
mod wasi_http_version_tests {
    use super::*;

    #[test]
    fn host_matched_version_is_compatible() {
        // 0.2.6 (the pin) and any older patch on the same line load fine.
        assert!(wasi_http_incompatibility(["wasi:http/types@0.2.6"].into_iter()).is_none());
        assert!(
            wasi_http_incompatibility(["wasi:http/outgoing-handler@0.2.0"].into_iter()).is_none()
        );
    }

    #[test]
    fn newer_minor_or_patch_is_rejected_with_a_clear_message() {
        // Patch ahead of the host (the exact `wasi` crate 0.2.12 skew that broke
        // the fetcher) — and a different line — must fail loud, naming versions.
        for bad in ["wasi:http/types@0.2.12", "wasi:http/types@0.3.0"] {
            let msg = wasi_http_incompatibility([bad].into_iter())
                .unwrap_or_else(|| panic!("{bad} should be rejected"));
            assert!(msg.contains("plugin requires wasi:http"), "{msg}");
            assert!(
                msg.contains("0.2.6"),
                "message names the host version: {msg}"
            );
        }
    }

    #[test]
    fn no_wasi_http_import_is_fine() {
        // A plugin that never imports wasi:http isn't gated on it.
        assert!(wasi_http_incompatibility(
            ["wasi:cli/environment@0.2.6", "wasi:io/streams@0.2.6"].into_iter()
        )
        .is_none());
    }
}
