---
title: "fidius"
description: "Fidius — a Rust plugin framework for trait-to-dylib plugin systems."
sidebar:
  order: 1
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


Fidius — a Rust plugin framework for trait-to-dylib plugin systems.

This is the facade crate. Interface crates should depend on `fidius` only.
It re-exports everything needed to define interfaces and implement plugins.

