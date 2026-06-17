#!/usr/bin/env bash
# Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
#
# Build the Python polyglot greeter component (FIDIUS-T-0105). Requires
# `componentize-py` (pip install componentize-py). Produces greeter_py.wasm,
# a WASM component implementing the SAME fidius:greeter WIT as the Rust guest.
set -euo pipefail
cd "$(dirname "$0")"

# Prefer a repo-local venv install, else whatever's on PATH.
CPY="componentize-py"
if [ -x "../../../.venv/bin/componentize-py" ]; then
  CPY="../../../.venv/bin/componentize-py"
fi

PYTHONPATH=. "$CPY" -d ../greeter/wit -w greeter-plugin componentize app -o greeter_py.wasm
echo "built greeter_py.wasm"
