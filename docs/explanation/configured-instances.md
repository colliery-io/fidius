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

# Configured Plugin Instances

A plain fidius plugin is a **singleton**: the framework constructs one instance and
every method takes its arguments per call. Some plugins want the opposite — bind a
config **once**, then call methods that close over it. A REST connector configured
with `{url, page_size, credentials}`; a transform that compiles a schema at startup;
anything where the bound value is expensive to set up or wasteful to re-send. This
is Python's `functools.partial`, as a plugin primitive.

## Why not just pass config every call?

You can — and for a method called *once* (a connector whose `read()` returns a
stream), passing config as an argument is fine; it crosses the boundary once. But
when a method is called **many** times, a per-call config argument is re-marshaled
on every call, and the plugin re-does any config-bound setup every call. Configured
instances fix both: **config crosses the boundary once**, **setup runs once**, and
**N differently-configured instances coexist** in one host.

## Authoring (one shape, all backends)

Declare a config type on the `impl` and an inherent `configure` constructor:

```rust
#[derive(Serialize, Deserialize)]
pub struct Config { pub url: String, pub page_size: u32 }

#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Source: Send + Sync {
    fn read_page(&self, cursor: u32) -> Page;   // uses the bound config
}

pub struct MySource { cfg: Config /* + a client opened once */ }

#[fidius::plugin_impl(Source, config = Config)]
impl Source for MySource {
    fn read_page(&self, cursor: u32) -> Page { /* … self.cfg … */ }
}

impl MySource {
    fn configure(cfg: Config) -> Self { Self { cfg } }   // wired to `construct`
}
```

The trait stays object-safe (the constructor lives on the `impl`, not the trait). A
plugin with **no** `config =` is the singleton — internally "construct with `()`".

## Loading a configured instance (host)

| Backend | Host call |
| --- | --- |
| **cdylib** | `PluginHandle::configure_in_process::<C>(&desc, &config)` |
| **WASM** | `host.load_wasm_configured::<C>(name, &desc, &config)` |
| **Python** | `host.load_python_configured::<C>(name, &desc, &config)` |

```rust
let src = host.load_wasm_configured("rest-source", &Source_WASM_DESCRIPTOR,
                                    &Config { url, page_size: 100 })?;
let page: Page = src.call_method(READ_PAGE, &(0u32,))?;   // config already bound
```

The config is serialized once, handed to the plugin's constructor, and the instance
is retained; subsequent method calls dispatch on it without re-sending config.

## How it works per backend

- **cdylib** — the descriptor carries `construct(cfg) -> *mut instance` + `destroy`;
  the host constructs at load and passes the instance pointer to every vtable
  method, freeing it on drop. (The singleton is `construct(())`.) This is an ABI
  change — `ABI_VERSION` 400→500, so cdylib plugins recompile against 0.5.0.
- **WASM** — the component exports `fidius-configure`; the host instantiates a
  **persistent store**, calls it once to bind config into a guest `OnceLock`, and
  retains the store so methods dispatch on the configured instance. Each configured
  handle is its own store, so N instances are genuinely independent.
- **Python** — the plugin module exports `__fidius_configure__(config) -> instance`;
  the host binds methods on the returned object instead of module-level functions.

## Limitations (0.5.0)

- **WASM streaming + config**: a server-streaming method on a *configured* WASM
  instance returns a clear error (a stream borrows its own store for its lifetime,
  which can't share the configured instance's persistent store). Use a unary method,
  or a zero-config streaming plugin. A follow-on lifts this.
- **WASM streaming of user-typed records** is a separate, pre-existing limit
  (streaming items must be primitives/`String`), independent of configuration.

## See also

- [Capabilities & the WASM Sandbox](wasm-capabilities.md) — a configured connector
  pairs naturally with `http` egress (`fidius_guest::http`).
- [Server-Streaming](streaming.md) — the other "bind a contract, not the framework's
  opinions" primitive.
