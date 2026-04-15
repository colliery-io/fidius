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

# Your First Fidius Plugin

Fidius is a Rust framework for building safe, version-checked plugin systems using dynamic libraries and procedural macros.

In this tutorial you will build a complete plugin system from scratch: an
interface crate that defines a `Calculator` trait, a plugin crate that
implements it as a cdylib (a C-compatible dynamic library that Cargo compiles to `.dylib`, `.so`, or `.dll`), and a host binary that loads the plugin at runtime
and calls its `add` method.

By the end you will have a working example where the host calls `add(3, 7)` on
a dynamically loaded plugin and gets back `10`.

## Prerequisites

- Rust toolchain (1.77+ recommended)
- `cargo` on your `PATH`
- The `fidius` CLI installed (`cargo install fidius-cli`), or you can create
  files manually as shown below

## What you will build

```
calculator-workspace/
  calculator-interface/   # defines the Calculator trait
  calculator-plugin/      # implements Calculator as a cdylib
  calculator-host/        # loads the plugin and calls add()
```

## Step 1: Create a workspace

Create a directory and a top-level `Cargo.toml`:

```bash
mkdir calculator-workspace && cd calculator-workspace
```

```toml
# Cargo.toml
[workspace]
resolver = "2"
members = [
    "calculator-interface",
    "calculator-plugin",
    "calculator-host",
]
```

## Step 2: Create the interface crate

The interface crate defines the trait that plugins implement. You can scaffold
it with the CLI:

```bash
fidius init-interface calculator-interface --trait Calculator
```

This creates `calculator-interface/` with a `Cargo.toml` and `src/lib.rs`. The
generated code is a starting point; replace the contents of `src/lib.rs` with
the definition below.

Alternatively, create the crate manually:

```bash
mkdir -p calculator-interface/src
```

### calculator-interface/Cargo.toml

```toml
[package]
name = "calculator-interface"
version = "0.1.0"
edition = "2021"

[dependencies]
fidius = "0.1"
serde = { version = "1", features = ["derive"] }
```

### calculator-interface/src/lib.rs

```rust
pub use fidius::{plugin_impl, PluginError};

#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Calculator: Send + Sync {
    fn add(&self, input: AddInput) -> AddOutput;
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AddInput {
    pub a: i64,
    pub b: i64,
}

#[derive(Serialize, Deserialize)]
pub struct AddOutput {
    pub result: i64,
}
```

Key points:

- `#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]`
  annotates the trait. `version` is a user-chosen integer you bump when the
  interface changes. `buffer = PluginAllocated` means the plugin allocates the
  output buffer (the only strategy currently supported).
- The trait requires `Send + Sync`, methods take `&self`, and all argument/return types must implement Serde's `Serialize + Deserialize` -- see the [ABI specification](../reference/abi-specification.md) for the full requirements.
- The crate re-exports `fidius::plugin_impl` and `fidius::PluginError` so
  plugin crates only need to depend on the interface crate.

The `#[plugin_interface]` macro generates a vtable, interface hash, and descriptor builder behind the scenes -- see the [ABI specification](../reference/abi-specification.md) for the full list.

## Step 3: Create the plugin crate

The plugin crate implements the interface as a `cdylib` shared library. You can
scaffold it with the CLI:

```bash
fidius init-plugin calculator-plugin \
    --interface ../calculator-interface \
    --trait Calculator
```

Or create it manually:

```bash
mkdir -p calculator-plugin/src
```

### calculator-plugin/Cargo.toml

The critical line is `crate-type = ["cdylib"]` -- this tells Cargo to produce a
`.dylib` / `.so` / `.dll` instead of an `.rlib`.

```toml
[package]
name = "calculator-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
calculator-interface = { path = "../calculator-interface" }
fidius-core = { version = "0.1" }
serde = { version = "1", features = ["derive"] }
```

### calculator-plugin/src/lib.rs

```rust
use calculator_interface::{plugin_impl, Calculator, AddInput, AddOutput};

pub struct BasicCalculator;

#[plugin_impl(Calculator)]
impl Calculator for BasicCalculator {
    fn add(&self, input: AddInput) -> AddOutput {
        AddOutput {
            result: input.a + input.b,
        }
    }
}

fidius_core::fidius_plugin_registry!();
```

Key points:

- `#[plugin_impl(Calculator)]` generates the FFI (Foreign Function Interface -- the mechanism for calling across language or binary boundaries) shims, a static vtable (a table of function pointers, one per method), and a
  `PluginDescriptor` for `BasicCalculator`. The attribute argument is the trait
  name -- it must match the trait annotated with `#[plugin_interface]`.
