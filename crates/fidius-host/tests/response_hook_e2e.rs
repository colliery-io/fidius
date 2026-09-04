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

//! Egress response hook E2E (FIDIUS-I-0035). A real WASM component
//! (`tests/wasm-fixtures/fetcher`) makes an outbound GET through an
//! [`EgressPolicy`] that opts into response observation; a scripted loopback
//! server answers 401-then-200. The auth-retry loop closes **invisibly to the
//! guest**: its single `fetch` succeeds while the wire saw two requests —
//! stale credential, then fresh.
//!
//! Non-replayable-body cases (oversized/trailered requests) are covered at
//! the dispatch layer in `executor::wasm::response_hook_dispatch_tests` — the
//! fetcher fixture is GET-only, so a guest-visible variant would need a new
//! fixture for a constraint that is enforced below the guest boundary anyway.

#![cfg(feature = "wasm")]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};

use fidius_core::descriptor::BufferStrategyKind;
use fidius_host::executor::{
    EgressDenied, EgressPolicy, ResponseDirective, WasmComponentExecutor, WasmMethod,
};
use fidius_host::{CallError, PluginHandle, PluginInfo, PluginRuntimeKind};

const IFACE: &str = "fidius:fetcher/fetcher@1.0.0";

fn fetcher_component() -> Option<Vec<u8>> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/wasm-fixtures/fetcher/fetcher_guest.wasm");
    std::fs::read(p).ok()
}

/// Scripted loopback server: serves `responses` in order, one connection
/// each, recording every request head it saw. Extra connections are never
/// accepted, so an unbounded retry shows up as a guest-visible error.
fn scripted_server(responses: &'static [&'static str]) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::channel();
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
    (format!("http://{addr}/v1/data"), rx)
}

fn load(egress: Arc<dyn EgressPolicy>) -> Result<PluginHandle, CallError> {
    let bytes = fetcher_component().expect("fetcher_guest.wasm present");
    let info = PluginInfo {
        name: "fetcher".into(),
        interface_name: "fetcher".into(),
        interface_hash: 0,
        interface_version: 1,
        capabilities: 0,
        buffer_strategy: BufferStrategyKind::PluginAllocated,
        runtime: PluginRuntimeKind::Wasm,
    };
    let methods = vec![WasmMethod {
        name: "fetch".into(),
        wire_raw: false,
        streaming: false,
    }];
    WasmComponentExecutor::from_component_bytes_with_egress(
        &bytes,
        IFACE.into(),
        methods,
        vec!["http".into()],
        Some(egress),
        info,
    )
    .map(PluginHandle::from_wasm)
}

/// The motivating weir shape: `authorize` stamps the current credential
/// (host-side; the guest never sees it), `on_response` matches a 401,
/// "re-mints" and retries. `authorize` also asserts it is always handed a
/// CLEAN request — the retry must re-stamp the pre-authorize clone, never a
/// request already carrying the stale header.
struct RefreshOn401 {
    stamps: AtomicUsize,
    observations: Mutex<Vec<(u16, bool)>>,
}

impl RefreshOn401 {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            stamps: AtomicUsize::new(0),
            observations: Mutex::new(Vec::new()),
        })
    }
}

