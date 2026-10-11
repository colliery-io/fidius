---
title: "fidius-guest::status"
description: "FFI status codes returned by plugin method shims."
sidebar:
  order: 21
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


FFI status codes returned by plugin method shims.

These `i32` values are the return type of every `extern "C"` function
in a plugin vtable. The host checks the status code before reading
the output buffer.

