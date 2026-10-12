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
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;

use fidius_core::Value;
use wasmtime::component::{Component, InstancePre, Linker, Val};
use wasmtime::{Engine, Store};
use wasmtime_wasi::p2::add_to_linker_sync;
use wasmtime_wasi::p2::bindings::sockets::ip_name_lookup;
use wasmtime_wasi::sockets::SocketAddrUse;
use wasmtime_wasi::{
    DirPerms, FilePerms, ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView,
};
use wasmtime_wasi_http::p2::bindings::http::types::ErrorCode;
use wasmtime_wasi_http::p2::body::HyperOutgoingBody;
use wasmtime_wasi_http::p2::types::{
    HostFutureIncomingResponse, IncomingResponse, OutgoingRequestConfig,
};
use wasmtime_wasi_http::p2::{
    add_only_http_to_linker_sync, default_send_request, default_send_request_handler, HttpResult,
    WasiHttpCtxView, WasiHttpHooks, WasiHttpView,
};
use wasmtime_wasi_http::WasiHttpCtx;

use crate::error::CallError;
use crate::executor::body_tee::{replay_body, CaptureHandle, TeeBody};
use crate::executor::name_lookup::{
    default_resolver, FidiusNameLookup, NameLookupView, PinTable, Resolver,
};
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
/// The target of an outbound TCP connect, as the guest expressed it
/// (FIDIUS-I-0034). Handed to [`EgressPolicy::authorize_tcp_target`].
///
/// Plain public fields on purpose: an embedder constructs these literally in
/// policy tests. Additional context (e.g. exhaustive name candidates for an
/// IP) would land as new fields in a breaking rev, not `#[non_exhaustive]`.
/// What an [`EgressPolicy`] wants done with a response it has observed via
/// [`on_response`](EgressPolicy::on_response) (FIDIUS-I-0035).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseDirective {
    /// Hand the response to the guest unchanged (the default).
    Forward,
    /// Discard this response; re-run [`authorize`](EgressPolicy::authorize) on
    /// a fresh clone of the **original** (pre-`authorize`) request parts —
    /// letting the policy inject a fresh credential — and dispatch again. The
    /// second response is forwarded to the guest unconditionally.
    ///
    /// Bounded by fidius to **at most one retry per original guest request**;
    /// a `RetryOnce` returned when `retry_available` is `false` (second
    /// observation, or a non-replayable body) is ignored and the response
    /// forwards. Each dispatch attempt gets its own connect/first-byte
    /// timeouts, so a retried request can take up to ~2× the configured
    /// budget. If the re-run `authorize` denies, the guest sees the same
    /// generic HTTP "request denied" as any refused request — the policy
    /// consumed the original response and then refused to re-stamp.
    RetryOnce,
}

pub struct TcpTarget<'a> {
    /// The hostname the guest dialed, when it dialed by name and fidius could
    /// pin the lookup (lowercased; DNS is case-insensitive). `None` = the
    /// guest dialed an IP literal (no lookup happened), or no pin was
    /// available for the resolved IP. A name-keyed policy should deny `None`
    /// — that is the honest default for an allow-list that speaks names.
    pub host: Option<&'a str>,
    /// The resolved peer the connect will actually reach (the same value
    /// [`EgressPolicy::authorize_tcp`] has always received).
    pub addr: SocketAddr,
}

