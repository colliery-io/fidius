#!/usr/bin/env bash
# Build the wasi:http fetcher component (FIDIUS-I-0027 E2). The wasm32-wasip2
# target emits a component directly; wit-bindgen reads the vendored 0.2.6 WIT
# (pinned to match wasmtime-wasi-http 45). Produces fetcher_guest.wasm.
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release --target wasm32-wasip2
cp target/wasm32-wasip2/release/fetcher_guest.wasm ./fetcher_guest.wasm
echo "built fetcher_guest.wasm"
