#!/usr/bin/env bash
# Build the C polyglot greeter component (FIDIUS-I-0025): wit-bindgen (C) for the
# bindings + the wasi-sdk clang targeting wasm32-wasip2 (which links straight to a
# component — no preview1 adapter needed). Produces a tiny (~18 KB) component, no
# embedded runtime.
#
# Tools: wit-bindgen (cargo install wit-bindgen-cli) + wasi-sdk. Point WASI_SDK at
# the install (default: MacPorts /opt/local/libexec/wasi-sdk).
set -euo pipefail
cd "$(dirname "$0")"
WASI_SDK="${WASI_SDK:-/opt/local/libexec/wasi-sdk}"

wit-bindgen c --world greeter-plugin ../greeter/wit --out-dir gen
# NOTE: no -O2 on purpose. With -O the wasi-sdk clang runs `wasm-opt` (if on PATH)
# as a post-link pass, and binaryen can't parse a Component Model binary
# ("surprising value at 0:8"). The guest is trivial, so we skip it.
"$WASI_SDK/bin/clang" --target=wasm32-wasip2 -mexec-model=reactor \
  --sysroot="$WASI_SDK/share/wasi-sysroot" \
  -I gen gen/greeter_plugin.c greeter_impl.c gen/greeter_plugin_component_type.o \
  -o greeter_c.wasm
echo "built greeter_c.wasm"