pub trait EgressPolicy: Send + Sync + 'static {
    /// Authorize (and optionally decorate) one outbound request before dispatch.
    fn authorize(&self, parts: &mut http::request::Parts) -> Result<(), EgressDenied>;

    /// Authorize one outbound **TCP** connection before `connect` (FIDIUS-I-0033).
    /// The second key of the same two-key gate as [`authorize`](Self::authorize):
    /// the guest's package must declare the `tcp` capability **and** the host must
    /// supply a policy whose `authorize_tcp` returns `Ok` for the target.
    ///
    /// This is the seam a database/warehouse connector reaches the wire through —
    /// a pure-Rust sync driver over `std::net::TcpStream` (which on
    /// `wasm32-wasip2` is `wasi:sockets`) becomes reachable only for the
    /// `host:port`s this method allows.
    ///
    /// `addr` is the **resolved** peer (`IP:port`). A hostname the guest dialed
    /// (`std::net::TcpStream::connect(("db.internal", 5432))`) is resolved via
    /// `wasi:sockets` name-lookup *first*; fidius pins that lookup and hands the
    /// dialed name to [`authorize_tcp_target`](Self::authorize_tcp_target)
    /// (FIDIUS-I-0034), so a policy that cares about names overrides that method
    /// instead of re-deriving resolve-and-pin itself. Only `TcpConnect` reaches
    /// here; bind/listen and UDP are denied outright.
    ///
    /// **Defaults to deny.** An existing HTTP-only [`EgressPolicy`] therefore
    /// never silently grants raw TCP; an embedder opts in by overriding this.
    fn authorize_tcp(&self, _addr: &SocketAddr) -> Result<(), EgressDenied> {
        Err(EgressDenied::new("tcp egress not permitted by this policy"))
    }

    /// Name-aware TCP authorization (FIDIUS-I-0034): like
    /// [`authorize_tcp`](Self::authorize_tcp), but the target carries the
    /// hostname the guest actually dialed, recovered from fidius's
    /// resolve-and-pin of the guest's `wasi:sockets` name lookups. This is the
    /// method a hostname allow-list implements — IP allow-lists are
    /// operationally broken for managed endpoints that rotate IPs.
    ///
    /// `target.host` is `Some(name)` only when the guest dialed by name and
    /// this instance's lookup was pinned; an IP-literal dial arrives as
    /// `None` (deny it if your policy speaks names). The pin narrows
    /// lookup→connect TOCTOU to "an address this instance was actually given
    /// for that name" — it cannot eliminate it.
    ///
    /// **Default delegates to `authorize_tcp(&target.addr)`**, so an embedder
    /// overriding only that method observes byte-identical behavior.
    fn authorize_tcp_target(&self, target: &TcpTarget<'_>) -> Result<(), EgressDenied> {
        self.authorize_tcp(&target.addr)
    }

    /// Authorize one guest DNS lookup, **before** resolution (FIDIUS-I-0034).
    /// Without this hook a guest granted the `tcp`/`udp` tier can probe
    /// arbitrary DNS even when every connect would be denied; a denial here
    /// fails the guest's lookup (it sees the same resolver failure as a
    /// lookup denied outright), resolves nothing, and pins nothing.
    ///
    /// **Defaults to allow** — deliberately the opposite polarity of
    /// [`authorize_tcp`](Self::authorize_tcp)'s default-deny: name lookup is
    /// already open whenever the tcp/udp tier is granted, so a deny default
    /// would break every existing embedder's hostname dials. The connect
    /// itself is still gated by
    /// [`authorize_tcp_target`](Self::authorize_tcp_target).
    fn authorize_dns(&self, _name: &str) -> Result<(), EgressDenied> {
        Ok(())
    }

    /// Authorize one outbound **UDP** datagram before it leaves (FIDIUS-I-0033) —
    /// the symmetric counterpart of [`authorize_tcp`](Self::authorize_tcp). The
    /// second key of the same two-key gate: the guest's package must declare the
    /// `udp` capability **and** the host must supply a policy whose `authorize_udp`
    /// returns `Ok` for the target.
    ///
    /// `addr` is the **resolved** remote peer (`IP:port`) of a `connect` or a
    /// one-shot `send_to` — name resolution happens *first*, exactly as for TCP, so
    /// the policy sees the IP the datagram will actually reach (resolve-and-pin
    /// closes DNS-rebinding if the embedder cares). Binding the local source socket
    /// is permitted as setup and never reaches this method; inbound and TCP uses
    /// are denied outright.
    ///
    /// **Defaults to deny.** An existing policy never silently gains UDP reach; an
    /// embedder opts in by overriding this.
    fn authorize_udp(&self, _addr: &SocketAddr) -> Result<(), EgressDenied> {
        Err(EgressDenied::new("udp egress not permitted by this policy"))
    }

    /// Opt-in gate for [`on_response`](Self::on_response) (FIDIUS-I-0035).
    ///
    /// **Defaults to `false`**, and while it stays `false` the dispatch path
    /// is byte-identical to a fidius without the response hook: no body tee,
    /// no observation, zero overhead. A policy that overrides `on_response`
    /// must also override this to return `true`, accepting the (small) cost
    /// of teeing every outgoing request body up to 64 KiB for possible replay.
    fn observes_responses(&self) -> bool {
        false
    }

    /// Observe the response HEAD for a request this policy authorized,
    /// **before** the guest sees it (FIDIUS-I-0035). Called only when
    /// [`observes_responses`](Self::observes_responses) returns `true`.
    ///
    /// This is the auth-retry seam: a policy that injects credentials in
    /// [`authorize`](Self::authorize) can match an expired-credential response
    /// here (typically a 401), invalidate its cache, and return
    /// [`ResponseDirective::RetryOnce`] — the request is re-authorized (fresh
    /// credential) and dispatched once more, invisibly to the guest.
    ///
    /// - `request` is the **as-dispatched** (post-`authorize`) parts — it
    ///   carries whatever this policy injected. The retry's `authorize` runs
    ///   on a clean pre-`authorize` clone instead, so a stale injected header
    ///   never feeds back into re-stamping.
    /// - Head-only: status + headers, never the response body.
    /// - `retry_available` is `false` when the request body was not replayable
    ///   (streamed past 64 KiB, carried trailers, or never finished before the
    ///   response arrived) and on the observation of a retried dispatch — a
    ///   `RetryOnce` returned then is ignored and the response forwards.
    /// - Transport errors never reach this method; the guest sees them as
    ///   today.
    ///
    /// **Defaults to [`ResponseDirective::Forward`]** — existing policies are
    /// unaffected.
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

        if !policy.observes_responses() {
            // The pre-FIDIUS-I-0035 path, kept literally: no tee, no
            // observation. Byte-identical for every policy that doesn't opt
            // into the response hook.
            if policy.authorize(&mut parts).is_err() {
                return Err(ErrorCode::HttpRequestDenied.into());
            }
            return Ok(default_send_request(
                http::Request::from_parts(parts, body),
                config,
            ));
        }

        // Observing path (FIDIUS-I-0035). The clone MUST be taken before
        // `authorize` runs: a retry re-stamps this clean copy, never a
        // request already carrying the policy's own (stale) injected headers.
        let original = parts.clone();
        if policy.authorize(&mut parts).is_err() {
            return Err(ErrorCode::HttpRequestDenied.into());
        }
        let (teed, capture) = TeeBody::wrap(body);
        let policy = Arc::clone(policy);
        // Mirror `default_send_request`: spawn the dispatch and hand back the
        // pending future the guest polls.
        let handle = wasmtime_wasi::runtime::spawn(async move {
            Ok(dispatch_observed(policy, original, parts, teed, capture, config).await)
        });
        Ok(HostFutureIncomingResponse::pending(handle))
    }
}

