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

//! `fidius::http` — the brokered outbound HTTP client for sandboxed WASM
//! connectors (FIDIUS-I-0028).
//!
//! A connector's `read()` calls [`get`]/[`post`]/[`send`]; the call crosses the
//! sandbox via `wasi:http/outgoing-handler`, where the **host's `EgressPolicy`**
//! decides allow/deny/decorate before the request leaves the process. There is
//! no ambient network: the request only succeeds when the package declares the
//! `http` capability **and** the host supplied a policy (the two-key gate).
//!
//! This module is `#[cfg(target_family = "wasm")]` and exists only in components
//! built for `wasm32-wasip2`. It is **the one place** the `wasi:http` version is
//! pinned (vendored WIT at `crates/fidius-guest/wit`, matched to the host's
//! `wasmtime-wasi-http`); see [[FIDIUS-A-0005]] for the stability contract.
//!
//! ```ignore
//! fn read(&self, cfg: Config) -> fidius::Stream<Record> {
//!     let resp = fidius::http::get(&cfg.url).expect("fetch");
//!     // … parse resp.body into records …
//! }
//! ```

// The single wasi:http `generate!` for the whole ecosystem. The macro's export
// `generate!` (in the connector crate) composes with this at link time — proven
// in the FIDIUS-T-0144 spike — so the connector imports wasi:http without any
// macro change.
mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "http-client",
        generate_all,
    });
}

use bindings::wasi::http::outgoing_handler;
use bindings::wasi::http::types::{
    Fields, Method, OutgoingBody, OutgoingRequest, RequestOptions, Scheme,
};
use bindings::wasi::io::streams::StreamError;

/// An outbound request. Build with [`Request::get`]/[`Request::post`] or the
/// struct literal, then [`send`].
#[derive(Debug, Clone)]
pub struct Request {
    /// HTTP method (uppercase, e.g. `"GET"`, `"POST"`).
    pub method: String,
    /// Full URL: `http://host[:port]/path?query` or `https://…`.
    pub url: String,
    /// Request headers (name, value) pairs.
    pub headers: Vec<(String, String)>,
    /// Request body (empty for GET).
    pub body: Vec<u8>,
    /// Optional request timeout. When set, it bounds connect, first-byte, and
    /// between-bytes waits (wasi:http `request-options`); a slow upstream then
    /// fails with an [`HttpError`] instead of hanging the call. `None` = no
    /// fidius-imposed timeout (the host/runtime default applies).
    pub timeout: Option<core::time::Duration>,
}

impl Request {
    /// A GET request for `url`.
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: "GET".into(),
            url: url.into(),
            headers: Vec::new(),
            body: Vec::new(),
            timeout: None,
        }
    }

    /// A POST request for `url` with `body`.
    pub fn post(url: impl Into<String>, body: impl Into<Vec<u8>>) -> Self {
        Self {
            method: "POST".into(),
            url: url.into(),
            headers: Vec::new(),
            body: body.into(),
            timeout: None,
        }
    }

    /// Add a header (builder style).
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Set a request timeout (builder style). Bounds connect / first-byte /
    /// between-bytes waits so a slow upstream fails fast instead of hanging.
    pub fn timeout(mut self, dur: core::time::Duration) -> Self {
        self.timeout = Some(dur);
        self
    }
}

/// A response. `body` is the fully-read response body.
#[derive(Debug, Clone)]
pub struct Response {
    /// HTTP status code.
    pub status: u16,
    /// Response headers (name, value) pairs.
    pub headers: Vec<(String, String)>,
    /// The response body, fully read.
    pub body: Vec<u8>,
}

impl Response {
    /// `true` for a 2xx status.
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// The body as UTF-8 (lossy).
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

/// A failed request. The most common cause in a sandbox is a **denied egress**:
/// the host's `EgressPolicy` refused the request, surfaced here as a transport
/// error (the guest never learns the policy's reasoning — by design).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpError {
    /// A short, guest-visible description.
    pub message: String,
}

impl HttpError {
    fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl core::fmt::Display for HttpError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "http: {}", self.message)
    }
}

impl std::error::Error for HttpError {}

/// GET `url`.
pub fn get(url: &str) -> Result<Response, HttpError> {
    send(Request::get(url))
}

