---
title: "fidius-guest"
description: "`fidius-guest` — the wasm-buildable subset of the Fidius shared types."
editUrl: false
sidebar:
  order: 9
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

`fidius-guest` — the wasm-buildable subset of the Fidius shared types.

Extracted from `fidius-core` (FIDIUS-I-0022) so plugin guests — including
Rust WASM components built from `#[plugin_impl]` — can depend on the
interface hashing, descriptors, the neutral [`value::Value`](../../rust/fidius-guest/value/#enum.Value) model, and the
bincode wire format **without** dragging in the host-only packaging stack
(archive/compression/filesystem). This crate has no such dependencies and
compiles cleanly to `wasm32-wasip2`.

`fidius-core` re-exports every module here, so `fidius_core::descriptor`,
`fidius_core::hash`, `fidius_core::value`, … (and the `fidius` facade
re-exports) resolve unchanged — this split is internal, with no public-API
churn.

Note: `descriptor::ABI_VERSION` derives from this crate's `CARGO_PKG_VERSION`
(per ADR-0002), so `fidius-guest` is versioned in lockstep with `fidius-core`.

## Re-exports

- [`descriptor::*`](../../rust/fidius-guest/descriptor/)
- [`PluginError`](../../rust/fidius-guest/error/#struct.PluginError)
- [`Frame`](../../rust/fidius-guest/frame/#enum.Frame)
- [`FrameError`](../../rust/fidius-guest/frame/#enum.FrameError)
- [`status::*`](../../rust/fidius-guest/status/)
- [`FidiusStreamHandle`](../../rust/fidius-guest/stream_ffi/#struct.FidiusStreamHandle)
- [`Stream`](../../rust/fidius-guest/stream_marker/#struct.Stream)
- [`from_value`](../../rust/fidius-guest/value/#fn.from_value)
- [`to_value`](../../rust/fidius-guest/value/#fn.to_value)
- [`Value`](../../rust/fidius-guest/value/#enum.Value)
- [`ValueError`](../../rust/fidius-guest/value/#struct.ValueError)
