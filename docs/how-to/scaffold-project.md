<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# How to scaffold a new plugin project with the CLI

This guide shows how to use `fidius init-interface` and `fidius init-plugin` to
generate the boilerplate for a new plugin project.

## Prerequisites

- The `fidius` CLI installed

## 1. Scaffold an interface crate

```
$ fidius init-interface my-api --trait MyTrait
Created interface crate: ./my-api
```

This creates:

```
my-api/
  Cargo.toml
  src/
    lib.rs
```

### Generated `Cargo.toml`

```toml
[package]
name = "my-api"
version = "0.1.0"
edition = "2021"

[dependencies]
fidius = "0.3.2"
```

The `fidius` dependency version is resolved automatically (see
[Dependency resolution](#dependency-resolution) below).

### Generated `src/lib.rs`

```rust
pub use fidius::{plugin_impl, PluginError};

#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait MyTrait: Send + Sync {
    fn process(&self, input: String) -> String;
}
```

The generated trait gives you a working starting point. Edit the methods to
match your actual interface.

## 2. Scaffold a plugin crate

```
$ fidius init-plugin my-plugin --interface my-api --trait MyTrait
Created plugin crate: ./my-plugin
```

This creates:

```
my-plugin/
  Cargo.toml
  src/
    lib.rs
```

### Generated `Cargo.toml`

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
my-api = { path = "my-api" }
fidius-core = { version = "0.1" }
```

Note that `crate-type = ["cdylib"]` is set automatically -- this is required
for Fidius plugins.

### Generated `src/lib.rs`

```rust
use my_api::{plugin_impl, MyTrait, PluginError};

pub struct MyMyTrait;

#[plugin_impl(MyTrait)]
impl MyTrait for MyMyTrait {
    fn process(&self, input: String) -> String {
        todo!("implement MyTrait")
    }
}

fidius_core::fidius_plugin_registry!();
```

The struct name is derived as `My{TraitName}`. The `fidius_plugin_registry!()`
call at the bottom emits the `fidius_get_registry` export that the host uses to
discover the plugin.

## 3. Dependency resolution

Both commands automatically resolve the `fidius` dependency by checking local paths first, then crates.io -- see the [CLI reference](../reference/cli.md) for the full resolution strategy and the `--version` flag.

## 4. Control the output directory with `--path`

By default both commands create the crate directory inside the current working
directory. Use `--path` to place it elsewhere:

```
$ fidius init-interface my-api --trait MyTrait --path /tmp/workspace
Created interface crate: /tmp/workspace/my-api
```

## 5. Error handling

Both commands refuse to overwrite an existing directory:

```
$ fidius init-interface my-api --trait MyTrait
Created interface crate: ./my-api

$ fidius init-interface my-api --trait MyTrait
error: directory './my-api' already exists
```

## 6. Typical workflow

```bash
# 1. Create the interface
fidius init-interface image-filter --trait ImageFilter

# 2. Edit the trait to define your actual methods
$EDITOR image-filter/src/lib.rs

# 3. Create the plugin, pointing at the local interface crate
fidius init-plugin blur-filter --interface ./image-filter --trait ImageFilter

# 4. Implement the trait
$EDITOR blur-filter/src/lib.rs

# 5. Build
cd blur-filter && cargo build --release

# 6. Inspect the result
fidius inspect target/release/libblur_filter.dylib
```

For the full CLI argument reference, see [CLI reference](../reference/cli.md).

## See also

- [How to add async methods](async-methods.md) -- add `async fn` to the
  generated trait
- [How to ship multiple plugins per dylib](multiple-plugins-per-dylib.md) --
  add more `#[plugin_impl]` blocks to the generated plugin
- [How to inspect a plugin](inspect-plugin.md) -- verify the built dylib
