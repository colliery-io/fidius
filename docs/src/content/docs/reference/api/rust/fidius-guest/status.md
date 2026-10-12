---
title: "fidius-guest::status"
description: "FFI status codes returned by plugin method shims."
editUrl: false
sidebar:
  order: 20
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

FFI status codes returned by plugin method shims.

These `i32` values are the return type of every `extern "C"` function
in a plugin vtable. The host checks the status code before reading
the output buffer.

## Constants

| Name | Type | Value | Description |
|------|------|-------|-------------|
| <span id="const.STATUS_OK"></span>`STATUS_OK` | `i32` | `0` | Method executed successfully. Output buffer contains the serialized result. |
| <span id="const.STATUS_BUFFER_TOO_SMALL"></span>`STATUS_BUFFER_TOO_SMALL` | `i32` | `-1` | Output buffer was too small (CallerAllocated/Arena strategies only). The `out_len` parameter contains the required size. Retry with a larger buffer. |
| <span id="const.STATUS_SERIALIZATION_ERROR"></span>`STATUS_SERIALIZATION_ERROR` | `i32` | `-2` | Serialization or deserialization failed at the FFI boundary. This indicates a bug in the generated shims or a type mismatch. |
| <span id="const.STATUS_PLUGIN_ERROR"></span>`STATUS_PLUGIN_ERROR` | `i32` | `-3` | The plugin method returned an error. The output buffer contains a serialized `PluginError` with details. |
| <span id="const.STATUS_PANIC"></span>`STATUS_PANIC` | `i32` | `-4` | A panic was caught at the `extern "C"` boundary via `catch_unwind`. The output buffer may contain a panic message string, but this is not guaranteed. |
| <span id="const.STATUS_STREAM_END"></span>`STATUS_STREAM_END` | `i32` | `-5` | Clean end of a server-stream: the streaming `next()` shim has no more items (FIDIUS-I-0026 / FIDIUS-T-0138). Distinct from `STATUS_OK` with `out_len == 0`, which is a real zero-byte item (e.g. a unit `()` item). |
| <span id="const.STATUS_INVALID_INDEX"></span>`STATUS_INVALID_INDEX` | `i32` | `-6` | The callee has no function at the requested dispatch index. Returned by a host-function table's `dispatch` (plugin → host callback channel) for an out-of-range index. Cannot occur through generated clients once the bind-time version/hash gate has passed. |
