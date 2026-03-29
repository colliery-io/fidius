<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# How to ship multiple plugin implementations in one cdylib

This guide shows how to put two or more `#[plugin_impl]` blocks in the same
Rust cdylib so a single `.dylib` / `.so` / `.dll` exposes multiple plugins to
the host.

## Prerequisites

- A Fidius interface crate with a `#[plugin_interface]` trait (see
  [How to scaffold a project](scaffold-project.md))
- Familiarity with `#[plugin_impl]` for a single plugin

## 1. Define the interface (once)

Define your interface trait in a shared crate as usual:

```rust
#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Greeter: Send + Sync {
    fn greet(&self, name: String) -> String;
}
```

## 2. Write multiple implementations in one `lib.rs`

In your plugin cdylib crate, implement the trait on as many structs as you
need. Apply `#[plugin_impl(TraitName)]` to each:

```rust
use my_interface::{plugin_impl, Greeter};

// --- Plugin 1 ---
pub struct HelloGreeter;

#[plugin_impl(Greeter)]
impl Greeter for HelloGreeter {
    fn greet(&self, name: String) -> String {
        format!("Hello, {}!", name)
    }
}

// --- Plugin 2 ---
pub struct GoodbyeGreeter;

#[plugin_impl(Greeter)]
impl Greeter for GoodbyeGreeter {
    fn greet(&self, name: String) -> String {
        format!("Goodbye, {}!", name)
    }
}

// Emit the combined registry (once, at the end)
fidius_core::fidius_plugin_registry!();
```

Each `#[plugin_impl]` generates:

- A static instance of the struct
- `extern "C"` shim functions for every method
- A `free_buffer` function
- A static `PluginDescriptor`
- An `inventory::submit!` call that registers the descriptor

## 3. Host-side discovery

On the host side, `PluginHost::discover()` scans search paths for dylib files,
loads each one, and iterates through all plugins in the registry:

```rust
use fidius_host::host::PluginHost;

let host = PluginHost::builder()
    .search_path("./plugins")
    .build()?;

let plugins = host.discover()?;
// plugins is Vec<PluginInfo> -- may contain multiple entries from one dylib

for info in &plugins {
    println!("{} implements {}", info.name, info.interface_name);
}
```

Each `PluginInfo` contains the plugin name, interface name, interface hash,
version, capabilities, wire format, and buffer strategy. Two plugins from the
same dylib will share the same interface hash and version but have different
names (e.g., `"HelloGreeter"` and `"GoodbyeGreeter"`).

To load a specific plugin by name:

```rust
let plugin = host.load("HelloGreeter")?;
```

## 4. Verify with `fidius inspect`

After building, confirm that both plugins are visible:

```
$ fidius inspect target/release/libmy_plugins.dylib

Plugin Registry: target/release/libmy_plugins.dylib
  Plugins: 2

  [0] HelloGreeter
      Interface: Greeter
      Interface hash: 0x...
      ...

  [1] GoodbyeGreeter
      Interface: Greeter
      Interface hash: 0x...
      ...
```

See [How to inspect a plugin](inspect-plugin.md) for full details on the
inspect output.

## Things to keep in mind

- You can implement **different** interface traits in the same dylib, not just
  multiple implementations of one trait. Each `#[plugin_impl]` is independent.
- Call `fidius_plugin_registry!()` exactly **once** per cdylib, typically at the
  bottom of `lib.rs`.
- The `plugin_count` field in the registry reflects the total number of
  `#[plugin_impl]` blocks, regardless of how many interfaces are involved.
- If the `async` feature is enabled, all plugins in the dylib share a single
  tokio runtime (see [How to add async methods](async-methods.md)).

## See also

- [How to scaffold a project](scaffold-project.md) -- generate the initial
  crate structure
- [How to add async methods](async-methods.md) -- async works the same way with
  multiple plugins
- [How to inspect a plugin](inspect-plugin.md) -- verify plugin count and
  metadata after building
