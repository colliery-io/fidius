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

//! Per-call latency: fidius plugin backends vs a localhost network round-trip
//! (FIDIUS-I-0024). Answers the "plugins are too slow / costly vs microservices"
//! pushback with numbers. Run: `cargo bench -p fidius-host --features wasm`.
//!
//! Backends:
//! - **cdylib** — native dynamic library, in-process FFI (bincode over the vtable).
//! - **wasm (JIT)** — Component Model in the wasmtime sandbox (JIT-compiled).
//! - **wasm (AOT)** — same, loaded from a precompiled `.cwasm`.
//! - **localhost TCP** — a persistent-connection round-trip to an in-process
//!   echo/add server. This is a *generous lower bound* for a microservice: no
//!   HTTP/gRPC framing, no TLS, no serialization library, no cross-host network,
//!   no per-call connect. A real microservice is strictly slower than this.
//!
//! Two ops, chosen so the same logical work runs on every backend:
//! - `add(i64, i64) -> i64` — tiny payload; dominated by call/dispatch overhead.
//! - `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::Command;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use fidius_core::wasm_descriptor::{WasmInterfaceDescriptor, WasmMethodDesc};
use fidius_host::{PluginHandle, PluginHost, PluginInfo, PluginRuntimeKind};
use fidius_test::dylib_fixture;

// ── greeter (wasm) descriptor — mirrors tests/wasm-fixtures/greeter ──────────
const IFACE: &str = "fidius:greeter/greeter@1.0.0";
const HASH: u64 = 0x0102_0304_0506_0708;
static METHODS: [WasmMethodDesc; 4] = [
    WasmMethodDesc {
        name: "greet",
        wire_raw: false,
    },
    WasmMethodDesc {
        name: "add",
        wire_raw: false,
    },
    WasmMethodDesc {
        name: "echo-bytes",
        wire_raw: true,
    },
    WasmMethodDesc {
        name: "probe-env",
        wire_raw: false,
    },
];
static GREETER: WasmInterfaceDescriptor = WasmInterfaceDescriptor {
    interface_name: "greeter",
    interface_export: IFACE,
    interface_hash: HASH,
    methods: &METHODS,
};
// greeter method indices: greet=0, add=1, echo-bytes=2 (raw), probe-env=3.
const W_ADD: usize = 1;
const W_ECHO: usize = 2;
// test-plugin-smoke: BasicCalculator.add_direct=1; ReverseBytes.reverse=0 (raw).
const C_ADD: usize = 1;
const C_ECHO: usize = 0;

const SIZES: &[usize] = &[64, 4096, 262_144];

fn greeter_component() -> Vec<u8> {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/wasm-fixtures/greeter");
    let status = Command::new("cargo")
        .args(["component", "build", "--release"])
        .current_dir(&fixture)
        .status()
        .expect("cargo component build (toolchain: see docs/how-to/wasm-component-toolchain.md)");
    assert!(status.success(), "greeter build failed");
    std::fs::read(fixture.join("target/wasm32-wasip1/release/greeter_guest.wasm")).unwrap()
}

/// Stage a wasm package dir (optionally with a precompiled `.cwasm`) and load it.
fn load_wasm(host: &PluginHost, root: &std::path::Path, bytes: &[u8], aot: bool) -> PluginHandle {
    let dir = root.join(if aot { "g-aot" } else { "g-jit" });
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("greeter_guest.wasm"), bytes).unwrap();
    let precompiled = if aot {
        let cwasm = fidius_host::executor::precompile_component(bytes).expect("precompile");
        std::fs::write(dir.join("greeter_guest.cwasm"), &cwasm).unwrap();
        "precompiled = \"greeter_guest.cwasm\"\n"
    } else {
        ""
    };
    std::fs::write(
        dir.join("package.toml"),
        format!(
            "[package]\nname = \"g-{0}\"\nversion = \"0.1.0\"\ninterface = \"greeter\"\n\
             interface_version = 1\nruntime = \"wasm\"\n\n[metadata]\ncategory = \"bench\"\n\n\
             [wasm]\ncomponent = \"greeter_guest.wasm\"\n{1}",
            if aot { "aot" } else { "jit" },
            precompiled
        ),
    )
    .unwrap();
    host.load_wasm(&format!("g-{}", if aot { "aot" } else { "jit" }), &GREETER)
        .expect("load_wasm")
}

