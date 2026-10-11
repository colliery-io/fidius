---
title: "fidius-core::async_runtime"
description: "Per-dylib async runtime for plugin methods."
sidebar:
  order: 6
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


Per-dylib async runtime for plugin methods.

When the `async` feature is enabled, this module provides a lazily-initialized
tokio runtime shared across all plugin implementations in the dylib.
The generated shims call `FIDIUS_RUNTIME.block_on(...)` for async methods.

