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

//! `fidius::sockets` — capability-gated outbound **TCP** for sandboxed WASM
//! connectors (FIDIUS-I-0033).
//!
//! This is the missing seam for connectors that speak a raw binary wire protocol
//! over TCP rather than HTTP — most importantly database/warehouse drivers (e.g.
//! Postgres on `:5432`), which a host can't broker as HTTP. A pure-Rust **sync**
//! driver built on [`tcp::TcpStream`] (with `rustls` layered on top for TLS) runs
//! fully sandboxed, closing the last gap that kept DB connectors on the retired
//! native path.
//!
//! ## Backed by `std::net` → `wasi:sockets`
//!
//! Unlike [`crate::http`] (which wraps `wasi:http` because std has no HTTP), TCP
//! needs no hand-written WIT: on `wasm32-wasip2` Rust's `std::net::TcpStream`
//! **is** `wasi:sockets` (since Rust 1.83, std's net layer calls wasi-libc's
//! socket functions). So [`tcp::TcpStream`] is a thin, blocking wrapper over
//! `std::net::TcpStream` and is **portable**: identical source compiles for the
//! host (a normal socket) and for `wasm32-wasip2` (a sandboxed one). Sync/blocking
//! semantics suit the off-tokio `block_on` execution model.
//!
//! ## Two-key, fail-closed — the same gate as `http`
//!
//! A connect only succeeds when **both** keys are turned:
//! 1. the package declares the `tcp` capability in `[wasm].capabilities`, **and**
//! 2. the host supplied an `EgressPolicy` whose `authorize_tcp(host:port)` allows
//!    the (resolved) peer.
//!
//! Miss either and the connect is refused (the deny-all `WasiCtx` default) — there
//! is no ambient network. The guest sees a normal [`std::io::Error`]; it never
//! learns the policy's reasoning, by design.
//!
//! ```ignore
//! use std::io::{Read, Write};
//! use fidius_guest::sockets::tcp;
//!
//! // Allowed only if the host's EgressPolicy::authorize_tcp permits db:5432.
//! let mut conn = tcp::connect("db.internal", 5432)?;
//! conn.write_all(startup_packet)?;
//! let mut buf = [0u8; 1024];
//! let n = conn.read(&mut buf)?;
//! ```
//!
//! `TcpStream` implements [`Read`] + [`Write`], so a TLS layer composes directly:
//!
//! ```ignore
//! let mut tls = rustls::Stream::new(&mut client_conn, &mut conn);
//! tls.write_all(startup_packet)?; // TLS over the sandboxed TCP stream
//! ```

/// Blocking outbound TCP, the policy-gated counterpart of [`crate::http`].
pub mod tcp {
    use std::io::{self, Read, Write};
    use std::net::{SocketAddr, TcpStream as StdTcpStream, ToSocketAddrs};
    use std::time::{Duration, Instant};

    /// A connected, blocking TCP stream. A thin newtype over
    /// [`std::net::TcpStream`] — on `wasm32-wasip2` that is `wasi:sockets`, so the
    /// connection only exists when the host's `EgressPolicy` allowed the peer.
    ///
    /// Implements [`Read`] + [`Write`] (and the same for `&TcpStream`, mirroring
    /// std), so byte-oriented drivers and TLS stacks (`rustls`) layer on directly.
    #[derive(Debug)]
    pub struct TcpStream {
        inner: StdTcpStream,
    }

    /// Open a TCP connection to `host:port`, blocking until connected.
    ///
    /// `host` may be a hostname or a literal IP. A hostname is resolved first
    /// (via `wasi:sockets` name-lookup in the sandbox); the host's
    /// `EgressPolicy::authorize_tcp` is then consulted on the **resolved** peer,
    /// so an allow-list keyed on a name must account for resolution (the embedder
    /// closes DNS-rebinding with resolve-and-pin — see the host docs).
    ///
    /// Fails with [`std::io::Error`] if the peer is not allow-listed (a denied
    /// egress is indistinguishable from an unreachable host, by design), if
    /// resolution fails, or on a normal connection error.
    pub fn connect(host: &str, port: u16) -> io::Result<TcpStream> {
        StdTcpStream::connect((host, port)).map(|inner| TcpStream { inner })
    }

    impl TcpStream {
        /// Connect to any [`ToSocketAddrs`] target — e.g. a `&str` like
        /// `"db.internal:5432"` or a pre-resolved [`SocketAddr`]. Each candidate
        /// address is still gated by the host policy.
        pub fn connect_to<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
            StdTcpStream::connect(addr).map(|inner| TcpStream { inner })
        }

