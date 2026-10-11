---
title: "fidius-host::executor"
description: "`PluginExecutor` — the dispatch seam across execution backends."
sidebar:
  order: 32
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


`PluginExecutor` — the dispatch seam across execution backends.

Fidius historically carried one dispatch implementation per backend: the
cdylib vtable/FFI path lived inside `PluginHandle`, and the Python (PyO3)
path lived in a *separate* `PythonPluginHandle` in `fidius-python`. This
module collapses that duplication: each backend is an executor, and the
caller-facing [`crate::handle::PluginHandle`] wraps them in an **enum**
(`Backend`) so its generic `call_method<I, O>` can serialise with each
backend's native currency.