/// Copy an [`OutgoingRequestConfig`] (all-`Copy` fields; the type itself isn't
/// `Clone` upstream). The retry attempt reuses the same budget, so connect and
/// first-byte timeouts apply **per attempt**.
fn copy_config(c: &OutgoingRequestConfig) -> OutgoingRequestConfig {
    OutgoingRequestConfig {
        use_tls: c.use_tls,
        connect_timeout: c.connect_timeout,
        first_byte_timeout: c.first_byte_timeout,
        between_bytes_timeout: c.between_bytes_timeout,
    }
}

/// The observing dispatch (FIDIUS-I-0035): send the request, show the policy
/// the response head, and honor at most one `RetryOnce` — structurally, a
/// straight-line function with a single possible second dispatch; no loop
/// exists for a policy to drive.
///
/// `original` is the pre-`authorize` parts; `dispatched` the as-sent parts.
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
    /// Host-function tables bound to this executor (plugin → host callback
    /// channel, wasm variant). Shared with the executor so tables bound after
    /// instantiation (including after `configure`'s persistent store was
    /// created) are visible to the `fidius:host-call` import.
    host_tables: HostTables,
    /// Resolve-and-pin table (FIDIUS-I-0034): what this store's name lookups
    /// resolved to, written by the shadowed `ip-name-lookup` and read by the
    /// same store's `socket_addr_check` (which holds a clone of the `Arc`).
    pins: PinTable,
    /// Host-side resolution function for the shadowed lookup. The executor's
    /// default matches upstream (std `ToSocketAddrs`); tests may inject one
    /// via `WasmComponentExecutor::set_resolver`.
    resolver: Resolver,
}

/// Accessor handed to `ip_name_lookup::add_to_linker` for the shadowed
/// instance: project the store's state into the lookup view (FIDIUS-I-0034).
/// The policy rides in via the (always-present) `EgressHooks`.
fn name_lookup_view(state: &mut HostState) -> NameLookupView<'_> {
    NameLookupView {
        table: &mut state.table,
        pins: &state.pins,
        policy: state.hooks.policy.as_ref(),
        resolver: &state.resolver,
    }
}

/// The executor-wide registry of bound host-function tables, keyed by
/// interface name. Shared (`Arc`) into every store's `HostState`.
type HostTables = Arc<std::sync::RwLock<std::collections::HashMap<String, HostTableRef>>>;

/// A `Send + Sync` wrapper for a bound, process-lifetime
/// [`HostFunctionTable`] pointer (same justification as the loader's static
/// table pointers: the generated binding leaks the table it builds).
#[derive(Clone, Copy)]
struct HostTableRef(*const fidius_core::host_ffi::HostFunctionTable);

// SAFETY: the pointed-to table has process lifetime per the bind contract and
// is immutable after construction; dispatch/free_buffer are thread-safe entry
// points (the host implementation behind them is required to be Send + Sync).
unsafe impl Send for HostTableRef {}
unsafe impl Sync for HostTableRef {}