/// The op a request asks the server to do. `add` sums two LE i64s; `echo`
/// reverses the payload — the same work the plugin backends do.
fn compute(op_is_add: bool, body: &[u8]) -> Vec<u8> {
    if op_is_add {
        let a = i64::from_le_bytes(body[0..8].try_into().unwrap());
        let b = i64::from_le_bytes(body[8..16].try_into().unwrap());
        (a + b).to_le_bytes().to_vec()
    } else {
        body.iter().rev().copied().collect()
    }
}

// ── Transport 1 & 2: length-prefixed framing over TCP / Unix socket (IPC). ──
// Request: [op:u8][len:u32 LE][body]; response: [len:u32 LE][body]. This is the
// thinnest possible RPC — a generous lower bound for a microservice.
fn serve_lenprefix<S: Read + Write>(mut s: S) {
    loop {
        let mut hdr = [0u8; 5];
        if s.read_exact(&mut hdr).is_err() {
            break;
        }
        let len = u32::from_le_bytes([hdr[1], hdr[2], hdr[3], hdr[4]]) as usize;
        let mut body = vec![0u8; len];
        if s.read_exact(&mut body).is_err() {
            break;
        }
        let resp = compute(hdr[0] == 0, &body);
        let mut out = (resp.len() as u32).to_le_bytes().to_vec();
        out.extend_from_slice(&resp);
        if s.write_all(&out).is_err() {
            break;
        }
    }
}

fn lenprefix_call<S: Read + Write>(s: &mut S, op: u8, payload: &[u8]) -> Vec<u8> {
    let mut req = vec![op];
    req.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    req.extend_from_slice(payload);
    s.write_all(&req).unwrap();
    let mut lenb = [0u8; 4];
    s.read_exact(&mut lenb).unwrap();
    let mut resp = vec![0u8; u32::from_le_bytes(lenb) as usize];
    s.read_exact(&mut resp).unwrap();
    resp
}

fn spawn_tcp() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = l.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for s in l.incoming().flatten() {
            s.set_nodelay(true).ok();
            serve_lenprefix(s);
        }
    });
    port
}

fn spawn_uds(path: PathBuf) {
    let l = UnixListener::bind(&path).unwrap();
    std::thread::spawn(move || {
        for s in l.incoming().flatten() {
            serve_lenprefix(s);
        }
    });
}

// ── Transport 3: real HTTP/1.1 over a keep-alive localhost connection. ──
// The common microservice transport. Still a lower bound (no TLS, no framework,
// no cross-host network), but it pays HTTP framing + header parsing per call.
fn spawn_http() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = l.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for mut s in l.incoming().flatten() {
            s.set_nodelay(true).ok();
            let mut buf = Vec::new();
            let mut tmp = [0u8; 8192];
            'conn: loop {
                // Read headers up to \r\n\r\n.
                let header_end = loop {
                    if let Some(p) = find_subslice(&buf, b"\r\n\r\n") {
                        break p + 4;
                    }
                    match s.read(&mut tmp) {
                        Ok(0) | Err(_) => break 'conn,
                        Ok(n) => buf.extend_from_slice(&tmp[..n]),
                    }
                };
                let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
                let is_add = head.starts_with("POST /add");
                let clen = content_length(&head);
                while buf.len() < header_end + clen {
                    match s.read(&mut tmp) {
                        Ok(0) | Err(_) => break 'conn,
                        Ok(n) => buf.extend_from_slice(&tmp[..n]),
                    }
                }
                let body = buf[header_end..header_end + clen].to_vec();
                buf.drain(..header_end + clen);
                let resp = compute(is_add, &body);
                let mut out = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n",
                    resp.len()
                )
                .into_bytes();
                out.extend_from_slice(&resp);
                if s.write_all(&out).is_err() {
                    break;
                }
            }
        }
    });
    port
}

