// Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
//
// FIDIUS-I-0033 (E2): the guest side of the TCP egress test — a minimal raw-TCP
// client over `std::net::TcpStream`, which on wasm32-wasip2 is `wasi:sockets`.
// This is precisely the "pure-Rust sync driver over std::net::TcpStream" story
// the feature unlocks (a Postgres driver does the same, just with a real wire
// protocol + TLS on top). `fidius_guest::sockets::tcp::connect` is a thin wrapper
// over exactly this `std::net::TcpStream::connect`.
//
// `connect_and_echo(addr, payload)` connects, writes `payload`, reads it back.
// The host's EgressPolicy::authorize_tcp is consulted on the resolved peer before
// connect; without the two-key grant (tcp capability + policy) the deny-all
// WasiCtx refuses the connect and this returns an empty list.
wit_bindgen::generate!({
    path: "wit",
    world: "tcp-echo-plugin",
    generate_all,
});

use exports::fidius::tcp_echo::tcp_echo::Guest;
use std::io::{Read, Write};
use std::net::TcpStream;

struct Component;

impl Guest for Component {
    fn connect_and_echo(addr: String, payload: Vec<u8>) -> Vec<u8> {
        do_echo(&addr, &payload).unwrap_or_default()
    }

    /// Interface-hash carrier; the host's `load_wasm` checks it against the
    /// descriptor. Arbitrary fixed value for this hand-built fixture.
    fn fidius_interface_hash() -> u64 {
        0x7CCB_0033_0000_0001
    }
}

fn do_echo(addr: &str, payload: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut conn = TcpStream::connect(addr)?;
    conn.write_all(payload)?;
    conn.flush()?;
    let mut back = vec![0u8; payload.len()];
    conn.read_exact(&mut back)?;
    Ok(back)
}

export!(Component);
