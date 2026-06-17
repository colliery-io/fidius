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

# Your First WASM Plugin (Rust)

This tutorial walks you through writing a fidius plugin that compiles to a
**sandboxed WebAssembly component** instead of a native `cdylib`. You author it
in Rust with the same `#[plugin_interface]` / `#[plugin_impl]` macros you already
know — the macros emit the WIT component glue for you — then build, package,
sign, and load it through `PluginHost::load_wasm`.

WASM plugins run inside a wasmtime sandbox with **no ambient authority**: no
filesystem, no network, no environment, unless you explicitly grant it (see the
[Capabilities & sandbox](../explanation/wasm-capabilities.md) guide). They are
also **polyglot** — the component your Rust plugin produces is the same kind of
artifact a Python author produces (see
[A WASM Plugin in Python](../how-to/wasm-python-plugin.md)).

## Prerequisites

- The WASM component toolchain — see
  [Set Up the WASM Component Toolchain](../how-to/wasm-component-toolchain.md)
  (the `wasm32-wasip2` target + `wasm-tools`).
- A `fidius` CLI built with WASM support (for `pack` validation/precompile):
  `cargo install --path crates/fidius-cli --features wasm` (or
  `cargo build -p fidius-cli --features wasm`).
- Familiarity with [Your First Plugin](your-first-plugin.md) (the cdylib flow).

## 1. Create the plugin crate

A WASM plugin crate is **wasm-only**: it depends on `fidius-guest` (the
wasm-buildable subset of the fidius runtime — interface hashing, descriptors, the
value model) plus the macros and `wit-bindgen`. It does **not** depend on the
full `fidius` facade, which is host-side and won't compile to wasm.

```toml
# Cargo.toml
[package]
name = "greeter-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
fidius-guest = "0.3"
fidius-macro = "0.3"
wit-bindgen = "0.44"

# Keep the component small.
[profile.release]
opt-level = "s"
lto = true
strip = true
```

## 2. Define the interface and implementation

The only difference from a cdylib plugin is `crate = "fidius_guest"` on both
macros — that points the generated code at the wasm-buildable crate.

```rust
// src/lib.rs
use fidius_macro::{plugin_impl, plugin_interface};

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;

    #[wire(raw)]
    fn echo(&self, data: Vec<u8>) -> Vec<u8>;
}

pub struct MyGreeter;

#[plugin_impl(Greeter, crate = "fidius_guest")]
impl Greeter for MyGreeter {
    fn greet(&self, name: String) -> String {
        format!("Hello, {name}!")
    }

    #[wire(raw)]
    fn echo(&self, data: Vec<u8>) -> Vec<u8> {
        let mut d = data;
        d.reverse();
        d
    }
}
```

On the `wasm32` target, `#[plugin_impl]` emits a `wit-bindgen` `Guest`
implementation that exports your interface as a WIT component (the native cdylib
machinery is compiled out). `#[plugin_interface]` also emits a
`Greeter_WASM_DESCRIPTOR` constant the host uses to load it.

