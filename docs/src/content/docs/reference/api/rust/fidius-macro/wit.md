---
title: "fidius-macro::wit"
description: "WIT mapping/rendering for the macro's WASM codegen."
sidebar:
  order: 51
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


WIT mapping/rendering for the macro's WASM codegen.

The implementation lives in the shared `fidius-wit` crate so the same logic
backs the macro, the `build.rs` helper, and the `fidius wit` CLI
(FIDIUS-I-0023). This module re-exports the pieces the macro uses.