impl EgressPolicy for RefreshOn401 {
    fn authorize(&self, parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
        assert!(
            parts.headers.get("x-cred").is_none(),
            "retry must re-authorize a clean pre-authorize clone"
        );
        let cred = if self.stamps.fetch_add(1, Ordering::SeqCst) == 0 {
            "stale"
        } else {
            "fresh"
        };
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

/// 401-then-200: the guest's single fetch succeeds; the server saw exactly
/// two requests — stale credential, then fresh. The retry is invisible to
/// the guest.
#[test]
fn auth_retry_is_invisible_to_the_guest() {
    if fetcher_component().is_none() {
        eprintln!("SKIP auth_retry_is_invisible_to_the_guest: fetcher_guest.wasm not built");
        return;
    }
    static RESPONSES: &[&str] = &[
        "HTTP/1.1 401 Unauthorized\r\ncontent-length: 4\r\nconnection: close\r\n\r\nnope",
        "HTTP/1.1 200 OK\r\ncontent-length: 13\r\nconnection: close\r\n\r\nfresh payload",
    ];
    let (url, heads) = scripted_server(RESPONSES);
    let policy = RefreshOn401::new();
    let handle = load(policy.clone()).expect("load");

    let body: String = handle.call_method(0, &(url,)).expect("fetch");
    assert_eq!(body, "fresh payload", "guest sees only the 200");

    let first = heads.recv().unwrap();
    let second = heads.recv().unwrap();
    assert!(first.contains("x-cred: stale"), "first head: {first}");
    assert!(second.contains("x-cred: fresh"), "second head: {second}");
    assert!(heads.try_recv().is_err(), "exactly two wire requests");
    assert_eq!(
        *policy.observations.lock().unwrap(),
        vec![(401, true), (200, false)],
        "second observation must carry retry_available=false"
    );
}

/// An always-RetryOnce policy is bounded by fidius to a single retry: the
/// second 401 forwards to the guest and the server sees exactly two requests.
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

#[test]
fn second_retry_directive_is_ignored() {
    if fetcher_component().is_none() {
        eprintln!("SKIP second_retry_directive_is_ignored");
        return;
    }
    static RESPONSES: &[&str] = &[
        "HTTP/1.1 401 Unauthorized\r\ncontent-length: 5\r\nconnection: close\r\n\r\nfirst",
        "HTTP/1.1 401 Unauthorized\r\ncontent-length: 11\r\nconnection: close\r\n\r\nstill stale",
    ];
    let (url, heads) = scripted_server(RESPONSES);
    let handle = load(Arc::new(AlwaysRetry)).expect("load");

    // The guest receives the SECOND response's body: one retry happened, the
    // directive on its observation was ignored.
    let body: String = handle.call_method(0, &(url,)).expect("fetch");
    assert_eq!(body, "still stale");

    let _ = heads.recv().unwrap();
    let _ = heads.recv().unwrap();
    assert!(
        heads.try_recv().is_err(),
        "exactly two requests, never more"
    );
}

/// The policy consumes the 401 and then refuses to re-stamp: the guest gets
/// the generic denied error (as if `authorize` had refused up front), not the
/// stale 401.
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

#[test]
fn deny_on_retry_surfaces_as_denied() {
    if fetcher_component().is_none() {
        eprintln!("SKIP deny_on_retry_surfaces_as_denied");
        return;
    }
    static RESPONSES: &[&str] =
        &["HTTP/1.1 401 Unauthorized\r\ncontent-length: 4\r\nconnection: close\r\n\r\nnope"];
    let (url, heads) = scripted_server(RESPONSES);
    let handle = load(Arc::new(DenyOnRetry(AtomicUsize::new(0)))).expect("load");

    let body: String = handle.call_method(0, &(url,)).expect("fetch");
    assert!(
        body.starts_with("ERROR:"),
        "guest must see a denied error, not the consumed 401; got: {body}"
    );
    let _ = heads.recv().unwrap();
    assert!(heads.try_recv().is_err(), "no second dispatch after deny");
}

/// A policy overriding neither `observes_responses` nor `on_response` rides
/// the pre-FIDIUS-I-0035 dispatch path: plain fetch works, one wire request.
/// (The broader byte-identical guarantee is the existing egress e2e suites
/// passing unmodified.)
struct AllowAll;

impl EgressPolicy for AllowAll {
    fn authorize(&self, _parts: &mut http::request::Parts) -> Result<(), EgressDenied> {
        Ok(())
    }
}

#[test]
fn policy_without_overrides_is_unaffected() {
    if fetcher_component().is_none() {
        eprintln!("SKIP policy_without_overrides_is_unaffected");
        return;
    }
    static RESPONSES: &[&str] =
        &["HTTP/1.1 200 OK\r\ncontent-length: 5\r\nconnection: close\r\n\r\nplain"];
    let (url, heads) = scripted_server(RESPONSES);
    let handle = load(Arc::new(AllowAll)).expect("load");
    let body: String = handle.call_method(0, &(url,)).expect("fetch");
    assert_eq!(body, "plain");
    let _ = heads.recv().unwrap();
    assert!(heads.try_recv().is_err(), "exactly one request");
}