/// Run one host-function dispatch through a bound table and return the raw
/// `(status, payload)` pair for the guest, copying the host-owned output
/// buffer and releasing it via the table's `free_buffer`.
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
    // FIDIUS-I-0033: declares the guest *wants* policy-gated outbound TCP
    // (`std::net::TcpStream` → `wasi:sockets`, for raw-wire DB/warehouse drivers).
    // Like `http` it is the first of a two-key gate: actual reachability needs the
    // embedder's `EgressPolicy::authorize_tcp` to allow the resolved host:port —
    // handled in `build_wasi_ctx`. Distinct from the coarse `network`/`sockets`
    // grant, which is per-IP SSRF-floored but has no per-target embedder policy.
    "tcp",
    // FIDIUS-I-0033: the UDP counterpart of `tcp` — policy-gated outbound UDP via
    // `EgressPolicy::authorize_udp`. Same two-key gate; composes with `tcp` (one
    // dispatching socket check) and is mutually exclusive with `network`/`sockets`.
    "udp",
    // NOTE: `env` is intentionally absent — it is grantable ONLY in the scoped
    // form `env:VAR_NAME` (FIDIUS-T-0142). Bare `env` (inherit ALL host env vars,
    // i.e. all secrets) is rejected by `validate_capabilities`.
];

/// Reject unknown capability names early (at load) so a typo fails closed and
/// loud rather than silently granting nothing.
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

/// Build a `WasiCtx` from the allow-list. Starts deny-all (a fresh builder
/// inherits nothing and has no preopens) and grants only what's listed.
/// Filesystem is granted only per `fs:ro:<path>` / `fs:rw:<path>` — a path-scoped
/// preopen, never the whole filesystem (FIDIUS-A-0008).
///
/// `pins` is the same table the store's shadowed `ip-name-lookup` writes
/// (FIDIUS-I-0034) — the `socket_addr_check` installed here reads it to
/// recover the hostname the guest dialed for a connect's IP.
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

/// The `wasi:http` version this host provides — what `wasmtime-wasi-http`
/// registers (FIDIUS-A-0005). WASI 0.2 is forward-compatible, so the host is the
/// **ceiling**: it satisfies any guest on the same `0.2` line up to this patch
/// (`guest_patch <= host_patch`, enforced by [`wasi_http_incompatibility`]).
/// wasmtime-wasi-http 46 registers `wasi:http@0.2.12`, which covers both the
/// `fidius-guest` vendored pin (0.2.6) and the higher versions a newer stable
/// `wasm32-wasip2` toolchain emits (e.g. 0.2.9 on CI). Bump this to the value
/// wasmtime-wasi-http provides whenever wasmtime is upgraded.
const HOST_WASI_HTTP: (u32, u32, u32) = (0, 2, 12);

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
    /// Host-function tables bound to this executor (plugin → host callback
    /// channel, wasm variant), keyed by interface name. Populated by
    /// [`Self::bind_host_table`]; read by the `fidius:host-call` import.
    host_tables: HostTables,
    /// Host-side resolver behind the shadowed `ip-name-lookup`
    /// (FIDIUS-I-0034). Defaults to std `ToSocketAddrs` (upstream parity);
    /// [`Self::set_resolver`] injects one for deterministic tests.
    resolver: Resolver,
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

    /// Replace the host-side resolver behind the shadowed `ip-name-lookup`
    /// (FIDIUS-I-0034). Test seam — lets the e2e suite model multi-name/
    /// same-IP and rotation without real DNS. Not part of the stable API.
    /// Takes effect for stores created after the call (per-call stores and
    /// a subsequent `configure`'s persistent store).
    #[doc(hidden)]
    pub fn set_resolver(
        &mut self,
        resolver: Arc<dyn Fn(&str) -> std::io::Result<Vec<IpAddr>> + Send + Sync>,
    ) {
        self.resolver = resolver;
    }

    /// Bind a host-function table (plugin → host callback channel) to this
    /// executor. Its identity fields are read here and gated against the
    /// guest's expectation on every `fidius:host-call` dispatch. Once-only
    /// per interface: a second bind fails rather than swapping the table
    /// under in-flight calls.
    ///
    /// # Safety
    /// `table` must be null or a valid, **process-lifetime**
    /// [`HostFunctionTable`](fidius_core::host_ffi::HostFunctionTable) —
    /// e.g. the leaked table a `#[host_interface]`-generated
    /// `<Trait>Binding::table` builds. The executor retains the pointer and
    /// dispatches through it for its remaining lifetime.
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

    /// Bidirectional streaming (FIDIUS-I-0032 / ADR-0010): the host produces `producer`
    /// (the plugin's `Stream<In>` argument, pulled via the `fidius:stream-pull` import)
    /// and consumes the plugin's `Stream<Out>` output resource as a `ChunkStream`. Pulling
    /// the output drives the plugin, which pulls input on demand. `args` are the
    /// non-stream args (as a `Value`).
    #[cfg(feature = "streaming")]
    pub async fn call_bidi_streaming(
        &self,
        method: usize,
        producer: Box<dyn Iterator<Item = Vec<u8>> + Send>,
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
        let _ctx = build_wasi_ctx(&[cap], None, PinTable::default());
    }
}

#[cfg(test)]
mod tcp_egress_tests {
    use super::*;

