<!--
Copyright 2026 Colliery, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
-->

# Performance — plugins vs a microservice

The recurring pushback on a plugin architecture is "it's too slow / too costly
compared to microservices." This page answers that with numbers from a
reproducible benchmark, and is honest about where a plugin backend wins, where it
ties, and where it currently loses.

## What's measured

`crates/fidius-host/benches/backends.rs` (criterion) runs the **same two
operations** on every backend:

- `add(i64, i64) -> i64` — a tiny call; dominated by per-call/dispatch overhead.
- `echo(bytes) -> bytes` at 64 B / 4 KiB / 256 KiB — payload marshalling/throughput.

Backends:

| Backend | What it is |
|---|---|
| **cdylib** | native dynamic library, in-process FFI (bincode over the vtable) |
| **wasm (JIT)** | WASM component in the wasmtime sandbox, JIT-compiled |
| **wasm (AOT)** | same, loaded from a precompiled `.cwasm` |
| **localhost TCP** | length-prefixed round-trip, persistent connection — a *generous lower bound* for a microservice |
| **unix socket** | same framing over a Unix domain socket (local IPC) |
| **HTTP** | real HTTP/1.1 request/response, keep-alive, on localhost |

The three network/IPC backends are deliberately **lower bounds** for a
microservice: no TLS, no serialization framework, no cross-host network, no
per-call connect — a real microservice is strictly slower. Run it yourself:

```bash
cargo bench -p fidius-host --features wasm --bench backends
```

!!! note "Read ratios, not absolutes"
    Numbers below are medians on one dev machine (Apple/Darwin, criterion,
    short measurement window). Absolute figures vary by hardware; the
    **orders of magnitude between backends** are the point. Large-payload
    network numbers are also sensitive to OS socket-buffer tuning.

## Results

Numbers below are **after** the two WASM optimizations described in the next
section (both landed pre-release).

**`add` (tiny call):**

| backend | median |
|---|---|
| cdylib | **~34 ns** |
| unix socket | ~6.4 µs |
| HTTP (localhost) | ~17 µs |
| localhost TCP | ~18 µs |
| wasm (JIT) | ~24 µs |
| wasm (AOT) | ~24 µs |

**`echo` (bytes), median per call:**

| backend | 64 B | 4 KiB | 256 KiB |
|---|---|---|---|
| cdylib | **~36 ns** | **~241 ns** | **~21 µs** |
| unix socket | ~9.8 µs | ~13 µs | ~603 µs |
| HTTP (localhost) | ~31 µs | ~17 µs | ~124 µs |
| localhost TCP | ~22 µs | ~23 µs | ~94 µs |
| wasm (JIT) | ~22 µs | ~24 µs | ~119 µs |
| wasm (AOT) | ~22 µs | ~33 µs | ~124 µs |

### WASM optimizations (landed before release)

The first run exposed two artifacts in the WASM path; both were fixed in
`fidius-host`, and the tables above reflect the fixes:

| optimization | before | after |
|---|---|---|
| **Cache `InstancePre`** — build the WASI `Linker` + pre-instantiate **once** at load instead of rebuilding it on every call (per-call still gets a fresh `Store` for isolation). | `add` ~90–124 µs | **~24 µs** (~4–5×) |
| **Typed raw-bytes path** — `#[wire(raw)]` `list<u8>` now uses wasmtime's typed call (bulk memcpy) instead of a `Val::List` of one `Val::U8` per byte. | 256 KiB echo ~6.7 ms | **~120 µs** (~55×) |

## Reading the numbers

**1. The native (cdylib) backend isn't close — it wins by 2–3 orders of
magnitude.** A tiny call is ~34 ns vs ~6–18 µs for any local transport
(~200–500× faster), and ~21 µs vs ~94 µs–603 µs at 256 KiB. An in-process
function call has no syscall, no copy across a socket, no scheduler hop. For a
native plugin the "too slow vs microservices" claim is simply false.

**2. The WASM backend now *matches* a local microservice on latency — while
adding a sandbox and polyglot support.** A tiny call is ~24 µs (vs ~17 µs HTTP,
~18 µs TCP, ~6 µs UDS), and a 256 KiB payload is ~120 µs (≈ HTTP's ~124 µs,
faster than UDS's ~603 µs). It pays a sandbox tax over cdylib, but it is in the
same band as the network transports a microservice would use — and it carries no
standing process (point 3). The earlier 6.7 ms / ~100 µs-floor figures were
fixed-by-design artifacts (fresh-instance-per-call + per-byte `Val`), now
addressed by the two optimizations above.

**3. The cost argument is separate from latency, and plugins win it outright.** A
microservice is a *running process*: idle RAM, a scheduler slot, a port, plus the
operational tax of deploys, restarts, health checks, and autoscaling. A plugin is
a loaded artifact: **no idle process**, N plugins share one host process, and load
is a one-time cost — cdylib `dlopen`, or for WASM ~83 µs from a precompiled
`.cwasm` (~6.6 ms JIT), per the [spike](wasm-component-abi.md). So WASM matches a
local microservice's latency *and* avoids its process + ops footprint entirely.

## Guidance

- **Latency-critical, trusted code** → cdylib. Nothing else is within two orders
  of magnitude.
- **Untrusted / polyglot / capability-scoped code** → WASM. You get the sandbox +
  language independence at roughly local-microservice latency and zero standing
  footprint. Prefer `#[wire(raw)]` for bulk bytes (typed bulk-copy path) and
  precompile to `.cwasm`.
- **vs a microservice** → a plugin removes the network hop *and* the standing
  process. cdylib is faster on both latency and footprint by orders of magnitude;
  WASM matches local-transport latency and still wins decisively on footprint and
  ops.

## See also

- [WASM Component ABI](wasm-component-abi.md) — the `Value ↔ Val` marshalling these numbers exercise
- [Capabilities & the WASM Sandbox](wasm-capabilities.md) — what the WASM per-call cost buys
- `crates/fidius-host/benches/backends.rs` — the benchmark source