!!! note "Supported types"
    Auto-generated WIT covers `bool`, the sized integers, `f32`/`f64`, `char`,
    `String`, `Vec<T>` (`Vec<u8>` → `list<u8>`), `Option<T>`, and
    `Result<T, PluginError>`. **Your own `struct`s and `enum`s** also work — see
    [Using your own types](#using-your-own-types-records--variants) below. See the
    [WASM Component ABI](../explanation/wasm-component-abi.md) for the full table.

## 3. Build the component

```bash
cargo build --target wasm32-wasip2 --release
```

This produces a **component** (not a core module) at
`target/wasm32-wasip2/release/greeter_wasm.wasm`. Confirm it:

```bash
wasm-tools validate --features component-model \
  target/wasm32-wasip2/release/greeter_wasm.wasm
wasm-tools component wit \
  target/wasm32-wasip2/release/greeter_wasm.wasm
```

The WIT dump shows your exported interface, e.g.
`export fidius:greeter/greeter@0.1.0;` with `greet`, `echo`, and the
`fidius-interface-hash` carrier the host checks at load.

## 4. Stage and pack the package

Create a package directory with a `package.toml` and the component:

```toml
# greeter-pkg/package.toml
[package]
name = "greeter-pkg"
version = "0.1.0"
interface = "greeter"
interface_version = 1
runtime = "wasm"

[metadata]
category = "demo"

[wasm]
component = "greeter.wasm"
# Empty = deny-all sandbox. Add capabilities only as needed (see the guide).
capabilities = []
```

```bash
mkdir -p greeter-pkg
cp target/wasm32-wasip2/release/greeter_wasm.wasm greeter-pkg/greeter.wasm
```

Pack it into a `.fid` archive. `pack` validates that the file is a real
Component-Model component before archiving:

```bash
fidius package pack greeter-pkg
# Validated wasm component: greeter.wasm
# Packed: greeter-pkg-0.1.0.fid (...)
```

!!! tip "Optional: precompile for faster loads"
    `fidius package pack greeter-pkg --precompile` ahead-of-time compiles the
    component to a `greeter.cwasm` (recorded in the manifest) so the host skips
    JIT at load. The `.cwasm` is engine-specific; if it doesn't match the host's
    wasmtime it is **ignored** (the host falls back to JIT), so it is purely a
    latency optimization. Run `--precompile` **before** signing, since it adds a
    file to the package.

## 5. Sign the package

Signing is artifact-agnostic — it covers the whole package directory (the
`.wasm`, the manifest, an optional `.cwasm`), so tampering with the component
invalidates the signature.

```bash
fidius keygen --out mykey                       # mykey.secret + mykey.public
fidius package sign --key mykey.secret greeter-pkg
fidius package pack greeter-pkg                 # the .fid now carries package.sig
```

Inspect what a deployer would review — note the capability allow-list is shown
prominently:

```bash
fidius package inspect greeter-pkg
#   Runtime: wasm
#   WASM:
#     Component: greeter.wasm
#     Precompiled (.cwasm): (none — JIT at load)
#     Capabilities: (none — deny-all sandbox)
```

## 6. Load it from a host

The host references the macro-emitted descriptor (the interface crate or a shared
definition provides `Greeter_WASM_DESCRIPTOR`). Loading is identical to the
cdylib and Python paths — `PluginHost` enforces the same signature policy:

```rust
use fidius_host::PluginHost;

let host = PluginHost::builder()
    .search_path("./packages")
    .require_signature(true)
    .trusted_keys(&[my_public_key])
    .build()?;

let handle = host.load_wasm("greeter-pkg", &Greeter_WASM_DESCRIPTOR)?;

let greeting: String = handle.call_method(0, &("Ada".to_string(),))?;
assert_eq!(greeting, "Hello, Ada!");
```

`load_wasm` validates the component's `fidius-interface-hash` against the
descriptor (rejecting a plugin built against a different interface) and runs the
guest in the deny-all sandbox.

## Using your own types (records & variants)

Real interfaces pass domain types, not just primitives. Annotate a `struct` with
`#[derive(WitType)]` to map it to a WIT `record`, and an `enum` to a WIT
`variant`. Enum cases may be unit, single-field, or **struct-style**
(`Case { .. }`, which synthesizes a payload record). The types may live in
submodules:

```rust
use fidius_macro::{plugin_impl, plugin_interface, WitType};

pub mod geom {                                   // types can live in submodules
    #[derive(super::WitType, Clone)]
    pub struct Point { pub x: i32, pub y: i32 }   // → record point { x: s32, y: s32 }
}
use geom::Point;

#[derive(WitType, Clone)]
pub enum Shape {                                  // → variant shape { ... }
    Circle(u32),                                  //   single-field case
    Rect(Point),                                  //   case carrying a record
    Triangle { base: u32, height: u32 },          //   struct case → synthetic record
    Dot,                                          //   unit case
}

#[plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Geo: Send + Sync {
    fn midpoint(&self, a: Point, b: Point) -> Point;
    fn describe(&self, s: Shape) -> String;
}
```

A proc-macro can't see the *definitions* of `Point`/`Shape` (it only sees the
method signatures), so the WIT for them is generated from your source by a tiny
`build.rs`:

```rust
// build.rs
fn main() {
    fidius_build::emit_wit();
}
```

```toml
# Cargo.toml
[build-dependencies]
fidius-build = "0.3"
```

On `cargo build`, `emit_wit()` parses your `src/lib.rs`, regenerates
`wit/<interface>.wit` (with the `record`/`variant` definitions) and the
generated↔your-type conversions, and `#[plugin_impl]` wires them in. Nothing else
changes — the build, packaging, signing, and loading steps above are identical.
Your types cross the wire as **real WIT records/variants**, so a
[Python guest](../how-to/wasm-python-plugin.md) sees them as native types too.

!!! note "Shapes & limits"
    Records need named fields. Enum cases may be unit, single-field, or
    struct-style (`Case { .. }`); a **multi-field tuple** case (`Case(A, B)`) is
    rejected — use a struct case, since a tuple serializes as a sequence and
    can't round-trip as a WIT record. Types may be in submodules of the crate. A
    type fidius can't map is a clear compile error on the wasm build. (`cargo`
    users can also run `fidius wit` to regenerate `wit/` manually.)

## What you built

A single Rust crate that compiles straight to a signed, sandboxed WASM component
loadable by any fidius host — no hand-written WIT, no glue. The same WIT contract
can be implemented in other languages: continue to
[A WASM Plugin in Python](../how-to/wasm-python-plugin.md), or read
[Capabilities & the WASM Sandbox](../explanation/wasm-capabilities.md) to grant
the component controlled access to the outside world.

## See also

- [Set Up the WASM Component Toolchain](../how-to/wasm-component-toolchain.md)
- [WASM Component ABI](../explanation/wasm-component-abi.md) — the WIT mapping
- [Capabilities & the WASM Sandbox](../explanation/wasm-capabilities.md)
- ADR FIDIUS-A-0003 — why the Component Model (Path B), and the 0.3.0 rebuild note
