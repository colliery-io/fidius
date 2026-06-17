#!/usr/bin/env bash
# Build the JavaScript polyglot greeter component (FIDIUS-I-0025) with
# jco/ComponentizeJS. Produces greeter_js.wasm implementing the same
# fidius:greeter WIT as the Rust + Python guests.
set -euo pipefail
cd "$(dirname "$0")"
npx -y @bytecodealliance/jco componentize greeter.js \
  --wit ../greeter/wit --world-name greeter-plugin --disable http fetch-event \
  --out greeter_js.wasm
echo "built greeter_js.wasm"
