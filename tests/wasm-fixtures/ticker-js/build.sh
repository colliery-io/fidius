#!/usr/bin/env bash
# Build the JavaScript polyglot *streaming* ticker component (FIDIUS-I-0026) with
# jco/ComponentizeJS. Produces ticker_js.wasm implementing the same fidius:ticker
# WIT (a server-streaming `tick-stream` resource) as the Rust guest.
set -euo pipefail
cd "$(dirname "$0")"
npx -y @bytecodealliance/jco componentize ticker.js \
  --wit ../ticker/wit --world-name ticker-plugin --disable http fetch-event \
  --out ticker_js.wasm
echo "built ticker_js.wasm"
