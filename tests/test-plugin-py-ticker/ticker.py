# Copyright 2026 Colliery, Inc.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
"""Python server-streaming implementation of the `Ticker` interface.

`Ticker` (defined in tests/test-plugin-smoke/src/lib.rs) has one method:

    tick(count: int) -> Stream<u64>

declared `-> fidius::Stream<u64>` on the Rust side. A streaming method is just a
normal registered function that *returns a generator* — fidius drives it one
item at a time with the GIL held only per step, so a slow consumer backpressures
the generator and dropping the stream runs its ``finally``.

`__interface_hash__` must match the hash the Rust macro computes for the trait
(which now includes the `!stream` marker). The integration test injects the
real value (read from the macro-generated descriptor) in place of the
``__HASH_PLACEHOLDER__`` sentinel below at stage time, so this file stays in
sync with the Rust side automatically.
"""

from fidius import method

# Replaced by the integration test with the runtime hash from
# `Ticker_PYTHON_DESCRIPTOR.interface_hash`.
__interface_hash__ = __HASH_PLACEHOLDER__

# Module-level sentinel a test can observe: set True when `tick`'s generator
# runs its `finally` (i.e. on clean exhaustion OR on cancellation via
# GeneratorExit). The drop-cancel test writes a marker file from here.
import os

_CLEANUP_MARKER = os.environ.get("FIDIUS_TICKER_CLEANUP_MARKER")


@method
def tick(count):
    """Yield 0, 1, ..., count-1 as a generator (server-streaming)."""
    try:
        i = 0
        while i < count:
            yield i
            i += 1
    finally:
        # Runs on clean completion and on GeneratorExit (drop-cancel).
        if _CLEANUP_MARKER:
            with open(_CLEANUP_MARKER, "w") as fh:
                fh.write("cleaned-up")