        /// Connect with a bound on the **total** connect time, resolving `host`
        /// first and trying each candidate address until one connects (or the
        /// timeout elapses). `timeout` is a budget across *all* candidates, not
        /// per-address — a name resolving to several dead addresses (e.g. dual-stack
        /// A/AAAA) still fails within `timeout`, so a slow/blocked peer fails fast
        /// instead of hanging the blocking call.
        pub fn connect_timeout(host: &str, port: u16, timeout: Duration) -> io::Result<TcpStream> {
            let deadline = Instant::now() + timeout;
            let mut last_err = None;
            for addr in (host, port).to_socket_addrs()? {
                // Split the remaining budget across candidates; once it's spent,
                // stop rather than starting another full-`timeout` attempt.
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    break;
                }
                match StdTcpStream::connect_timeout(&addr, remaining) {
                    Ok(inner) => return Ok(TcpStream { inner }),
                    Err(e) => last_err = Some(e),
                }
            }
            Err(last_err.unwrap_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "no address resolved for host")
            }))
        }

        /// Set the read timeout; `None` blocks indefinitely. (Some socket options
        /// have reduced fidelity on `wasm32-wasip2` — see the target docs.)
        pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
            self.inner.set_read_timeout(dur)
        }

        /// Set the write timeout; `None` blocks indefinitely.
        pub fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
            self.inner.set_write_timeout(dur)
        }

        /// Enable/disable `TCP_NODELAY` (disable Nagle's algorithm).
        pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
            self.inner.set_nodelay(nodelay)
        }

        /// The remote peer's address.
        pub fn peer_addr(&self) -> io::Result<SocketAddr> {
            self.inner.peer_addr()
        }

        /// The local address this socket is bound to.
        pub fn local_addr(&self) -> io::Result<SocketAddr> {
            self.inner.local_addr()
        }

        /// Borrow the underlying [`std::net::TcpStream`] — an escape hatch for
        /// std-typed APIs.
        pub fn get_ref(&self) -> &StdTcpStream {
            &self.inner
        }

        /// Mutably borrow the underlying [`std::net::TcpStream`].
        pub fn get_mut(&mut self) -> &mut StdTcpStream {
            &mut self.inner
        }

        /// Consume `self`, returning the underlying [`std::net::TcpStream`].
        pub fn into_std(self) -> StdTcpStream {
            self.inner
        }
    }

    impl From<StdTcpStream> for TcpStream {
        fn from(inner: StdTcpStream) -> Self {
            TcpStream { inner }
        }
    }

    impl Read for TcpStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.inner.read(buf)
        }
    }

    impl Write for TcpStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.inner.write(buf)
        }
        fn flush(&mut self) -> io::Result<()> {
            self.inner.flush()
        }
    }

    // std impls Read/Write for `&TcpStream` too (the socket is full-duplex and
    // internally synchronized); mirror that so a shared reference reads/writes,
    // which some driver/TLS plumbing relies on.
    impl Read for &TcpStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            (&self.inner).read(buf)
        }
    }

    impl Write for &TcpStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            (&self.inner).write(buf)
        }
        fn flush(&mut self) -> io::Result<()> {
            (&self.inner).flush()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::tcp;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// The surface is portable: on the host it is a real socket. Round-trip a few
    /// bytes through a loopback echo to prove `connect` + `Read`/`Write` (the same
    /// code path a sandboxed guest drives over `wasi:sockets`).
    #[test]
    fn connect_read_write_roundtrip() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            let mut buf = [0u8; 5];
            s.read_exact(&mut buf).unwrap();
            s.write_all(&buf).unwrap(); // echo
        });

        let mut conn = tcp::connect("127.0.0.1", addr.port()).expect("connect");
        conn.write_all(b"hello").unwrap();
        let mut back = [0u8; 5];
        conn.read_exact(&mut back).unwrap();
        assert_eq!(&back, b"hello");
        assert_eq!(conn.peer_addr().unwrap().port(), addr.port());
        server.join().unwrap();
    }

    #[test]
    fn connect_refused_is_an_io_error() {
        // Nothing listening on this loopback port → a normal io::Error (the same
        // shape a denied egress surfaces as in the sandbox).
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener); // free the port so the connect is refused
        assert!(tcp::connect("127.0.0.1", port).is_err());
    }
}
