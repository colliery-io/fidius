<!-- Copyright 2026 Colliery, Inc. Licensed under Apache 2.0 -->

# fidius examples

Runnable, self-contained host programs — each defines an interface + an
**in-process** plugin and drives it through the `PluginHost`/`PluginHandle` API.
No artifact building, no toolchain: `cargo run -p fidius-examples --example <name>`.

| Example | Pattern (host-side composition) |
| --- | --- |
| `01_load_and_call` | Load a plugin and call a method through the unified handle. |
| `02_configure` | Bind config once; N differently-configured instances of one plugin coexist. |
| `03_streaming` | Consume a server-streaming plugin via `call_streaming` (lazy, backpressured). |
| `04_pipeline` | **Multi-plugin pipeline** — the host wires plugin A's stream into plugin B. |
| `05_record_stream` | Stream **rich-typed records** (a record with a `HashMap` field) item-by-item. |
| `06_bidi_transform` | **Bidirectional** streaming — the host produces an input stream, the plugin consumes it and produces a transformed output stream, pulled lazily (ADR-0010). |
| `07_client_streaming` | **Client-streaming** — the host produces a stream the plugin pulls and reduces (lazy producer; unbounded input, bounded memory). |

These use the **in-process** cdylib path (the plugin is linked into the example
binary) so they're self-contained. The same host API loads dylib / WASM / Python
packages — see `PluginHost::load`, `load_wasm`, `load_python` (and their
`*_configured` / `*_with_egress` variants), exercised end-to-end in
`crates/fidius-host/tests/` and walked through in `docs/`.
