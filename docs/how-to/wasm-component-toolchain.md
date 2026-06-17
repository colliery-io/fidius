<!-- Copyright 2026 Colliery, Inc. Licensed under Apache 2.0 -->

# How to Set Up the WASM Component Toolchain

The WASM execution backend (FIDIUS-I-0021) builds plugins as **WebAssembly
components** (Component Model + WIT). Working on that backend — or building a
WASM plugin — needs three things on top of the standard Rust toolchain. The
host runtime (`wasmtime`) is an ordinary Cargo dependency and needs no special
setup; this guide covers only the *build* tools.

> You do **not** need this to work on the cdylib or Python backends. It is only
> required for the WASM component work.

## Prerequisites

The main repo has no Flox/devbox environment — it uses your ambient
`rustup`/`cargo`. The tools below install globally into `~/.cargo/bin`.

## Install

```bash
# 1. The component build target
rustup target add wasm32-wasip2

# 2. The component build tools
cargo install cargo-component wasm-tools
```

### Verified versions

These are the versions this backend was developed and verified against. Newer
releases generally work, but the Component Model tooling moves fast — if a
component build or validation behaves unexpectedly, check against these first.

| Tool | Verified version |
|------|------------------|
| `wasm32-wasip2` target | rustc 1.93+ |
| `cargo-component` | 0.21.1 |
| `wasm-tools` | 1.252.0 |

## Verify it works

```bash
cargo component --version     # -> cargo-component-component 0.21.1
wasm-tools --version          # -> wasm-tools 1.252.0

# Build + validate a throwaway component end-to-end:
cargo component new --lib /tmp/cc-smoke
cd /tmp/cc-smoke && cargo component build
wasm-tools validate --features component-model target/wasm32-wasip1/debug/cc_smoke.wasm
wasm-tools component wit target/wasm32-wasip1/debug/cc_smoke.wasm   # prints the WIT
```

If `wasm-tools validate` prints nothing and exits 0, the artifact is a valid
component.

> **Note for the capability model (Phase 2):** a default `cargo component new`
> imports a stack of `wasi:cli` / `wasi:io` interfaces. fidius's connectors are
> sandboxed **deny-by-default**, so the WASM executor will instantiate with an
> empty `Linker` and grant only the WASI imports a plugin explicitly declares.
> Expect to strip the default imports when authoring fidius components.

## CI

A dedicated **`wasm`** job in `.github/workflows/ci.yml` (separate from the main
`check`/`test` matrix, so the cdylib/Python pipeline is unaffected by
component-tooling install time) installs the `wasm32-wasip2` target plus pinned
`cargo-component`, `wasm-tools`, and `componentize-py` (the polyglot guest
toolchain), builds the fixtures, and runs `cargo test -p fidius-host --features wasm`.

Run the WASM-backend tests locally the same way:

```bash
# build the Rust + Python greeter components, then test
(cd tests/wasm-fixtures/greeter && cargo component build --release)
tests/wasm-fixtures/greeter-py/build.sh          # needs componentize-py
cargo test -p fidius-host --features wasm
```

The `polyglot_python_guest_behaves_identically` test skips cleanly (does not
fail) if `greeter_py.wasm` hasn't been built, so the suite still runs where
`componentize-py` isn't installed.
