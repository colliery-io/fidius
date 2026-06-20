<!-- Copyright 2026 Colliery, Inc. Licensed under Apache 2.0 -->

# How to: build a production WASM connector

A "production" connector usually needs three things beyond a hello-world plugin:
**rich types** in its interface, **streaming** of those typed records, and
**time-boxed** outbound HTTP so a slow upstream can't hang it. fidius supports all
three together. This page shows the shape; each piece has a worked fixture/test.

## Rich types (maps, tuples, nesting)

A connector's records can use `HashMap`/`BTreeMap`, tuples, and nesting — they
project to WIT automatically (`HashMap<K,V>` → `list<tuple<k,v>>`, `(A,B)` →
`tuple<a,b>`):

```rust
#[derive(fidius_macro::WitType, Clone)]
pub struct Event {
    pub id: u64,
    pub tags: std::collections::HashMap<String, String>, // rich type inside a record
}
```

See `docs/explanation/wasm-component-abi.md` and the `records-greeter` fixture's
`tally(HashMap, (i32,i32)) -> HashMap` round-trip
(`crates/fidius-host/tests/records_wasm.rs`).

## Stream the typed records

Return `Stream<Event>` — the macro emits a `events-stream` resource whose `next()`
yields one record at a time; the host pulls them lazily with `call_streaming`:

```rust
#[fidius::plugin_interface(version = 1, buffer = PluginAllocated, crate = "fidius_guest")]
pub trait Source: Send + Sync {
    fn events(&self, since: u64) -> fidius_guest::Stream<Event>;
}
```

Worked end-to-end in the `records-stream` fixture +
`crates/fidius-host/tests/records_stream_wasm.rs`. The **host-side** consumption
pattern (any backend) is the runnable `examples/05_record_stream`.

## Time-box the HTTP

Inside the connector, fetch upstream pages with a timeout so a stalled server fails
fast instead of hanging the stream:

```rust
use core::time::Duration;
let req = fidius_guest::http::Request::get(url).timeout(Duration::from_secs(5));
let body = match fidius_guest::http::send(req) {
    Ok(resp) => resp.text(),
    Err(_) => return /* surface a typed error / end the stream */,
};
```

Egress is still **two-key gated** — the package declares `capabilities = ["http"]`
and the host supplies an `EgressPolicy`. See
[Capabilities & the WASM Sandbox](../explanation/wasm-capabilities.md) and the
`macro-fetcher` fixture's `fetch_timeout` test
(`crates/fidius-host/tests/macro_egress_e2e.rs`).

## Putting it together

A REST-source connector = `Source::events` streams `Event` records, each page
fetched with `fidius_guest::http` under a timeout, the records typed with whatever
maps/tuples the upstream needs. The host loads it with `load_wasm` (+ an
`EgressPolicy`), binds config once with `load_wasm_configured`
([Configured Instances](../explanation/configured-instances.md)), and pulls the
stream with `call_streaming` — exactly the host composition in
[Write a Host Application](host-application.md).