fn http_call(s: &mut TcpStream, path: &str, payload: &[u8]) -> Vec<u8> {
    let mut req = format!(
        "POST {path} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n",
        payload.len()
    )
    .into_bytes();
    req.extend_from_slice(payload);
    s.write_all(&req).unwrap();

    let mut buf = Vec::new();
    let mut tmp = [0u8; 8192];
    let header_end = loop {
        if let Some(p) = find_subslice(&buf, b"\r\n\r\n") {
            break p + 4;
        }
        let n = s.read(&mut tmp).unwrap();
        buf.extend_from_slice(&tmp[..n]);
    };
    let clen = content_length(&String::from_utf8_lossy(&buf[..header_end]));
    while buf.len() < header_end + clen {
        let n = s.read(&mut tmp).unwrap();
        buf.extend_from_slice(&tmp[..n]);
    }
    buf[header_end..header_end + clen].to_vec()
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn content_length(head: &str) -> usize {
    head.lines()
        .find_map(|l| {
            l.to_ascii_lowercase()
                .strip_prefix("content-length:")
                .map(|v| v.trim().parse().unwrap_or(0))
        })
        .unwrap_or(0)
}

fn cdylib_handle(host: &PluginHost, name: &str) -> PluginHandle {
    PluginHandle::from_loaded(host.load(name).unwrap())
}

fn benches(c: &mut Criterion) {
    // ── set up every backend ────────────────────────────────────────────────
    let tmp = tempfile::TempDir::new().unwrap();
    let wasm_bytes = greeter_component();
    let host = PluginHost::builder()
        .search_path(tmp.path())
        .build()
        .unwrap();
    let w_jit = load_wasm(&host, tmp.path(), &wasm_bytes, false);
    let w_aot = load_wasm(&host, tmp.path(), &wasm_bytes, true);

    let dylib_dir = dylib_fixture(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/test-plugin-smoke"),
    )
    .with_release(true)
    .build();
    let chost = PluginHost::builder()
        .search_path(dylib_dir.dir())
        .build()
        .unwrap();
    let c_calc = cdylib_handle(&chost, "BasicCalculator");
    let c_rev = cdylib_handle(&chost, "ReverseBytes");
    let _: &PluginInfo = c_calc.info(); // touch to keep the runtime-kind import used
    assert_eq!(w_jit.info().runtime, PluginRuntimeKind::Wasm);

    // Three "microservice" transports, each on a warm persistent connection.
    let mut tcp = TcpStream::connect(("127.0.0.1", spawn_tcp())).unwrap();
    tcp.set_nodelay(true).unwrap();
    let uds_path = tmp.path().join("ms.sock");
    spawn_uds(uds_path.clone());
    let mut uds = UnixStream::connect(&uds_path).unwrap();
    let mut http = TcpStream::connect(("127.0.0.1", spawn_http())).unwrap();
    http.set_nodelay(true).unwrap();

    let add_payload = {
        let mut p = 2i64.to_le_bytes().to_vec();
        p.extend_from_slice(&3i64.to_le_bytes());
        p
    };

    // ── add(i64, i64) — call/dispatch overhead ───────────────────────────────
    let mut g = c.benchmark_group("add");
    g.bench_function("cdylib", |b| {
        b.iter(|| {
            c_calc
                .call_method::<(i64, i64), i64>(C_ADD, &(2, 3))
                .unwrap()
        })
    });
    g.bench_function("wasm_jit", |b| {
        b.iter(|| {
            w_jit
                .call_method::<(i64, i64), i64>(W_ADD, &(2, 3))
                .unwrap()
        })
    });
    g.bench_function("wasm_aot", |b| {
        b.iter(|| {
            w_aot
                .call_method::<(i64, i64), i64>(W_ADD, &(2, 3))
                .unwrap()
        })
    });
    g.bench_function("localhost_tcp", |b| {
        b.iter(|| lenprefix_call(&mut tcp, 0, &add_payload))
    });
    g.bench_function("unix_socket", |b| {
        b.iter(|| lenprefix_call(&mut uds, 0, &add_payload))
    });
    g.bench_function("http", |b| {
        b.iter(|| http_call(&mut http, "/add", &add_payload))
    });
    g.finish();

    // ── echo(bytes) — payload marshalling / throughput ───────────────────────
    let mut g = c.benchmark_group("echo");
    for &size in SIZES {
        let payload = vec![0xABu8; size];
        g.throughput(Throughput::Bytes(size as u64));
        g.bench_with_input(BenchmarkId::new("cdylib", size), &payload, |b, p| {
            b.iter(|| c_rev.call_method_raw(C_ECHO, p).unwrap())
        });
        g.bench_with_input(BenchmarkId::new("wasm_jit", size), &payload, |b, p| {
            b.iter(|| w_jit.call_method_raw(W_ECHO, p).unwrap())
        });
        g.bench_with_input(BenchmarkId::new("wasm_aot", size), &payload, |b, p| {
            b.iter(|| w_aot.call_method_raw(W_ECHO, p).unwrap())
        });
        g.bench_with_input(BenchmarkId::new("localhost_tcp", size), &payload, |b, p| {
            b.iter(|| lenprefix_call(&mut tcp, 1, p))
        });
        g.bench_with_input(BenchmarkId::new("unix_socket", size), &payload, |b, p| {
            b.iter(|| lenprefix_call(&mut uds, 1, p))
        });
        g.bench_with_input(BenchmarkId::new("http", size), &payload, |b, p| {
            b.iter(|| http_call(&mut http, "/echo", p))
        });
    }
    g.finish();
}

criterion_group!(b, benches);
criterion_main!(b);
