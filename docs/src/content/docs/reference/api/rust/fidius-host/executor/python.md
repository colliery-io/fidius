---
title: "fidius-host::executor::python"
description: "`Pyo3Executor` — the Python execution backend, behind the `python` feature."
editUrl: false
sidebar:
  order: 35
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

`Pyo3Executor` — the Python execution backend, behind the `python` feature.

Thin host-side wrapper over `fidius_python::PythonPluginHandle` (which owns
the embedded-interpreter dispatch) that adapts it to the
[`crate::executor`](../../../../rust/fidius-host/executor/) traits so Python plugins flow through the same
[`crate::handle::PluginHandle`](../../../../rust/fidius-host/handle/#struct.PluginHandle) as cdylib plugins.

Typed calls cross as a self-describing `Value`. The Python layer already
speaks self-describing JSON (`call_typed_json`), so the adapter bridges
`Value ↔ JSON` with `serde_json` — `Value` serialises to exactly the JSON
the Python `value_bridge` expects, so this is behaviour-identical to the
pre-unification path (`serde_json::to_vec(input) → call_typed_json`), just
routed through the neutral `Value` currency.

## Structs

### <span id="struct.Pyo3Executor"></span><code style="font-size: 0.85em">struct Pyo3Executor</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

Python-backed executor: an embedded-interpreter plugin handle plus the host-facing [`PluginInfo`](../../../../rust/fidius-host/types/#struct.PluginInfo) (built from the package manifest + interface descriptor at load time).

#### Methods

##### <span id="Pyo3Executor.new"></span><code>new</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn new(py: PythonPluginHandle, info: PluginInfo) -> Self
```

Wrap a loaded `PythonPluginHandle` with its owned metadata.

<details>
<summary>Source</summary>

```rust
    pub fn new(py: PythonPluginHandle, info: PluginInfo) -> Self {
        Self { py, info }
    }
```

</details>

##### <span id="Pyo3Executor.call_client_streaming"></span><code>call_​client_​streaming</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn call_client_streaming(
    &self,
    method: usize,
    items: Box<dyn Iterator<Item = serde_json::Value> + Send>,
    args: Value,
) -> Result<Value, CallError>
```

Client-streaming (FIDIUS-I-0030 CS2.4): the host produces `items`; the plugin method receives them as a host-fed iterator + returns a value. Pivots through JSON like the unary path.

<details>
<summary>Source</summary>

```rust
    pub fn call_client_streaming(
        &self,
        method: usize,
        items: Box<dyn Iterator<Item = serde_json::Value> + Send>,
        args: Value,
    ) -> Result<Value, CallError> {
        let args_json =
            serde_json::to_vec(&args).map_err(|e| CallError::Serialization(e.to_string()))?;
        let out = self
            .py
            .call_client_streaming_json(method, items, &args_json)
            .map_err(CallError::from)?;
        serde_json::from_slice(&out).map_err(|e| CallError::Deserialization(e.to_string()))
    }
```

</details>

##### <span id="Pyo3Executor.call_bidi_streaming"></span><code>call_​bidi_​streaming</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn call_bidi_streaming(
    &self,
    method: usize,
    items: Box<dyn Iterator<Item = serde_json::Value> + Send>,
    args: Value,
) -> Result<crate::stream::ChunkStream, CallError>
```

Bidirectional streaming (FIDIUS-I-0032 / ADR-0010): the host produces `items` (the plugin's input iterator) and consumes the plugin's returned generator as a `ChunkStream`. Pulling the output pulls the input — the synchronous lazy-pull composition. `args` are the non-stream args.

<details>
<summary>Source</summary>

```rust
    pub fn call_bidi_streaming(
        &self,
        method: usize,
        items: Box<dyn Iterator<Item = serde_json::Value> + Send>,
        args: Value,
    ) -> Result<crate::stream::ChunkStream, CallError> {
        let args_json =
            serde_json::to_vec(&args).map_err(|e| CallError::Serialization(e.to_string()))?;
        let stream = self
            .py
            .call_bidi_streaming_start(method, items, &args_json)
            .map_err(CallError::from)?;
        Ok(pump_python_stream(stream))
    }
```

</details>