    /// A reference embedder policy: allow TCP to one allow-listed `host:port`
    /// (here keyed on the resolved peer), deny HTTP. Mirrors what the docs say an
    /// embedder writes for a DB connector — fidius ships none of this.
    struct AllowOnePort(u16);
    impl EgressPolicy for AllowOnePort {
        fn authorize(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
            Err(EgressDenied::new("http denied by tcp-only policy"))
        }
        fn authorize_tcp(&self, addr: &SocketAddr) -> Result<(), EgressDenied> {
            if addr.port() == self.0 {
                Ok(())
            } else {
                Err(EgressDenied::new("port not allow-listed"))
            }
        }
    }

    #[test]
    fn tcp_is_a_known_capability() {
        assert!(validate_capabilities(&["tcp".into()]).is_ok());
        assert!(validate_capabilities(&["tcp".into(), "stdout".into()]).is_ok());
    }

    #[test]
    fn udp_is_a_known_capability() {
        assert!(validate_capabilities(&["udp".into()]).is_ok());
        // `tcp` and `udp` are the same policy-gated tier — they compose.
        assert!(validate_capabilities(&["tcp".into(), "udp".into()]).is_ok());
    }

    #[test]
    fn policy_egress_and_network_are_mutually_exclusive() {
        // The policy-gated tier (`tcp`/`udp`) and the coarse `network`/`sockets`
        // tier both install a single (last-wins) `socket_addr_check`; declaring
        // both would silently keep only one gate depending on order. Reject at
        // load, regardless of order or which member of each tier appears.
        for combo in [
            vec!["tcp".to_string(), "network".to_string()],
            vec!["network".to_string(), "tcp".to_string()],
            vec!["tcp".to_string(), "sockets".to_string()],
            vec!["udp".to_string(), "network".to_string()],
            vec!["sockets".to_string(), "udp".to_string()],
            vec![
                "sockets".to_string(),
                "stdout".to_string(),
                "tcp".to_string(),
            ],
        ] {
            assert!(
                validate_capabilities(&combo).is_err(),
                "expected {combo:?} to be rejected as a conflicting grant"
            );
        }
        // Each tier alone (and `tcp`+`udp` together) is still fine.
        assert!(validate_capabilities(&["network".into()]).is_ok());
        assert!(validate_capabilities(&["sockets".into()]).is_ok());
        assert!(validate_capabilities(&["tcp".into(), "udp".into()]).is_ok());
    }

    #[test]
    fn default_authorize_tcp_and_udp_deny() {
        // A policy that only implements the (required) http `authorize` must NOT
        // accidentally grant TCP or UDP — both default to deny (fail-closed).
        struct HttpOnly;
        impl EgressPolicy for HttpOnly {
            fn authorize(&self, _p: &mut http::request::Parts) -> Result<(), EgressDenied> {
                Ok(())
            }
        }
        let addr: SocketAddr = "93.184.216.34:5432".parse().unwrap();
        assert!(HttpOnly.authorize_tcp(&addr).is_err());
        assert!(HttpOnly.authorize_udp(&addr).is_err());
    }

    #[test]
    fn tcp_grant_with_policy_builds_a_ctx() {
        // The two-key happy path: `tcp` cap + a policy installs a socket check
        // without panicking (the guest would then connect only to allow-listed
        // peers). The no-policy path is the fail-closed default (no check → all
        // connects denied), covered by the e2e fixture.
        let policy: Arc<dyn EgressPolicy> = Arc::new(AllowOnePort(5432));
        let _ctx = build_wasi_ctx(&["tcp".into()], Some(policy), PinTable::default());
    }

    #[test]
    fn tcp_grant_without_policy_builds_a_ctx() {
        // `tcp` declared but no policy: no socket check is installed, so the
        // deny-all default stands — building the ctx must still succeed.
        let _ctx = build_wasi_ctx(&["tcp".into()], None, PinTable::default());
    }

    #[test]
    fn udp_and_combined_grants_build_a_ctx() {
        // `udp` alone, and `tcp`+`udp` together (one dispatching check), each build
        // a ctx without panicking — with and without a policy (fail-closed default).
        let policy: Arc<dyn EgressPolicy> = Arc::new(AllowOnePort(5432));
        let _ = build_wasi_ctx(&["udp".into()], Some(policy.clone()), PinTable::default());
        let _ = build_wasi_ctx(&["udp".into()], None, PinTable::default());
        let _ = build_wasi_ctx(
            &["tcp".into(), "udp".into()],
            Some(policy),
            PinTable::default(),
        );
        let _ = build_wasi_ctx(&["tcp".into(), "udp".into()], None, PinTable::default());
    }
}

#[cfg(test)]
mod wasi_http_version_tests {
    use super::*;

