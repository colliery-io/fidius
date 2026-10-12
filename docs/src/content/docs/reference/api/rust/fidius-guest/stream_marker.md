---
title: "fidius-guest::stream_marker"
description: "The `fidius::Stream<T>` server-streaming return marker (FIDIUS-I-0026, D4)."
editUrl: false
sidebar:
  order: 22
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

The `fidius::Stream<T>` server-streaming return marker (FIDIUS-I-0026, D4).

## Structs

### <span id="struct.Stream"></span><code style="font-size: 0.85em">struct Stream&lt;T&gt;</code>

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

**Two forms (FIDIUS-I-0026)**

- **Marker form** ([`Stream::new`](../../../rust/fidius-guest/stream_marker/#Stream.new)): no items. Used purely to *declare* a
  streaming method in an interface trait — the Python path (Phase 1) never
  iterates it in Rust (its generator is bridged to a `ChunkStream`
  host-side).
- **Iterator-backed form** ([`Stream::from_iter`](../../../rust/fidius-guest/stream_marker/#Stream.from_iter)): a Rust **WASM** guest
  (Phase 2) returns real items. The macro-generated component resource pulls
  them one at a time via [`Stream::next_item`](../../../rust/fidius-guest/stream_marker/#Stream.next_item) to satisfy the WIT contract:
  ```wit
  resource <m>-stream { next: func() -> result<option<T>, plugin-error>; }
  <m>: func(args) -> own<<m>-stream>;
  ```
  `some(item)` = item, `none` = clean end, `err` = mid-stream error;
  dropping the resource runs the guest dtor = cancel (design decision D3).

#### Methods

##### <span id="Stream.new"></span><code>new</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn new() -> Self
```

The marker form — declares a streaming method without producing items. The interface/Python path uses this; it is never iterated in Rust.

<details>
<summary>Source</summary>

```rust
    pub fn new() -> Self {
        Self {
            iter: None,
            _marker: PhantomData,
        }
    }
```

</details>

##### <span id="Stream.from_iter"></span><code>from_​iter</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn from_iter<I>(items: I) -> Self
where
    I: IntoIterator<Item = T>,
    I::IntoIter: Send + 'static,
```

Build a stream from any iterator — how a Rust WASM guest produces its items. The iterator must be `Send + 'static` so the host can drive the component resource across its pump thread.

<details>
<summary>Source</summary>

```rust
    pub fn from_iter<I>(items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + 'static,
    {
        Self {
            iter: Some(Box::new(items.into_iter())),
            _marker: PhantomData,
        }
    }
```

</details>

##### <span id="Stream.next_item"></span><code>next_​item</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn next_item(&mut self) -> Option<T>
```

Advance the underlying iterator by one item. Driven by the macro-generated component resource's `next()` (WS.2). Returns `None` for the marker form and at end of iteration.

<details>
<summary>Source</summary>

```rust
    pub fn next_item(&mut self) -> Option<T> {
        self.iter.as_mut().and_then(|it| it.next())
    }
```

</details>
