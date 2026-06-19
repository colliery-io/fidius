<!-- Copyright 2026 Colliery, Inc. Licensed under Apache 2.0 -->

<p align="center">
  <img src="docs/assets/logo.png" alt="fidius" width="220" />
</p>

# fidius

**A Rust plugin framework for trait-to-dylib plugin systems**

fidius lets you define a Rust trait, annotate it with a macro, and get a compiled dynamic library with a stable C ABI. Host applications load, validate, and call plugins through a type-safe proxy — no handwritten FFI.

## Features

- **Trait-driven** — define a plugin interface as a Rust trait; the macro generates the ABI shim.
- **Stable C ABI** — plugins compile to `.dylib`/`.so`/`.dll` with versioned, hash-checked entry points.
- **Type-safe host loading** — `fidius-host` loads plugins behind a typed proxy that mirrors your trait.
- **Optional methods & interface evolution** — add methods without breaking existing plugins.
- **Signing & verification** — Ed25519 signatures over plugin artifacts.
- **Python plugins** — write plugins in Python that satisfy a Rust trait via `fidius-python`.
- **Sandboxed WASM plugins** — compile a plugin to a WebAssembly component that runs in a deny-all wasmtime sandbox with a capability allow-list; polyglot (Rust *and* other languages implement the same interface). Outbound HTTP is host-brokered and policy-gated (`wasi:http` + a required egress hook).
- **Server-streaming** — a method can return `fidius::Stream<T>`: pull-based, backpressured, drop-to-cancel, implemented natively on all three backends (Rust/Python/WASM) and proven across Rust, JS, Python, and C guests.
- **CLI tooling** — scaffold interfaces and plugins, sign, inspect, and package.

## Workspace Layout

| Crate | Purpose |
|---|---|
| `fidius` | Top-level facade re-exporting the public API |
| `fidius-core` | Descriptors, wire format, hashing, registry, metadata |
| `fidius-macro` | Proc macros (`#[fidius::interface]`, `#[fidius::plugin]`) and IR |
| `fidius-host` | Loading, calling, signing, arch detection, arena pool |
| `fidius-cli` | `fidius` command-line tool |
| `fidius-test` | Test helpers (dylib fixtures, signing fixtures) |
| `fidius-python` | Python plugin support |
| `fidius-guest` | wasm-buildable guest types for plugins compiled to WASM components |

## Installation

```bash
cargo install fidius-cli
```

## Quick Example

A fidius plugin system is three crates: an **interface** (the trait), a **plugin** (a cdylib that implements it), and a **host** (loads and calls it). The CLI scaffolds the first two:

```bash
fidius init-interface my-api    --trait ImageFilter
fidius init-plugin    my-plugin --interface my-api --trait ImageFilter
```

You then fill in two small pieces of code. **In the interface crate**, annotate the trait — the macro generates the vtable, ABI hash, and the host-side typed proxy:

```rust
use serde::{Deserialize, Serialize};

#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait ImageFilter: Send + Sync {
    fn apply(&self, input: ApplyInput) -> ApplyOutput;
}

#[derive(Serialize, Deserialize)]
pub struct ApplyInput  { pub pixels: Vec<u8> }
#[derive(Serialize, Deserialize)]
pub struct ApplyOutput { pub pixels: Vec<u8> }
```

**In the plugin crate**, annotate your `impl` and emit the registry — that's all the FFI you write:

```rust
use my_api::{plugin_impl, ImageFilter, ApplyInput, ApplyOutput};

pub struct Invert;

#[plugin_impl(ImageFilter)]
impl ImageFilter for Invert {
    fn apply(&self, input: ApplyInput) -> ApplyOutput {
        ApplyOutput { pixels: input.pixels.into_iter().map(|p| 255 - p).collect() }
    }
}

fidius::fidius_plugin_registry!();
```

Then build, optionally sign, and inspect:

```bash
cd my-plugin && cargo build

fidius keygen --out mykey
fidius sign --key mykey.secret target/debug/libmy_plugin.dylib
fidius inspect target/debug/libmy_plugin.dylib
```

The host loads the resulting dylib through `fidius-host` and calls `apply()` through a generated `ImageFilterClient` proxy. See [Your First Plugin](docs/tutorials/your-first-plugin.md) for the full walkthrough including the host crate.

## Development

This project uses [angreal](https://github.com/angreal/angreal) as its task runner. Common tasks:

```bash
angreal tree            # list all tasks
angreal build           # build the workspace
angreal test            # run the test suite
angreal python-test     # run the Python SDK tests
angreal check           # cargo check + clippy
angreal lint            # formatting and lint checks
angreal license-header  # add/check license headers
```

## Documentation

Full documentation lives in [`docs/`](docs/index.md) and covers tutorials, how-to guides, reference, and architecture explanation. Build it locally with `mkdocs serve`.

- [Your First Plugin](docs/tutorials/your-first-plugin.md)
- [Your First Python Plugin](docs/tutorials/python-plugin.md)
- [Your First WASM Plugin](docs/tutorials/your-first-wasm-plugin.md)
- [Capabilities & the WASM Sandbox](docs/explanation/wasm-capabilities.md)
- [Architecture Overview](docs/explanation/architecture.md)
- [ABI Specification](docs/reference/abi-specification.md)
- [CLI Reference](docs/reference/cli.md)

## License

Apache-2.0. See [LICENSE](LICENSE).
