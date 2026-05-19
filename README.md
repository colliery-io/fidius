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

## Installation

```bash
cargo install fidius-cli
```

## Quick Example

```bash
# Scaffold an interface and a plugin
fidius init-interface my-api --trait ImageFilter
fidius init-plugin my-plugin --interface my-api --trait ImageFilter

# Build the plugin
cd my-plugin && cargo build

# Sign it (optional)
fidius keygen --out mykey
fidius sign --key mykey.secret target/debug/libmy_plugin.dylib

# Inspect the compiled plugin
fidius inspect target/debug/libmy_plugin.dylib
```

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
- [Architecture Overview](docs/explanation/architecture.md)
- [ABI Specification](docs/reference/abi-specification.md)
- [CLI Reference](docs/reference/cli.md)

## License

Apache-2.0. See [LICENSE](LICENSE).
