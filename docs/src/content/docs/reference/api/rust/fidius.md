---
title: "fidius"
description: "Fidius — a Rust plugin framework for trait-to-dylib plugin systems."
editUrl: false
sidebar:
  order: 1
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Fidius — a Rust plugin framework for trait-to-dylib plugin systems.

This is the facade crate. Interface crates should depend on `fidius` only.
It re-exports everything needed to define interfaces and implement plugins.

**For interface crate authors**

```rust
pub use fidius::{plugin_impl, PluginError};

#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait MyPlugin: Send + Sync {
    fn process(&self, input: String) -> String;
}
```

**For plugin crate authors**

```rust
use my_interface::{plugin_impl, MyPlugin, PluginError};

pub struct MyImpl;

#[plugin_impl(MyPlugin)]
impl MyPlugin for MyImpl {
    fn process(&self, input: String) -> String {
        format!("processed: {input}")
    }
}

fidius::fidius_plugin_registry!();
```

**What fidius does *not* provide: timeouts and cancellation**

Fidius has **no built-in timeout, deadline, or cancellation mechanism**
for plugin method calls. A call to `PluginHandle::call_method` (or
`call_method_raw`) runs to completion, panics, or — in the case of a
truly stuck plugin — never returns. There is no `PluginError::Timeout`
variant and the framework will not interrupt a misbehaving plugin.

This is a deliberate consequence of the cdylib + in-process architecture:
plugin code runs synchronously on the host's calling thread, and Rust
cannot safely interrupt a thread mid-FFI-call. Any honest timeout
implementation requires running the plugin in a separate, killable
process — out of scope for the current framework.

**If your threat model includes runaway plugins, you must add a
watchdog yourself.** The usual pattern is to run the host process
itself with a supervisor that can SIGKILL it on deadline; per-call
timeouts inside a single host process are not safely achievable for
arbitrary plugin code.

Future work: the `fidius-python` initiative is the natural carrier for
a real subprocess-isolated execution tier, which would be the only path
by which fidius could grow first-class timeout semantics.

## Re-exports

- [`host_interface`](../../rust/fidius-macro/host_interface/)
- [`plugin_impl`](../../rust/fidius-macro/#fn.plugin_impl)
- [`plugin_interface`](../../rust/fidius-macro/#fn.plugin_interface)
- [`descriptor`](../../rust/fidius-guest/descriptor/)
- [`error`](../../rust/fidius-guest/error/)
- [`hash`](../../rust/fidius-guest/hash/)
- [`host_ffi`](../../rust/fidius-guest/host_ffi/)
- [`host_registry`](../../rust/fidius-core/host_registry/)
- [`python_descriptor`](../../rust/fidius-guest/python_descriptor/)
- [`status`](../../rust/fidius-guest/status/)
- [`stream_ffi`](../../rust/fidius-guest/stream_ffi/)
- [`stream_marker`](../../rust/fidius-guest/stream_marker/)
- [`wasm_descriptor`](../../rust/fidius-guest/wasm_descriptor/)
- [`wire`](../../rust/fidius-guest/wire/)
- [`http`](../../rust/fidius-guest/http/)
- [`host_call`](../../rust/fidius-guest/host_call/)
- [`sockets`](../../rust/fidius-guest/sockets/)
- [`BufferStrategyKind`](../../rust/fidius-guest/descriptor/#enum.BufferStrategyKind)
- [`PluginDescriptor`](../../rust/fidius-guest/descriptor/#struct.PluginDescriptor)
- [`PluginRegistry`](../../rust/fidius-guest/descriptor/#struct.PluginRegistry)
- [`ABI_VERSION`](../../rust/fidius-guest/descriptor/#const.ABI_VERSION)
- [`FIDIUS_MAGIC`](../../rust/fidius-guest/descriptor/#const.FIDIUS_MAGIC)
- [`REGISTRY_VERSION`](../../rust/fidius-guest/descriptor/#const.REGISTRY_VERSION)
- [`PluginError`](../../rust/fidius-guest/error/#struct.PluginError)
- [`fnv1a`](../../rust/fidius-guest/hash/#fn.fnv1a)
- [`interface_hash`](../../rust/fidius-guest/hash/#fn.interface_hash)
- [`HostBindError`](../../rust/fidius-guest/host_ffi/#struct.HostBindError)
- [`HostCallError`](../../rust/fidius-guest/host_ffi/#enum.HostCallError)
- [`Stream`](../../rust/fidius-guest/stream_marker/#struct.Stream)
- [`from_value`](../../rust/fidius-guest/value/#fn.from_value)
- [`to_value`](../../rust/fidius-guest/value/#fn.to_value)
- [`Value`](../../rust/fidius-guest/value/#enum.Value)
- [`ValueError`](../../rust/fidius-guest/value/#struct.ValueError)
- [`async_runtime`](../../rust/fidius-core/async_runtime/)
- [`CallError`](../../rust/fidius-host/error/#enum.CallError)
- [`HostImportInfo`](../../rust/fidius-host/host_import/#struct.HostImportInfo)
- [`LoadError`](../../rust/fidius-host/error/#enum.LoadError)
- [`LoadPolicy`](../../rust/fidius-host/types/#enum.LoadPolicy)
- [`LoadedLibrary`](../../rust/fidius-host/loader/#struct.LoadedLibrary)
- [`LoadedPlugin`](../../rust/fidius-host/loader/#struct.LoadedPlugin)
- [`PluginExecutor`](../../rust/fidius-host/executor/#trait.PluginExecutor)
- [`PluginHandle`](../../rust/fidius-host/handle/#struct.PluginHandle)
- [`PluginHost`](../../rust/fidius-host/host/#struct.PluginHost)
- [`PluginHostBuilder`](../../rust/fidius-host/host/#struct.PluginHostBuilder)
- [`PluginInfo`](../../rust/fidius-host/types/#struct.PluginInfo)
- [`PluginRuntimeKind`](../../rust/fidius-host/types/#enum.PluginRuntimeKind)
- [`host_import`](../../rust/fidius-host/host_import/)
- [`ChunkStream`](../../rust/fidius-host/stream/#struct.ChunkStream)
- [`StreamExecutor`](../../rust/fidius-host/stream/#trait.StreamExecutor)
- `http_types` (`fidius_host::http_types`)
- [`EgressDenied`](../../rust/fidius-host/executor/wasm/#struct.EgressDenied)
- [`EgressPolicy`](../../rust/fidius-host/executor/wasm/#trait.EgressPolicy)
- [`ResponseDirective`](../../rust/fidius-host/executor/wasm/#enum.ResponseDirective)
- [`TcpTarget`](../../rust/fidius-host/executor/wasm/#struct.TcpTarget)
- `inventory` (`fidius_core::inventory`)
- [`registry`](../../rust/fidius-core/registry/)
- `fidius_plugin_registry` (`fidius_core::fidius_plugin_registry`)
