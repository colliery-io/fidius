#!/usr/bin/env bash
# Build the C polyglot *streaming* ticker component (FIDIUS-I-0026) with
# wit-bindgen (C) + the wasi-sdk clang. Produces ticker_c.wasm implementing the
# same fidius:ticker WIT (a server-streaming `tick-stream` resource) as the
# Rust/JS/Python guests. No runtime embedded → tiny component.
set -euo pipefail
cd "$(dirname "$0")"
WASI_SDK="${WASI_SDK:-/opt/local/libexec/wasi-sdk}"
wit-bindgen c --world ticker-plugin ../ticker/wit --out-dir gen
"$WASI_SDK/bin/clang" --target=wasm32-wasip2 -mexec-model=reactor \
  --sysroot="$WASI_SDK/share/wasi-sysroot" \
  -I gen gen/ticker_plugin.c ticker_impl.c gen/ticker_plugin_component_type.o \
  -o ticker_c.wasm
echo "built ticker_c.wasm"
