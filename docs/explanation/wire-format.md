<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# Wire Format

How fidius serializes data across the FFI boundary.

## The Decision

All data crossing the FFI boundary — method arguments, return values, and
error details — is serialized as **bincode** via `serde`. Method arguments
are always **tuple-encoded**: zero args serialize as `()`, one arg as
`(T,)`, multiple args as `(A, B, C)`. This encoding is uniform regardless
of arg count.

Bincode is the single wire format. Build profile has no effect: debug and
release builds produce byte-identical wire output, and a release host
loads a debug plugin (and vice versa) without issue.

## Why not JSON?

Before 0.1.0 fidius used JSON in debug builds and bincode in release,
gated by `cfg(debug_assertions)`. The intent was that a developer could
inspect raw wire bytes during debugging — a JSON string is human-readable,
bincode bytes are not.

In practice the cost outweighed the benefit:

- **Profile mixing blew up.** A release host refused to load a debug
  plugin (and vice versa) because the wire format constants didn't
  match. The failure manifested as a `WireFormatMismatch` error with no
  workaround other than "rebuild everything in one profile."
- **Nobody read raw wire bytes in the wild.** Inspection happens through
  `fidius inspect`, a debugger, or a `println!` of the deserialized
  value — not by staring at hex dumps of wire-format JSON.
- **JSON is 2–10x slower than bincode** for typical payloads. Every
  debug-build test cycle paid that cost.

0.1.0 removed the dual path per FIDIUS-I-0016. Wire format is now a
non-choice — bincode always.

## Inspecting Wire Bytes for Debugging

If you need to inspect what's being sent across the FFI during debugging,
the deserialized value is almost always what you want, not the raw bytes:

```rust
#[plugin_impl(MyTrait)]
impl MyTrait for MyPlugin {
    fn process(&self, input: MyInput) -> MyOutput {
        eprintln!("plugin received: {:?}", input);   // <-- here
        // ...
    }
}
```

For raw-byte inspection, `bincode::serialize` is deterministic — feed the
same input through it in a standalone binary and compare against what
flows across the FFI. This is only needed for very unusual debugging.

## PluginError.details — Still Stringified JSON

One subtlety: `PluginError::details` is typed `Option<String>` and
carries a JSON-encoded blob of structured context. This is **not** a
wire-format concern — it's an internal convention within `PluginError` so
arbitrary structured data can cross the FFI as a serde-friendly string
(bincode can't deserialize a free-form `serde_json::Value` because it
lacks `deserialize_any`).

Use `PluginError::with_details(code, msg, value)` to set it and
`err.details_value()` to parse it back to a `serde_json::Value` on the
host side. The outer `PluginError` struct is bincode-serialized as a
whole; the `details` string is just a normal string field in that
structure.

## WireError

`fidius-core::wire::WireError` is a thin wrapper for bincode errors:

```rust
#[derive(Debug, thiserror::Error)]
pub enum WireError {
    #[error("bincode wire error: {0}")]
    Bincode(#[from] bincode::Error),
}
```

In shim codegen, serialization failures are caught and converted to
`STATUS_SERIALIZATION_ERROR` (-2). The host receives this status code
and returns `CallError::Serialization`. No partial or corrupt data
reaches the caller.

## The Full Error Flow

When a plugin method returns `Err(PluginError { ... })`:

```
Plugin side:
  1. Method returns Err(PluginError)
  2. Shim calls wire::serialize(&err)  → Vec<u8>
  3. Shim sets out_ptr/out_len to the serialized error
  4. Shim returns STATUS_PLUGIN_ERROR (-3)

Host side:
  5. PluginHandle::call_method sees status == STATUS_PLUGIN_ERROR
  6. Reads output buffer as slice
  7. wire::deserialize::<PluginError>(slice) → PluginError
  8. Calls free_buffer(out_ptr, out_len)
  9. Returns Err(CallError::Plugin(plugin_error))
```

Both the success path and the error path use the same wire format. The
status code tells the host whether to deserialize the output buffer as
the expected return type or as a `PluginError`.

---

## Raw-wire methods (`#[wire(raw)]`)

Added in 0.2.0. A method declared `#[wire(raw)]` skips bincode in both
directions: the argument and successful return value are passed as raw
bytes with no encoding overhead. The signature is constrained to
`fn(data: &[u8]) -> Result<Vec<u8>, CallError>`.

Use raw wire when:

- The payload is already a binary format (Parquet, Arrow IPC, image
  buffers, audio frames, gzipped JSON…).
- The payload is large enough that the bincode round-trip is a
  measurable cost — typically ≥ a few hundred KB.

The error path still uses bincode-encoded `PluginError`. Raw mode is
recorded in the interface hash via a `!raw` marker on the method
signature, so a host built against a typed method cannot accidentally
load a plugin that ships the same name as raw (or vice-versa) — drift
fails at load time the same way any other signature change does.

Host call sites use `call_raw(method_index, &input_bytes)` instead of
the generated typed `Client` methods.

---

*Related documentation:*

- [Architecture Overview](architecture.md) — where wire format fits in the pipeline
- [Buffer Strategies](buffer-strategies.md) — how serialized data moves through buffers
- [Interface Evolution](interface-evolution.md) — detecting interface drift at load time
