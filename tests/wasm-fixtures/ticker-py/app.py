# Copyright 2026 Colliery, Inc. Licensed under Apache 2.0
#
# Non-Rust polyglot *streaming* guest (FIDIUS-I-0026): a Python implementation of
# the SAME `ticker` WIT as the Rust/JS guests, built into a WASM component with
# componentize-py. It exports a streaming `tick-stream` resource the fidius host
# polls identically — proving the streaming contract is language-neutral across
# Rust, JavaScript, and Python.
#
# WIT mapping (componentize-py): u32 -> int, u64 -> int, an exported resource ->
# a Python class implementing the generated `TickStream` protocol, and
# result<option<u64>, plugin-error> -> return the option (int for some, None for
# none = clean end) and raise for the error arm.

from typing import Optional

import wit_world.exports.ticker as ticker_exports


class TickStream(ticker_exports.TickStream):
    """The server-stream resource handle. `next()` is the poll method."""

    def __init__(self, count: int):
        self.i = 0
        self.count = count

    def next(self) -> Optional[int]:
        if self.i < self.count:
            v = self.i
            self.i += 1
            return v  # ok(some(v))
        return None  # ok(none) -> clean end of stream


class Ticker:
    """Implements the exported `ticker` interface (the free functions)."""

    def tick(self, count: int) -> TickStream:
        return TickStream(count)

    def fidius_interface_hash(self) -> int:
        # fnv1a("tick:u32->u64!stream"); must match the host's expected hash.
        return 0xFD152C8AA1112FC3