    #[test]
    fn host_matched_version_is_compatible() {
        // The host ceiling (0.2.12) and any older patch on the same 0.2 line load
        // fine — including the `fidius-guest` vendored pin (0.2.6) and the higher
        // versions a newer stable `wasm32-wasip2` toolchain emits (e.g. 0.2.9).
        for ok in [
            "wasi:http/types@0.2.12",
            "wasi:http/types@0.2.9",
            "wasi:http/types@0.2.6",
            "wasi:http/outgoing-handler@0.2.0",
        ] {
            assert!(
                wasi_http_incompatibility([ok].into_iter()).is_none(),
                "{ok} should be compatible with the 0.2.12 host"
            );
        }
    }

    #[test]
    fn newer_minor_or_patch_is_rejected_with_a_clear_message() {
        // A patch ahead of the host ceiling — and a different line — must fail
        // loud, naming versions.
        for bad in ["wasi:http/types@0.2.13", "wasi:http/types@0.3.0"] {
            let msg = wasi_http_incompatibility([bad].into_iter())
                .unwrap_or_else(|| panic!("{bad} should be rejected"));
            assert!(msg.contains("plugin requires wasi:http"), "{msg}");
            assert!(
                msg.contains("0.2.12"),
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

#[cfg(test)]
mod egress_policy_tests {
    use super::{EgressDenied, EgressPolicy, TcpTarget};
    use std::net::SocketAddr;

    /// A policy written before FIDIUS-I-0034: overrides ONLY `authorize_tcp`.
    struct LegacyLoopbackOnly;
    impl EgressPolicy for LegacyLoopbackOnly {
        fn authorize(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
            Err(EgressDenied::new("http denied"))
        }
        fn authorize_tcp(&self, addr: &SocketAddr) -> Result<(), EgressDenied> {
            if addr.ip().is_loopback() {
                Ok(())
            } else {
                Err(EgressDenied::new("only loopback"))
            }
        }
    }

    #[test]
    fn authorize_tcp_target_default_delegates_to_authorize_tcp() {
        let policy = LegacyLoopbackOnly;
        for (addr, allowed) in [
            ("127.0.0.1:5432", true),
            ("[::1]:5432", true),
            ("10.0.0.5:5432", false),
        ] {
            let addr: SocketAddr = addr.parse().unwrap();
            // With and without a pinned host, the default must give exactly
            // authorize_tcp's verdict on the resolved addr.
            for host in [None, Some("db.internal")] {
                assert_eq!(
                    policy
                        .authorize_tcp_target(&TcpTarget { host, addr })
                        .is_ok(),
                    policy.authorize_tcp(&addr).is_ok(),
                    "delegation must match authorize_tcp for {addr} (host: {host:?})"
                );
                assert_eq!(
                    policy
                        .authorize_tcp_target(&TcpTarget { host, addr })
                        .is_ok(),
                    allowed
                );
            }
        }
    }

    #[test]
    fn authorize_dns_defaults_to_allow() {
        // Opposite polarity of authorize_tcp's default-deny — see the trait docs.
        assert!(LegacyLoopbackOnly.authorize_dns("anything.example").is_ok());
        assert!(LegacyLoopbackOnly
            .authorize_tcp(&"8.8.8.8:53".parse().unwrap())
            .is_err());
    }

    /// FIDIUS-I-0035: the response hook is strictly opt-in.
    #[test]
    fn response_hook_defaults_are_opt_out() {
        use super::ResponseDirective;

        let policy = LegacyLoopbackOnly;
        assert!(!policy.observes_responses());

        let (parts, _) = http::Request::builder()
            .uri("https://api.example/v1/thing")
            .body(())
            .unwrap()
            .into_parts();
        let headers = parts.headers.clone();
        for retry_available in [true, false] {
            assert_eq!(
                policy.on_response(
                    &parts,
                    http::StatusCode::UNAUTHORIZED,
                    &headers,
                    retry_available,
                ),
                ResponseDirective::Forward,
                "a policy that overrides nothing must always forward"
            );
        }
    }
}

/// FIDIUS-I-0035: the observing dispatch (`dispatch_observed`) against real
/// loopback servers — retry mechanics without a wasm guest (the guest-visible
/// path is covered by the response-hook e2e suite).
#[cfg(test)]
mod response_hook_dispatch_tests {
    use super::{
        dispatch_observed, EgressDenied, EgressPolicy, OutgoingRequestConfig, ResponseDirective,
        TeeBody,
    };
    use crate::executor::body_tee::replay_body;
    use std::io::{Read as _, Write as _};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    const R401: &str =
        "HTTP/1.1 401 Unauthorized\r\ncontent-length: 0\r\nconnection: close\r\n\r\n";
    const R200: &str = "HTTP/1.1 200 OK\r\ncontent-length: 0\r\nconnection: close\r\n\r\n";

    /// Scripted loopback server: serves `responses` in order, one connection
    /// each, recording every request head it saw. Extra connections are not
    /// accepted — a runaway retry loop shows up as a client-side error.
    fn scripted_server(
        responses: &'static [&'static str],
    ) -> (String, std::sync::mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for resp in responses {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
                    match stream.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    }
                }
                let _ = tx.send(String::from_utf8_lossy(&buf).into_owned());
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.flush();
            }
        });
        (addr.to_string(), rx)
    }

    fn parts_for(addr: &str) -> http::request::Parts {
        http::Request::builder()
            .method(http::Method::GET)
            .uri(format!("http://{addr}/v1/data"))
            .body(())
            .unwrap()
            .into_parts()
            .0
    }

    fn config() -> OutgoingRequestConfig {
        let t = std::time::Duration::from_secs(5);
        OutgoingRequestConfig {
            use_tls: false,
            connect_timeout: t,
            first_byte_timeout: t,
            between_bytes_timeout: t,
        }
    }

    /// An empty body in the shape wasi-http's `BodyImpl` hands us: end only
    /// observable by POLLING (`is_end_stream()` stays false), exactly like a
    /// guest that finished its (empty) outgoing body before dispatch.
    struct ChannelShapedEmpty;
    impl http_body::Body for ChannelShapedEmpty {
        type Data = bytes::Bytes;
        type Error = super::ErrorCode;
        fn poll_frame(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
            std::task::Poll::Ready(None)
        }
    }

    /// Run the observing dispatch the way `send_request` does: pre-authorize
    /// clone, authorize, tee a wasi-http-shaped empty body, dispatch.
    async fn run(
        policy: Arc<dyn EgressPolicy>,
        addr: &str,
    ) -> Result<super::IncomingResponse, super::ErrorCode> {
        use http_body_util::BodyExt;
        let mut parts = parts_for(addr);
        let original = parts.clone();
        policy
            .authorize(&mut parts)
            .expect("first authorize allows");
        let (teed, capture) = TeeBody::wrap(ChannelShapedEmpty.boxed_unsync());
        dispatch_observed(policy, original, parts, teed, capture, config()).await
    }

    /// The motivating policy: stamps a credential, refreshes it on 401.
    /// `authorize` asserts it always sees a CLEAN request — the retry must
    /// re-stamp the pre-authorize clone, not the as-dispatched parts.
    struct RefreshOn401 {
        stamps: AtomicUsize,
        observations: Mutex<Vec<(u16, bool)>>,
    }

    impl RefreshOn401 {
        fn new() -> Self {
            Self {
                stamps: AtomicUsize::new(0),
                observations: Mutex::new(Vec::new()),
            }
        }
    }

    impl EgressPolicy for RefreshOn401 {
        fn authorize(&self, parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
            assert!(
                parts.headers.get("x-cred").is_none(),
                "retry must re-authorize a clean pre-authorize clone"
            );
            let n = self.stamps.fetch_add(1, Ordering::SeqCst);
            let cred = if n == 0 { "stale" } else { "fresh" };
            parts.headers.insert("x-cred", cred.parse().unwrap());
            Ok(())
        }
        fn observes_responses(&self) -> bool {
            true
        }
        fn on_response(
            &self,
            _request: &http::request::Parts,
            status: http::StatusCode,
            _headers: &http::HeaderMap,
            retry_available: bool,
        ) -> ResponseDirective {
            self.observations
                .lock()
                .unwrap()
                .push((status.as_u16(), retry_available));
            if status == http::StatusCode::UNAUTHORIZED && retry_available {
                ResponseDirective::RetryOnce
            } else {
                ResponseDirective::Forward
            }
        }
    }

    #[tokio::test]
    async fn retry_once_restamps_and_forwards_the_second_response() {
        let (addr, heads) = scripted_server(&[R401, R200]);
        let policy = Arc::new(RefreshOn401::new());

        let resp = run(policy.clone(), &addr).await.expect("dispatch succeeds");
        assert_eq!(resp.resp.status(), http::StatusCode::OK);

        // The server saw exactly two requests: stale credential, then fresh.
        let first = heads.recv().unwrap();
        let second = heads.recv().unwrap();
        assert!(first.contains("x-cred: stale"), "first head: {first}");
        assert!(second.contains("x-cred: fresh"), "second head: {second}");
        assert!(heads.try_recv().is_err(), "exactly two requests");

        // Both responses were observed; the second with retry_available=false.
        assert_eq!(
            *policy.observations.lock().unwrap(),
            vec![(401, true), (200, false)]
        );
    }

    /// A policy that consumes the 401 and then refuses to re-stamp: the guest
    /// gets the generic request-denied error, not the stale 401.
    struct DenyOnRetry(AtomicUsize);

    impl EgressPolicy for DenyOnRetry {
        fn authorize(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
            match self.0.fetch_add(1, Ordering::SeqCst) {
                0 => Ok(()),
                _ => Err(EgressDenied::new("re-mint failed")),
            }
        }
        fn observes_responses(&self) -> bool {
            true
        }
        fn on_response(
            &self,
            _request: &http::request::Parts,
            _status: http::StatusCode,
            _headers: &http::HeaderMap,
            _retry_available: bool,
        ) -> ResponseDirective {
            ResponseDirective::RetryOnce
        }
    }

    #[tokio::test]
    async fn deny_on_retry_maps_to_request_denied() {
        let (addr, heads) = scripted_server(&[R401]);
        let err = run(Arc::new(DenyOnRetry(AtomicUsize::new(0))), &addr)
            .await
            .expect_err("retry authorize denied");
        assert!(
            matches!(err, super::ErrorCode::HttpRequestDenied),
            "got: {err:?}"
        );
        let _ = heads.recv().unwrap();
        assert!(heads.try_recv().is_err(), "exactly one request dispatched");
    }

    /// An always-RetryOnce policy is bounded to a single retry: the second
    /// 401 forwards (its directive is ignored) and the server sees exactly
    /// two requests.
    struct AlwaysRetry;

    impl EgressPolicy for AlwaysRetry {
        fn authorize(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
            Ok(())
        }
        fn observes_responses(&self) -> bool {
            true
        }
        fn on_response(
            &self,
            _request: &http::request::Parts,
            _status: http::StatusCode,
            _headers: &http::HeaderMap,
            _retry_available: bool,
        ) -> ResponseDirective {
            ResponseDirective::RetryOnce
        }
    }

    #[tokio::test]
    async fn retry_is_bounded_to_one() {
        let (addr, heads) = scripted_server(&[R401, R401]);
        let resp = run(Arc::new(AlwaysRetry), &addr).await.expect("forwarded");
        assert_eq!(resp.resp.status(), http::StatusCode::UNAUTHORIZED);
        let _ = heads.recv().unwrap();
        let _ = heads.recv().unwrap();
        assert!(
            heads.try_recv().is_err(),
            "exactly two requests, never more"
        );
    }

    /// A body streaming past the 64 KiB cap is not replayable: the 401
    /// forwards even though the policy asked to retry.
    #[tokio::test]
    async fn oversized_body_forwards() {
        // Bespoke server: read the head, respond 401, then drain the rest of
        // the body to EOF so the client's in-flight write never sees a reset.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut buf = Vec::new();
            let mut chunk = [0u8; 65536];
            while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
                match stream.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => buf.extend_from_slice(&chunk[..n]),
                }
            }
            let _ = tx.send(());
            let _ = stream.write_all(R401.as_bytes());
            let _ = stream.flush();
            while matches!(stream.read(&mut chunk), Ok(n) if n > 0) {}
        });

        let policy: Arc<dyn EgressPolicy> = Arc::new(AlwaysRetry);
        let mut parts = parts_for(&addr);
        parts.method = http::Method::POST;
        let original = parts.clone();
        policy.authorize(&mut parts).unwrap();
        let big = bytes::Bytes::from(vec![0u8; 2 * crate::executor::body_tee::REPLAY_CAP]);
        let (teed, capture) = TeeBody::wrap(replay_body(big));
        let resp = dispatch_observed(policy, original, parts, teed, capture, config())
            .await
            .expect("401 forwards");
        assert_eq!(resp.resp.status(), http::StatusCode::UNAUTHORIZED);
        rx.recv().unwrap();
        assert!(rx.try_recv().is_err(), "no retry for an oversized body");
    }

    /// A non-replayable body (here: carrying trailers) forwards the 401
    /// untouched even though the policy asked to retry.
    #[tokio::test]
    async fn non_replayable_body_forwards() {
        use http_body::{Body, Frame};
        use std::pin::Pin;
        use std::task::{Context, Poll};

        struct TrailerBody(Vec<Frame<bytes::Bytes>>);
        impl Body for TrailerBody {
            type Data = bytes::Bytes;
            type Error = super::ErrorCode;
            fn poll_frame(
                mut self: Pin<&mut Self>,
                _cx: &mut Context<'_>,
            ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
                Poll::Ready(if self.0.is_empty() {
                    None
                } else {
                    Some(Ok(self.0.remove(0)))
                })
            }
        }

        let (addr, heads) = scripted_server(&[R401]);
        let policy: Arc<dyn EgressPolicy> = Arc::new(AlwaysRetry);
        let mut parts = parts_for(&addr);
        parts.method = http::Method::POST;
        let original = parts.clone();
        policy.authorize(&mut parts).unwrap();
        let body = {
            use http_body_util::BodyExt;
            TrailerBody(vec![
                Frame::data(bytes::Bytes::from_static(b"payload")),
                Frame::trailers(http::HeaderMap::new()),
            ])
            .boxed_unsync()
        };
        let (teed, capture) = TeeBody::wrap(body);
        let resp = dispatch_observed(policy, original, parts, teed, capture, config())
            .await
            .expect("401 forwards");
        assert_eq!(resp.resp.status(), http::StatusCode::UNAUTHORIZED);
        let _ = heads.recv().unwrap();
        assert!(heads.try_recv().is_err(), "no retry for a trailered body");
    }
}
