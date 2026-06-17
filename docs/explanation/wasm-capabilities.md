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

# Capabilities & the WASM Sandbox

Native (cdylib) plugins run with the host's full authority — they can open files,
make network calls, and read the environment, because they are just shared
libraries. WASM component plugins are different: they run inside a wasmtime
sandbox with **no ambient authority**. A plugin can do nothing to the outside
world unless the package's `[wasm].capabilities` allow-list explicitly grants it.
This is the security reason to ship a plugin as a component.

## The model: WASI present, zero grants

A subtle but important design point (FIDIUS-T-0102): real components built by
standard toolchains *import* WASI interfaces (`wasi:cli`, `wasi:io`,
`wasi:clocks`, `wasi:filesystem`, …) even when the plugin never calls them — the
language runtime references them. An **empty** wasmtime `Linker` therefore can't
even instantiate such a component.

So fidius does **not** use an empty linker. Instead it:

1. Wires `wasmtime-wasi` into the `Linker` so any conforming component
   instantiates, **and**
2. Gives the guest a **deny-all `WasiCtx`** — no filesystem preopens, no
   environment, no inherited stdio, no sockets.

The WASI *interfaces* are present (so the component links), but they are backed
by a context that grants **nothing**. Capabilities in the manifest selectively
open specific facets of that context.

## Declaring capabilities

Capabilities are a string allow-list under `[wasm]` in `package.toml`:

```toml
[wasm]
component = "plugin.wasm"
capabilities = ["clocks", "random", "stdout"]
```

An **empty or absent** list (`capabilities = []`) is the default: a fully
deny-all sandbox. `fidius package inspect` surfaces this list prominently — it is
the single most security-relevant field a deployer reviews before trusting a
plugin.

### Recognized capabilities

| Capability          | Grants                                                        |
| ------------------- | ------------------------------------------------------------- |
| `env`               | Read the host's environment variables                         |
| `args`              | Read process arguments                                        |
| `stdout` / `stderr` | Write to the host's standard out / error                      |
| `stdin`             | Read the host's standard input                                |
| `network` / `sockets` | Outbound network + DNS name lookup (coarse; see below)      |
| `clocks`            | Wall/monotonic clocks (always available; accepted as a no-op) |
| `random`            | Secure randomness (always available; accepted as a no-op)     |

Unknown names **fail closed**: a manifest listing a capability fidius doesn't
recognize (a typo, or an unsupported one) is rejected at load with a clear error,
rather than silently granting nothing. This is verified by the
`unknown_capability_rejected_at_load` test.

### Filesystem is never grantable

There is deliberately **no** `filesystem` capability. v1 never grants filesystem
access — there are no preopens, ever, and `"filesystem"` in a manifest is an
unknown-capability error. A plugin that needs to work with file *contents* should
receive them as method arguments (bytes), not reach into the host filesystem.
Granular, path-scoped filesystem grants are a possible future addition; the
current posture is "no filesystem, full stop."

### Network is coarse in v1

`network`/`sockets` enable outbound connections and DNS lookup, but without
per-host or per-port filtering. It is still opt-in and off by default; treat it
as "this plugin may talk to the network" and gate it on review. Fine-grained
egress policy is a documented follow-on.

## How a deployer reasons about it

Because the package is **signed** (see [Signing Plugins](../tutorials/signing-plugins.md))
and the signature covers the whole package including `package.toml`, the
capability list cannot be altered after signing without invalidating the
signature. So the trust workflow is:

1. `fidius package inspect` the package and read the `Capabilities` line.
2. Decide whether those grants are acceptable for this plugin's job.
3. Verify the signature against a trusted key (`require_signature(true)` +
   `trusted_keys`), which also guarantees the capability list is the one the
   signer approved.

A plugin asking for `network` when it claims to be a pure data transform is a red
flag the allow-list makes visible.

## Relation to the interface hash

Capabilities are about *authority*; the `fidius-interface-hash` is about
*contract integrity* (the plugin implements the interface the host expects). They
are independent: the hash check rejects a plugin built against the wrong
interface; the capability list bounds what a correctly-typed plugin may do; and
the signature is the security boundary over both. See the
[WASM Component ABI](wasm-component-abi.md) for the hash, and
[Your First WASM Plugin](../tutorials/your-first-wasm-plugin.md) for the end-to-end
flow.
