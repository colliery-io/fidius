# Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
#
# Non-Rust polyglot guest (FIDIUS-T-0105): a Python implementation of the SAME
# `greeter` WIT interface as the Rust guest, built into a WASM component with
# componentize-py. The fidius host loads and calls it identically — the concrete
# proof that Path B (Component Model + WIT) delivers language-agnostic plugins.
#
# Build (see build.sh):
#   componentize-py -d ../greeter/wit -w greeter-plugin componentize app -o greeter_py.wasm

import os


class Greeter:
    """Implements the exported `greeter` interface."""

    def greet(self, name: str) -> str:
        return f"Hello, {name}!"

    def add(self, a: int, b: int) -> int:
        # result<s64, plugin-error>: returning the int is the Ok arm.
        return a + b

    def echo_bytes(self, data: bytes) -> bytes:
        return bytes(reversed(data))

    def probe_env(self) -> bool:
        # Visible only when the host grants the `env` capability.
        return os.environ.get("FIDIUS_TEST_CAP") is not None

    def fidius_interface_hash(self) -> int:
        # Same hash the Rust guest reports — the host validates it at load.
        return 0x0102_0304_0506_0708
