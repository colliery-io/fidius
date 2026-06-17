#!/usr/bin/env bash
# Build the Go polyglot greeter component (FIDIUS-I-0025) with TinyGo +
# wit-bindgen-go. Produces greeter_go.wasm implementing the same fidius:greeter
# WIT as the Rust/Python/JS guests.
#
# Tools (see .github/workflows/ci.yml for pinned install): tinygo (>=0.41),
# wit-bindgen-go, and wasm-opt (binaryen, via $WASMOPT or PATH).
set -euo pipefail
cd "$(dirname "$0")"

# The local build world imports wasi:cli (TinyGo's runtime needs it) and exports
# the greeter interface. Populate its deps: WASI 0.2.0 wit (from TinyGo's lib)
# and the shared greeter interface.
tinygo_lib="$(cd "$(dirname "$(command -v tinygo)")/.." && pwd)/lib"
rm -rf wit/deps && mkdir -p wit/deps/cli wit/deps/greeter
cp "$tinygo_lib"/wasi-cli/wit/*.wit wit/deps/cli/
cp -r "$tinygo_lib"/wasi-cli/wit/deps/* wit/deps/
cp ../greeter/wit/world.wit wit/deps/greeter/greeter.wit

# Bindings come from the interface-only WIT (clean exports, no wasi noise).
wit-bindgen-go generate -w greeter-plugin -o internal -p greetergo/internal \
  --cm go.bytecodealliance.org/cm ../greeter/wit

tinygo build -target=wasip2 --wit-package wit --wit-world greeter-plugin \
  -o greeter_go.wasm .
echo "built greeter_go.wasm"
