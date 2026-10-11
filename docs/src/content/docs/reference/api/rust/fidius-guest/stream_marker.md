---
title: "fidius-guest::stream_marker"
description: "The `fidius::Stream<T>` server-streaming return marker (FIDIUS-I-0026, D4)."
sidebar:
  order: 23
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


The `fidius::Stream<T>` server-streaming return marker (FIDIUS-I-0026, D4).

## Structs

### <span id="struct.Stream"></span>`struct Stream<T>`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>


Marker type a plugin interface uses to declare a **server-streaming** method:

```rust
#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait Source: Send + Sync {
fn read(&self, config: String) -> fidius::Stream<Row>;
}
```
`#[plugin_interface]` recognises a return type whose final path segment is
`Stream<T>` and, for that method:
1. folds a `!stream` marker into the interface hash (so a streaming method
can never be confused with a unary `-> T` method of the same name/args —
a producer/consumer mismatch is rejected at load), and
2. (Phase 1+) generates a host-side client method returning the runtime
pull handle (`fidius_host::ChunkStream`).
The marker carries no data — the *runtime* representation of a stream is the
host-side `ChunkStream`, not this type. It exists so the interface trait
type-checks and so the macro has an explicit, unambiguous thing to detect
(rather than guessing from `impl Stream`). `T` is the per-item type and
follows the same wire/`WitType` rules as a unary return.
Argument-position `Stream<T>` (client-streaming / bidirectional) is rejected
in v1.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `iter` | `Option < Box < dyn Iterator < Item = T > + Send > >` | `None` = the pure marker form; `Some` = an iterator-backed producer. |
| `_marker` | `PhantomData < fn () -> T >` |  |



