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

# Optional Methods

This tutorial extends the Calculator plugin from
[Your First Plugin](your-first-plugin.md) with an optional `multiply` method.
You will learn how to evolve an interface without breaking existing plugins, how
the capability bitfield works, and how the host checks whether a loaded plugin
supports an optional method before calling it.

## Prerequisites

- Completed [Your First Plugin](your-first-plugin.md)
- A working `calculator-workspace` with interface, plugin, and host crates

## What you will learn

1. Declare optional methods with `#[optional(since = N)]`
2. Implement them in a plugin
3. Check capabilities from the host before calling
4. Understand what happens when an older plugin lacks the optional method

## Step 1: Add the optional method to the interface

Open `calculator-interface/src/lib.rs` and add a `multiply` method annotated
with `#[optional(since = 2)]`. Also add the necessary input/output types:

```rust
pub use fidius::{plugin_impl, PluginError};

#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Calculator: Send + Sync {
    fn add(&self, input: AddInput) -> AddOutput;

    #[optional(since = 2)]
    fn multiply(&self, input: MulInput) -> MulOutput;
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

#[derive(Serialize, Deserialize)]
pub struct MulInput {
    pub a: i64,
    pub b: i64,
}

#[derive(Serialize, Deserialize)]
pub struct MulOutput {
    pub result: i64,
}
```

Key points about `#[optional(since = 2)]`:

- The `since` value is informational -- it documents which interface `version`
  (from `#[plugin_interface(version = N, ...)]`) introduced the method. In this
  example, `version = 1` is the original interface and `since = 2` means
  "added in version 2." The `since` value does not change the interface hash
  (only required methods contribute to the hash).
- Optional methods do not break backward compatibility. A plugin compiled
  against the old interface (without `multiply`) will still load and work for
  `add` calls.
- You can have up to 64 optional methods per interface (they are tracked in a
  `u64` capability bitfield).

The macro generates capability-bit constants and wraps optional vtable entries in `Option` -- see the [ABI specification](../reference/abi-specification.md) for details.

## Step 2: Implement the optional method in the plugin

Open `calculator-plugin/src/lib.rs` and add the `multiply` implementation:

```rust
use calculator_interface::{
    plugin_impl, Calculator,
    AddInput, AddOutput,
    MulInput, MulOutput,
};

pub struct BasicCalculator;

#[plugin_impl(Calculator)]
impl Calculator for BasicCalculator {
    fn add(&self, input: AddInput) -> AddOutput {
        AddOutput {
            result: input.a + input.b,
        }
    }

    fn multiply(&self, input: MulInput) -> MulOutput {
        MulOutput {
            result: input.a * input.b,
        }
    }
}

fidius_core::fidius_plugin_registry!();
```

When `#[plugin_impl(Calculator)]` sees that the impl block includes `multiply`,
and `multiply` appears in `Calculator_OPTIONAL_METHODS`, it sets the
corresponding capability bit in the plugin descriptor. In this case, capability
bit 0 is set, so the plugin's capabilities field becomes `0x1`.

## Step 3: Check capabilities from the host

Open `calculator-host/src/main.rs` and update it to check for `multiply`
support before calling:

