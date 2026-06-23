#!/usr/bin/env bash
# Build the wasi:sockets TCP-echo component (FIDIUS-I-0033 E2). The wasm32-wasip2
# target emits a component directly; std::net::TcpStream lowers to wasi:sockets,
# so no vendored WIT is needed (unlike the wasi:http fetcher). Produces
# tcp_echo_guest.wasm.
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release --target wasm32-wasip2
cp target/wasm32-wasip2/release/tcp_echo_guest.wasm ./tcp_echo_guest.wasm
echo "built tcp_echo_guest.wasm"
