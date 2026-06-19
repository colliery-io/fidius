// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// FIDIUS-I-0027 (E2): the guest side of the egress test — a minimal `wasi:http`
// client. `fetch(url)` issues a GET and returns the body. The host's
// EgressPolicy is consulted (allow/deny/decorate) before the request leaves the
// sandbox; without the two-key grant the `wasi:http` import is absent and this
// component fails to instantiate.
wit_bindgen::generate!({
    path: "wit",
    world: "fetcher-plugin",
    generate_all,
});

use exports::fidius::fetcher::fetcher::Guest;
use wasi::http::outgoing_handler;
use wasi::http::types::{Fields, Method, OutgoingBody, OutgoingRequest, Scheme};
use wasi::io::streams::StreamError;

struct Component;

impl Guest for Component {
    /// Plain-string return so the host test never has to round-trip a WIT
    /// `result<>`: success = the body, failure (incl. a denied/blocked egress) =
    /// `"ERROR: …"`.
    fn fetch(url: String) -> String {
        match do_fetch(url) {
            Ok(body) => body,
            Err(e) => format!("ERROR: {e}"),
        }
    }
}

fn do_fetch(url: String) -> Result<String, String> {
    {
        // Minimal URL split: <scheme>://<authority><path>.
        let (scheme, rest) = if let Some(r) = url.strip_prefix("http://") {
            (Scheme::Http, r)
        } else if let Some(r) = url.strip_prefix("https://") {
            (Scheme::Https, r)
        } else {
            return Err(format!("unsupported url: {url}"));
        };
        let (authority, path) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, "/"),
        };

        let req = OutgoingRequest::new(Fields::new());
        req.set_method(&Method::Get).map_err(|_| "set_method".to_string())?;
        req.set_scheme(Some(&scheme)).map_err(|_| "set_scheme".to_string())?;
        req.set_authority(Some(authority))
            .map_err(|_| "set_authority".to_string())?;
        req.set_path_with_query(Some(path))
            .map_err(|_| "set_path".to_string())?;

        // GET: finish the (empty) request body before dispatch.
        let body = req.body().map_err(|_| "body".to_string())?;
        OutgoingBody::finish(body, None).map_err(|e| format!("finish: {e:?}"))?;

        let fut = outgoing_handler::handle(req, None).map_err(|e| format!("handle: {e:?}"))?;
        // Block until the response future resolves.
        fut.subscribe().block();
        let resp = fut
            .get()
            .ok_or_else(|| "no response".to_string())?
            .map_err(|_| "response already taken".to_string())?
            .map_err(|e| format!("response error: {e:?}"))?;

        let _status = resp.status();
        let incoming = resp.consume().map_err(|_| "consume".to_string())?;
        let stream = incoming.stream().map_err(|_| "stream".to_string())?;

        let mut buf = Vec::new();
        loop {
            match stream.blocking_read(8192) {
                Ok(chunk) if chunk.is_empty() => continue,
                Ok(chunk) => buf.extend_from_slice(&chunk),
                Err(StreamError::Closed) => break,
                Err(e) => return Err(format!("read: {e:?}")),
            }
        }
        String::from_utf8(buf).map_err(|e| e.to_string())
    }
}

export!(Component);