```rust
use fidius_host::{PluginHost, PluginHandle};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct AddInput { a: i64, b: i64 }

#[derive(Deserialize, Debug)]
struct AddOutput { result: i64 }

#[derive(Serialize)]
struct MulInput { a: i64, b: i64 }

#[derive(Deserialize, Debug)]
struct MulOutput { result: i64 }

// Method indices (zero-based, declaration order across all methods).
const ADD_METHOD: usize = 0;
const MULTIPLY_METHOD: usize = 1;

// Capability bit indices (zero-based among optional methods only).
const MULTIPLY_CAP: u32 = 0;

fn main() {
    let plugin_dir = std::env::args()
        .nth(1)
        .expect("usage: calculator-host <plugin-dir>");

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .build()
        .expect("failed to build plugin host");

    let loaded = host
        .load("BasicCalculator")
        .expect("failed to load BasicCalculator");

    let handle = PluginHandle::from_loaded(loaded);

    // Call add (always available -- required method).
    let sum: AddOutput = handle
        .call_method(ADD_METHOD, &AddInput { a: 3, b: 7 })
        .expect("add() failed");
    println!("add(3, 7) = {}", sum.result);

    // Check if multiply is supported before calling.
    if handle.has_capability(MULTIPLY_CAP) {
        let product: MulOutput = handle
            .call_method(MULTIPLY_METHOD, &MulInput { a: 4, b: 5 })
            .expect("multiply() failed");
        println!("multiply(4, 5) = {}", product.result);
    } else {
        println!("multiply is not supported by this plugin");
    }
}
```

`handle.has_capability(bit)` checks whether the given bit is set in the
plugin's capability bitfield. The `bit` argument must be less than 64. It
corresponds to the zero-based index among optional methods in declaration
order.

The method index passed to `call_method` counts all methods (both required and
optional) in declaration order. In this example:

| Method | Declaration index | Kind |
|---|---|---|
| `add` | 0 | required |
| `multiply` | 1 | optional (capability bit 0) |

## Step 4: Build and run

```bash
cargo build
cargo run --bin calculator-host -- target/debug/
```

Expected output:

```
add(3, 7) = 10
multiply(4, 5) = 20
```

## Step 5: Simulate an old plugin without multiply

To see what happens when a plugin does not implement the optional method,
create a second plugin crate that only implements `add`:

```bash
mkdir -p calculator-plugin-v1/src
```

### calculator-plugin-v1/Cargo.toml

```toml
[package]
name = "calculator-plugin-v1"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
calculator-interface = { path = "../calculator-interface" }
fidius-core = { version = "0.1" }
serde = { version = "1", features = ["derive"] }
```

### calculator-plugin-v1/src/lib.rs

Notice that `multiply` is **not** implemented. The Rust compiler does not
require it because the `#[plugin_interface]` macro treats optional methods as
having default implementations in the vtable:

```rust
use calculator_interface::{plugin_impl, Calculator, AddInput, AddOutput};

pub struct LegacyCalculator;

#[plugin_impl(Calculator)]
impl Calculator for LegacyCalculator {
    fn add(&self, input: AddInput) -> AddOutput {
        AddOutput {
            result: input.a + input.b,
        }
    }
}

fidius_core::fidius_plugin_registry!();
```

Add `"calculator-plugin-v1"` to the workspace members in the root
`Cargo.toml`:

```toml
# Cargo.toml
[workspace]
resolver = "2"
members = [
    "calculator-interface",
    "calculator-plugin",
    "calculator-plugin-v1",
    "calculator-host",
]
```

Rebuild, then run:

```bash
cargo build
cargo run --bin calculator-host -- target/debug/
```

If the host loads `LegacyCalculator` (change the name in the `host.load(...)`
call), the output will be:

```
add(3, 7) = 10
multiply is not supported by this plugin
```

The plugin loads without error. The capability bit 0 is `0` because
`LegacyCalculator` does not implement `multiply`. The host's
`has_capability(0)` check returns `false`, and the host skips the call.

## You can also inspect capabilities via the CLI

```bash
fidius inspect target/debug/libcalculator_plugin.dylib
```

Output includes a `Capabilities` hex value. For a plugin that implements
`multiply`, you will see `0x0000000000000001` (bit 0 set). For one that does
not, you will see `0x0000000000000000`.

## Interface evolution rules

For the full compatibility matrix (which changes are backward-compatible and which require recompilation), see [Interface Evolution](../explanation/interface-evolution.md).

## Next steps

- [Signing Plugins](signing-plugins.md) -- protect plugin integrity with
  Ed25519 signatures
- [Your First Plugin](your-first-plugin.md) -- review the basics
