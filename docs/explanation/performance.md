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

**`add` (tiny call):**

| backend | median |
|---|---|
| cdylib | **~42 ns** |
| unix socket | ~8.9 µs |
| HTTP (localhost) | ~19 µs |
| localhost TCP | ~24 µs |
| wasm (AOT) | ~97 µs |
| wasm (JIT) | ~124 µs |

**`echo` (bytes), median per call:**

| backend | 64 B | 4 KiB | 256 KiB |
|---|---|---|---|
| cdylib | **~39 ns** | **~218 ns** | **~15 µs** |
| unix socket | ~8.3 µs | ~10 µs | ~485 µs |
| HTTP (localhost) | ~18 µs | ~26 µs | ~138 µs |
| localhost TCP | ~20 µs | ~21 µs | ~96 µs |
| wasm (JIT/AOT) | ~86 µs | ~171 µs | ~6.7 ms |

## Reading the numbers

**1. The native (cdylib) backend isn't close — it wins by 2–3 orders of
magnitude.** A tiny call is ~40 ns vs ~9–24 µs for any local transport
(~200–600× faster), and ~15 µs vs ~100 µs–6.9 ms at 256 KiB. An in-process
function call has no syscall, no copy across a socket, no scheduler hop. For a
native plugin the "too slow vs microservices" claim is simply false.

**2. The WASM backend, *as currently implemented*, is in the same ballpark as a
local microservice — it is not faster.** ~86–124 µs for a tiny call (vs ~10–25 µs
for HTTP/UDS) and ~6.7 ms for a 256 KiB payload. If the "too slow" feedback comes
from benchmarking the WASM path, **it is fair** — and it's a fixable artifact, not
fundamental:

- The executor builds a **fresh wasmtime `Store` and re-instantiates the
  component on every call** (for per-call isolation), and **copies the whole
  payload through the `Value ↔ component::Val` space**. Instantiation is the
  fixed ~85 µs floor; the copy is what turns 256 KiB into milliseconds.
- **Mitigations (designed-for, not yet implemented):** cache the
  `InstancePre` / reuse a pooled `Store` for trusted long-lived plugins (removes
  the per-call instantiation floor), and write `#[wire(raw)]` bulk bytes straight
  into guest memory instead of round-tripping through `Value` (removes the
  large-payload copy). These are optimizations on the existing design and should
  recover most of the gap; today's numbers reflect the **safe-by-default
  fresh-instance-per-call** posture.

So WASM today trades latency for **sandboxing + polyglot** — not for speed.

**3. The cost argument is separate from latency, and plugins win it outright.** A
microservice is a *running process*: idle RAM, a scheduler slot, a port, plus the
operational tax of deploys, restarts, health checks, and autoscaling. A plugin is
a loaded artifact: **no idle process**, N plugins share one host process, and load
is a one-time cost — cdylib `dlopen`, or for WASM ~83 µs from a precompiled
`.cwasm` (~6.6 ms JIT), per the [spike](wasm-component-abi.md). So even where WASM
per-call *latency* ties a local microservice, it avoids the per-service process
and ops footprint entirely.

## Guidance

- **Latency-critical, trusted code** → cdylib. Nothing else is within two orders
  of magnitude.
- **Untrusted / polyglot / capability-scoped code** → WASM. Accept the per-call
  overhead for the sandbox + language independence; keep payloads modest, prefer
  `#[wire(raw)]` for bulk bytes, and precompile to `.cwasm`. Instance reuse (a
  planned optimization) is the lever if WASM per-call latency matters.
- **vs a microservice** → a plugin removes the network hop *and* the standing
  process. cdylib is faster on both latency and footprint; WASM is comparable on
  local latency but still wins decisively on footprint and ops.

## See also

- [WASM Component ABI](wasm-component-abi.md) — the `Value ↔ Val` marshalling these numbers exercise
- [Capabilities & the WASM Sandbox](wasm-capabilities.md) — what the WASM per-call cost buys
- `crates/fidius-host/benches/backends.rs` — the benchmark source
