<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# How to add async methods to a plugin interface

This guide shows how to declare and implement `async fn` methods in a Fidius
plugin trait. The generated FFI shims remain synchronous (`extern "C"`), but
internally call `block_on` on a per-dylib tokio runtime so your implementation
code can use `.await`.

## Prerequisites

- A working Fidius interface crate and plugin crate (see
  [How to scaffold a project](scaffold-project.md))
- Familiarity with Rust `async`/`await`

## 1. Enable the `async` feature on `fidius-core`

> **Note:** Only the [tokio](https://tokio.rs/) runtime is supported. Other
> async runtimes (async-std, smol, etc.) are not compatible with the generated
> shims.

The interface crate does not need any changes for async -- `async fn` methods
in the trait are handled entirely by the proc macro. Only the plugin crate
needs the feature enabled.

In your **plugin crate's** `Cargo.toml`, enable the feature:

```toml
[dependencies]
fidius-core = { version = "0.1", features = ["async"] }
tokio = { version = "1", features = ["full"] }
```

The `async` feature gates the `fidius_core::async_runtime` module and pulls in
`tokio` as a dependency.

## 2. Declare `async fn` in the interface trait

In your interface crate, mark any method as `async fn`. All other attributes
work the same way:

```rust
#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait AsyncProcessor: Send + Sync {
    async fn process(&self, input: String) -> String;
}
```

The `#[plugin_interface]` macro generates the same `#[repr(C)]` vtable struct
(`AsyncProcessor_VTable`) regardless of whether methods are sync or async. The
FFI function pointer signature is always:

```rust
unsafe extern "C" fn(*const u8, u32, *mut *mut u8, *mut u32) -> i32
```

## 3. Implement async methods in the plugin

In your plugin crate, write the `impl` block with `async fn` just as you would
in normal Rust:

```rust
use my_interface::{plugin_impl, AsyncProcessor};

pub struct MyProcessor;

#[plugin_impl(AsyncProcessor)]
impl AsyncProcessor for MyProcessor {
    async fn process(&self, input: String) -> String {
        // You can .await here freely
        format!("processed: {}", input)
    }
}

fidius_core::fidius_plugin_registry!();
```

## 4. Generated shim and runtime

The macro generates a synchronous FFI shim that calls `block_on` on a per-dylib tokio runtime -- see the [ABI specification](../reference/abi-specification.md) for the generated code and the [async runtime explanation](../explanation/async-runtime.md) for runtime lifecycle details.

## Things to keep in mind

- The FFI boundary is always synchronous. The host calls `extern "C"` functions
  and blocks until they return. Async is an implementation detail inside the
  plugin.
- You can mix sync and async methods in the same trait. Each method's shim is
  generated independently.
- If your async method needs to spawn tasks, they run on the dylib's
  `FIDIUS_RUNTIME` and must complete before the shim returns (since `block_on`
  waits for the future to resolve).

## See also

- [How to scaffold a project](scaffold-project.md) -- generate interface and
  plugin crates with the CLI
- [How to ship multiple plugins per dylib](multiple-plugins-per-dylib.md) --
  all plugins in a dylib share one async runtime
- [How to inspect a plugin](inspect-plugin.md) -- verify your compiled plugin
  metadata
