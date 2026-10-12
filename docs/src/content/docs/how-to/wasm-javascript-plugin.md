---
title: "A WASM Plugin in JavaScript (jco / ComponentizeJS)"
---

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

A fidius WASM plugin is an ordinary WebAssembly **component** that satisfies an
interface's WIT contract — the language is irrelevant to the host. This guide
implements the **same `greeter` interface** as
[Your First WASM Plugin (Rust)](../../tutorials/your-first-wasm-plugin/) and
[the Python guide](../wasm-python-plugin/), this time in **JavaScript**, and loads
it through the identical host path. Three languages, one host, one descriptor —
that is the polyglot payoff of the Component Model (ADR FIDIUS-A-0003, "Path B").

The worked example is the committed fixture `tests/wasm-fixtures/greeter-js/`,
verified by `polyglot_js_guest_behaves_identically` in `crates/fidius-host`.

## Prerequisites

- Node.js + `jco` (ComponentizeJS): invoked via `npx -y @bytecodealliance/jco`.
- `wasm-tools` to validate — see
  [Set Up the WASM Component Toolchain](../wasm-component-toolchain/).

There is **no fidius dependency** in the guest — it only has to satisfy the WIT
([the same contract](../wasm-python-plugin/#1-the-wit-contract) the Rust and
Python guests implement).

## 1. Implement it in JavaScript

`jco` maps the exported interface to an ESM **named export** matching the
interface (`greeter`). Kebab-case WIT names become lowerCamelCase
(`echo-bytes` → `echoBytes`). Type mapping: `s64`/`u64` → **`BigInt`**,
`list<u8>` → **`Uint8Array`**, `result<T, _>` → return `T` (throw for the error
arm):

```js
// greeter.js
export const greeter = {
  greet(name) {
    return `Hello, ${name}!`;
  },
  add(a, b) {
    return a + b; // BigInt + BigInt — the Ok arm of result<s64, plugin-error>
  },
  echoBytes(data) {
    return new Uint8Array(Array.from(data).reverse());
  },
  fidiusInterfaceHash() {
    return 0x0102030405060708n; // MUST equal the interface hash (BigInt / u64)
  },
};
```

:::caution[The interface hash must match]
`fidius-interface-hash` is an integrity check — the host compares it to the
descriptor's `interface_hash` and refuses a mismatch at load. The interface
author publishes the expected value (the Rust macros compute it from the
signatures). It is **not** a security boundary — signing is.
:::

## 2. Build the component

```bash
npx -y @bytecodealliance/jco componentize greeter.js \
  --wit path/to/wit --world-name greeter-plugin \
  --disable http fetch-event \
  --out greeter_js.wasm

wasm-tools validate --features component-model greeter_js.wasm
```

:::note[Disable features the sandbox won't grant]
ComponentizeJS embeds a JS engine (StarlingMonkey) that, by default, imports
`wasi:http` for `fetch`. fidius's WASM sandbox is **deny-all** and does not
wire `wasi:http` into the linker, so an HTTP-enabled component fails to
instantiate with *"component imports instance `wasi:http/types`, but a
matching implementation was not found"*. `--disable http fetch-event` drops
those imports; `clocks`/`random`/`stdio` are fine (the host provides them).
The result is a component exporting `fidius:greeter/greeter` — the same
artifact shape as the Rust and Python guests (larger, since it embeds the JS
engine: ~12 MB).
:::

## 3. Package, sign, and load

Identical to any fidius package — the `[wasm]` section names the component and its
capability allow-list:

```toml
# greeter-js-pkg/package.toml
[package]
name = "greeter-js-pkg"
version = "0.1.0"
interface = "greeter"
interface_version = 1
runtime = "wasm"

[metadata]
category = "demo"

[wasm]
component = "greeter_js.wasm"
capabilities = []
```

```rust
// The host loads it through the SAME API as the Rust and Python guests.
let handle = host.load_wasm("greeter-js-pkg", &Greeter_WASM_DESCRIPTOR)?;
let greeting: String = handle.call_method(0, &("Ada".to_string(),))?;
assert_eq!(greeting, "Hello, Ada!");
```

The host neither knows nor cares that the component is JavaScript — it loads,
sandboxes, and dispatches it exactly like the Rust and Python guests. That is the
polyglot guarantee.

## See also

- [A WASM Plugin in Go](../wasm-go-plugin/) — the same interface, natively compiled
- [A WASM Plugin in Python](../wasm-python-plugin/)
- [Your First WASM Plugin (Rust)](../../tutorials/your-first-wasm-plugin/)
- [WASM Component ABI](../../explanation/wasm-component-abi/)