- `fidius_core::fidius_plugin_registry!()` emits the `fidius_get_registry`
  export symbol that the host looks up at runtime. Call it exactly once per
  cdylib crate. It collects all `#[plugin_impl]` descriptors in the crate
  (you can have multiple plugins in one dylib).

For details on what the macro generates, see the [reference documentation](../reference/abi-specification.md).

## Step 4: Create the host binary

The host binary uses `fidius-host` to discover, load, and call the plugin.

```bash
mkdir -p calculator-host/src
```

### calculator-host/Cargo.toml

```toml
[package]
name = "calculator-host"
version = "0.1.0"
edition = "2021"

[dependencies]
fidius-host = { version = "0.1" }
serde = { version = "1", features = ["derive"] }
```

### calculator-host/src/main.rs

```rust
use fidius_host::{PluginHost, PluginHandle};
use serde::{Deserialize, Serialize};

// The host defines its own copies of the input/output types rather than
// importing them from the interface crate. This keeps the host decoupled
// from the plugin's compile-time dependencies -- the host only needs
// fidius-host, not fidius-core or the proc macros. The structs must have
// the same field names and types (they are serialized over the wire as
// JSON or bincode).

#[derive(Serialize)]
struct AddInput {
    a: i64,
    b: i64,
}

#[derive(Deserialize, Debug)]
struct AddOutput {
    result: i64,
}

// Method indices are zero-based, in declaration order.
const ADD_METHOD: usize = 0;

fn main() {
    // Point the host at the directory containing the compiled cdylib.
    // After `cargo build`, this is typically target/debug/.
    let plugin_dir = std::env::args()
        .nth(1)
        .expect("usage: calculator-host <plugin-dir>");

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .build()
        .expect("failed to build plugin host");

    // Load the plugin by its struct name.
    let loaded = host
        .load("BasicCalculator")
        .expect("failed to load BasicCalculator");

    println!("Loaded plugin: {}", loaded.info.name);
    println!("  Interface: {}", loaded.info.interface_name);
    println!("  Version: {}", loaded.info.interface_version);

    // Wrap in a PluginHandle for type-safe method calls.
    let handle = PluginHandle::from_loaded(loaded);

    // Call add with a=3, b=7.
    // Arguments are tuple-encoded: single arg wraps in (T,).
    let input = (AddInput { a: 3, b: 7 },);
    let output: AddOutput = handle
        .call_method(ADD_METHOD, &input)
        .expect("add() call failed");

    println!("add(3, 7) = {}", output.result);
    assert_eq!(output.result, 10);
}
```

Key points:

- `handle.call_method(ADD_METHOD, &input)` calls the method at
  its vtable index (zero-based, declaration order).
- Arguments are always **tuple-encoded**: single arg is `(T,)`, two args
  is `(A, B)`, zero args is `()`. This matches how the `#[plugin_impl]`
  macro deserializes input on the plugin side.

See the [host API reference](../api/rust/fidius-host.md) for the full builder and handle API.

## Step 5: Build and run

```bash
# From the workspace root:
cargo build

# Run the host, pointing it at the directory containing the plugin dylib.
# On macOS the dylib lands in target/debug/ as libcalculator_plugin.dylib.
# On Linux it is libcalculator_plugin.so, on Windows calculator_plugin.dll.
cargo run --bin calculator-host -- target/debug/
```

Expected output:

```
Loaded plugin: BasicCalculator
  Interface: Calculator
  Version: 1
add(3, 7) = 10
```

## Step 6: Discover plugins

Instead of loading a plugin by name, you can discover all plugins in a
directory:

```rust
let plugins = host.discover().expect("discovery failed");
for info in &plugins {
    println!("Found: {} (implements {})", info.name, info.interface_name);
}
```

`discover()` returns a `Vec<PluginInfo>` containing metadata (name, interface
name, interface hash, version, capabilities, wire format, buffer strategy) for
every valid plugin found.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `PluginNotFound` | Plugin name doesn't match | The name is the Rust struct name (`BasicCalculator`), not the crate name |
| `SymbolNotFound` | Missing registry export | Ensure `fidius_core::fidius_plugin_registry!()` is called in the plugin's `lib.rs` |
| `InvalidMagic` | Corrupt or non-fidius dylib | Check that the dylib was built from a fidius plugin crate |
| Deserialization error | Mismatched struct fields | Input/output structs in host must have identical field names and types to the interface crate |

## Next steps

- [Optional Methods](optional-methods.md) -- extend the Calculator with
  version-gated methods
- [Signing Plugins](signing-plugins.md) -- sign and verify plugins with
  Ed25519 keys
