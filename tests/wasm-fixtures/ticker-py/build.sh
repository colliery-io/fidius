#!/usr/bin/env bash
# Build the Python polyglot *streaming* ticker component (FIDIUS-I-0026) with
# componentize-py. Produces ticker_py.wasm implementing the same fidius:ticker
# WIT (a server-streaming `tick-stream` resource) as the Rust/JS guests.
set -euo pipefail
cd "$(dirname "$0")"
CPY="componentize-py"
if [ -x "../../../.venv/bin/componentize-py" ]; then
  CPY="../../../.venv/bin/componentize-py"
fi
PYTHONPATH=. "$CPY" -d ../ticker/wit -w ticker-plugin componentize app -o ticker_py.wasm
echo "built ticker_py.wasm"