/// POST `body` to `url`.
pub fn post(url: &str, body: &[u8]) -> Result<Response, HttpError> {
    send(Request::post(url, body.to_vec()))
}

/// Send an arbitrary [`Request`], blocking until the response is read. The host
/// `EgressPolicy` is consulted before dispatch.
pub fn send(req: Request) -> Result<Response, HttpError> {
    let (scheme, rest) = if let Some(r) = req.url.strip_prefix("http://") {
        (Scheme::Http, r)
    } else if let Some(r) = req.url.strip_prefix("https://") {
        (Scheme::Https, r)
    } else {
        return Err(HttpError::new(format!(
            "unsupported url scheme: {}",
            req.url
        )));
    };
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };

    let method = match req.method.as_str() {
        "GET" => Method::Get,
        "POST" => Method::Post,
        "PUT" => Method::Put,
        "DELETE" => Method::Delete,
        "HEAD" => Method::Head,
        "PATCH" => Method::Patch,
        other => Method::Other(other.to_string()),
    };

    // Request headers.
    let fields = Fields::new();
    for (name, value) in &req.headers {
        fields
            .append(name, &value.clone().into_bytes())
            .map_err(|e| HttpError::new(format!("header {name}: {e:?}")))?;
    }

    let outgoing = OutgoingRequest::new(fields);
    outgoing
        .set_method(&method)
        .map_err(|_| HttpError::new("set_method"))?;
    outgoing
        .set_scheme(Some(&scheme))
        .map_err(|_| HttpError::new("set_scheme"))?;
    outgoing
        .set_authority(Some(authority))
        .map_err(|_| HttpError::new("set_authority"))?;
    outgoing
        .set_path_with_query(Some(path))
        .map_err(|_| HttpError::new("set_path"))?;

    // Body: write (if any) then finish before dispatch.
    let body = outgoing.body().map_err(|_| HttpError::new("body"))?;
    if !req.body.is_empty() {
        let stream = body.write().map_err(|_| HttpError::new("body.write"))?;
        // wasi:io caps a single write at 4096; chunk to be safe.
        for chunk in req.body.chunks(4096) {
            stream
                .blocking_write_and_flush(chunk)
                .map_err(|e| HttpError::new(format!("write: {e:?}")))?;
        }
        drop(stream);
    }
    OutgoingBody::finish(body, None).map_err(|e| HttpError::new(format!("finish: {e:?}")))?;

    // Build request-options when a timeout was set (wasi:http durations are
    // nanoseconds). The same bound applies to connect, first-byte, and
    // between-bytes waits so any stall fails fast.
    let options = req.timeout.map(|d| {
        let ns = d.as_nanos().min(u64::MAX as u128) as u64;
        let opts = RequestOptions::new();
        let _ = opts.set_connect_timeout(Some(ns));
        let _ = opts.set_first_byte_timeout(Some(ns));
        let _ = opts.set_between_bytes_timeout(Some(ns));
        opts
    });
    let fut = outgoing_handler::handle(outgoing, options)
        .map_err(|e| HttpError::new(format!("dispatch denied or failed: {e:?}")))?;
    fut.subscribe().block();
    let resp = fut
        .get()
        .ok_or_else(|| HttpError::new("no response"))?
        .map_err(|_| HttpError::new("response already taken"))?
        .map_err(|e| HttpError::new(format!("response error: {e:?}")))?;

    let status = resp.status();
    let headers = resp
        .headers()
        .entries()
        .into_iter()
        .map(|(k, v)| (k, String::from_utf8_lossy(&v).into_owned()))
        .collect();

    let incoming = resp.consume().map_err(|_| HttpError::new("consume"))?;
    let stream = incoming.stream().map_err(|_| HttpError::new("stream"))?;
    let mut buf = Vec::new();
    loop {
        match stream.blocking_read(8192) {
            Ok(chunk) if chunk.is_empty() => continue,
            Ok(chunk) => buf.extend_from_slice(&chunk),
            Err(StreamError::Closed) => break,
            Err(e) => return Err(HttpError::new(format!("read: {e:?}"))),
        }
    }

    Ok(Response {
        status,
        headers,
        body: buf,
    })
}
