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

# A WASM Plugin in Go (TinyGo)

The **same `greeter` interface** as the [Rust](../tutorials/your-first-wasm-plugin.md),
[Python](wasm-python-plugin.md), and [JavaScript](wasm-javascript-plugin.md)
guests, now in **Go** — compiled to a component with TinyGo and loaded through the
identical host path. Unlike the interpreted guests, this one is *natively
compiled*: the component is ~0.5 MB with no embedded runtime.

The worked example is the committed fixture `tests/wasm-fixtures/greeter-go/`,
verified by `polyglot_go_guest_behaves_identically` in `crates/fidius-host`.

## Prerequisites

- **TinyGo ≥ 0.41** (the standard `go` compiler can't yet emit component-model
  exports; TinyGo can). TinyGo pulls in the matching `go` toolchain.
- **`wit-bindgen-go`**: `go install go.bytecodealliance.org/cmd/wit-bindgen-go@latest`.
- **`wasm-opt`** (binaryen) on `PATH` — TinyGo invokes it.

## 1. Generate bindings + implement

`wit-bindgen-go` turns the WIT into a Go package with an `Exports` struct of
function fields; you assign your implementation to them. WIT type mapping:
`s64`/`u64` → `int64`/`uint64`, `list<u8>` → `cm.List[uint8]`,
`result<T, _>` → `cm.Result[...]` (built with `cm.OK` / `cm.Err`):

```bash
wit-bindgen-go generate -w greeter-plugin -o internal -p mymod/internal \
  --cm go.bytecodealliance.org/cm path/to/greeter/wit
```

```go
// main.go
package main

import (
	greeter "mymod/internal/fidius/greeter/greeter"

	"go.bytecodealliance.org/cm"
)

type addResult = cm.Result[greeter.PluginErrorShape, int64, greeter.PluginError]

func init() {
	greeter.Exports.Greet = func(name string) string { return "Hello, " + name + "!" }
	greeter.Exports.Add = func(a, b int64) addResult {
		return cm.OK[addResult, greeter.PluginErrorShape, int64, greeter.PluginError](a + b)
	}
	greeter.Exports.EchoBytes = func(data cm.List[uint8]) cm.List[uint8] {
		src := data.Slice()
		out := make([]uint8, len(src))
		for i := range src {
			out[i] = src[len(src)-1-i]
		}
		return cm.ToList(out)
	}
	greeter.Exports.FidiusInterfaceHash = func() uint64 { return 0x0102030405060708 }
}

func main() {}
```

The interface hash **must** equal the descriptor's (the host rejects a mismatch
at load); signing — not this hash — is the security boundary.

## 2. Build the component

TinyGo's runtime imports `wasi:cli` (e.g. `wasi:cli/environment`), so the
**build world must declare those imports** even though the greeter interface
itself uses none. Define a small local world that pulls them in and re-exports
the interface:

```wit
// wit/world.wit
package fidius:greeter-go@1.0.0;

world greeter-plugin {
    include wasi:cli/imports@0.2.0;
    export fidius:greeter/greeter@1.0.0;
}
```

Populate `wit/deps/` with the WASI wit (TinyGo ships it under
`$(tinygo)/lib/wasi-cli/wit`) and the greeter interface, then build:

```bash
tinygo build -target=wasip2 --wit-package wit --wit-world greeter-plugin \
  -o greeter_go.wasm .
wasm-tools validate --features component-model greeter_go.wasm
```

!!! note "Why the world imports WASI"
    A bare `export greeter` world fails component encoding with
    *"failed to resolve import `wasi:cli/environment`"* — TinyGo's runtime needs
    it. Declaring `include wasi:cli/imports` makes the import explicit; the host
    wires WASI into the linker (with a **deny-all** `WasiCtx` by default), so the
    import resolves at load without granting the guest anything. See
    `tests/wasm-fixtures/greeter-go/build.sh` for the exact dep wiring.

## 3. Package, sign, and load

Identical to any fidius package (`[wasm].component = "greeter_go.wasm"`). The host
loads it through the same `load_wasm` + descriptor as every other guest:

```rust
let handle = host.load_wasm("greeter-go-pkg", &Greeter_WASM_DESCRIPTOR)?;
let greeting: String = handle.call_method(0, &("Ada".to_string(),))?;
assert_eq!(greeting, "Hello, Ada!");
```

## See also

- [A WASM Plugin in JavaScript](wasm-javascript-plugin.md)
- [A WASM Plugin in Python](wasm-python-plugin.md)
- [Your First WASM Plugin (Rust)](../tutorials/your-first-wasm-plugin.md)
