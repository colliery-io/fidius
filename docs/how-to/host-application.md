<!-- Copyright 2026 Colliery, Inc. Licensed under Apache 2.0 -->

# How to: write a host application

A *host* loads plugins and calls them. This page is the map; each pattern has a
runnable counterpart under [`examples/`](https://github.com/colliery-io/fidius/tree/main/examples)
(`cargo run -p fidius-examples --example <name>`).

Enable the host surface on the `fidius` facade:

```toml
fidius = { version = "0.5", features = ["streaming"] } # implies "host"
```

## Build a host and load a plugin

```rust
use fidius::PluginHost;

let host = PluginHost::builder()
    .search_path("./plugins")        // where packages live
    .require_signature(true)         // optional: enforce Ed25519 signatures
    .trusted_keys(&keys)
    .build()?;

let handle = host.load("my-plugin", &MyTrait_DESCRIPTOR)?;        // cdylib
// host.load_wasm("…", &MyTrait_WASM_DESCRIPTOR)?                  // WASM package
// host.load_python("…", &MyTrait_PYTHON_DESCRIPTOR)?             // Python package
let out: Out = handle.call_method(METHOD_GREET, &(input,))?;
```

In-process (the plugin is linked into the host binary) uses
`PluginHandle::find_in_process_descriptor(name)` + `from_descriptor` — see
`examples/01_load_and_call`.

## Configure once, call many

Bind config at construction; the config crosses the boundary once and N
differently-configured instances coexist (see [Configured Instances](../explanation/configured-instances.md)):

```rust
let src = host.load_wasm_configured("rest-source", &Source_WASM_DESCRIPTOR, &cfg)?;
// cdylib: PluginHandle::configure_in_process(desc, &cfg)?
// python: host.load_python_configured(name, &desc, &cfg)?
```
`examples/02_configure`.

## Brokered HTTP egress (WASM)

A sandboxed connector gets outbound HTTP only with the two-key gate — the package
declares `capabilities = ["http"]` **and** the host supplies an `EgressPolicy`:

```rust
let host = PluginHost::builder().egress(my_policy).build()?;   // or .egress_policy(arc)
```
See [Capabilities & the WASM Sandbox](../explanation/wasm-capabilities.md). The
guest calls `fidius_guest::http::get(url)`.

## Consume a stream

```rust
use futures::StreamExt;
let mut s = handle.call_streaming::<_, Record>(READ, &(cfg,)).await?;
while let Some(item) = s.next().await {
    let r: Record = fidius::from_value(item?)?;
}
```
`examples/03_streaming` (primitives); `examples/05_record_stream` (rich-typed
records). For a full WASM connector — typed records + maps + time-boxed HTTP — see
[Build a Production Connector](production-connector.md).

## Compose plugins (a pipeline)

The host owns the wiring between plugins — pull from one, feed another:

```rust
let mut stream = reader.call_streaming::<_, u64>(READ, &(n,)).await?;
while let Some(item) = stream.next().await {
    let v = fidius::from_value::<u64>(item?)?;
    let out: String = transformer.call_method(TRANSFORM, &(v,))?;   // plugin B
}
```
`examples/04_pipeline` (and `fidius_test::pump` + a plugin-backed `StreamSink`, in
`crates/fidius-host/tests/multi_plugin_pipeline.rs`).

## See also

- Scaffolding: `fidius init-interface` / `init-plugin` / `init-host` ([CLI](../reference/cli.md)).
- [Your First Plugin](../tutorials/your-first-plugin.md), [Errors](../reference/errors.md).
