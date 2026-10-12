---
title: "fidius-host::executor"
description: "`PluginExecutor` — the dispatch seam across execution backends."
editUrl: false
sidebar:
  order: 31
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

`PluginExecutor` — the dispatch seam across execution backends.

Fidius historically carried one dispatch implementation per backend: the
cdylib vtable/FFI path lived inside `PluginHandle`, and the Python (PyO3)
path lived in a *separate* `PythonPluginHandle` in `fidius-python`. This
module collapses that duplication: each backend is an executor, and the
caller-facing [`crate::handle::PluginHandle`](../../../rust/fidius-host/handle/#struct.PluginHandle) wraps them in an **enum**
(`Backend`) so its generic `call_method<I, O>` can serialise with each
backend's native currency.

**Why an enum, not `Box<dyn>` — and why two traits**

The backends do **not** share a typed wire. cdylib decodes concrete-type
**bincode** (not self-describing — it can't be reconstructed from an erased
value), while Python (and the future WASM component backend) consume a
self-describing [`fidius_core::Value`](../../../rust/fidius-guest/value/#enum.Value). A single `call(method, Value)` trait
method therefore cannot serve cdylib without breaking its ABI (see
FIDIUS-A-0003 / FIDIUS-I-0021 amendment). So:

- [`PluginExecutor`](../../../rust/fidius-host/executor/#trait.PluginExecutor) is the **common** surface every backend shares:
  metadata plus the raw byte path. For cdylib, `call_raw` is *also* the
  carrier for typed calls (the wrapper bincode-wraps the concrete type).
- [`ValueExecutor`](../../../rust/fidius-host/executor/#trait.ValueExecutor) adds the typed [`fidius_core::Value`](../../../rust/fidius-guest/value/#enum.Value) call, implemented
  only by the self-describing backends (Python, WASM). cdylib does not
  implement it — `PluginHandle` routes cdylib typed calls through its own
  bincode `call_method`, keeping the bytes byte-identical to pre-refactor.

## Re-exports

- [`CdylibExecutor`](../../../rust/fidius-host/executor/cdylib/#struct.CdylibExecutor)
- [`Pyo3Executor`](../../../rust/fidius-host/executor/python/#struct.Pyo3Executor)
- [`precompile_component`](../../../rust/fidius-host/executor/wasm/#fn.precompile_component)
- [`validate_component`](../../../rust/fidius-host/executor/wasm/#fn.validate_component)
- [`EgressDenied`](../../../rust/fidius-host/executor/wasm/#struct.EgressDenied)
- [`EgressPolicy`](../../../rust/fidius-host/executor/wasm/#trait.EgressPolicy)
- [`ResponseDirective`](../../../rust/fidius-host/executor/wasm/#enum.ResponseDirective)
- [`TcpTarget`](../../../rust/fidius-host/executor/wasm/#struct.TcpTarget)
- [`WasmComponentExecutor`](../../../rust/fidius-host/executor/wasm/#struct.WasmComponentExecutor)
- [`WasmMethod`](../../../rust/fidius-host/executor/wasm/#struct.WasmMethod)

## Traits

### <span id="trait.PluginExecutor"></span><code style="font-size: 0.85em">trait PluginExecutor</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

**Supertraits:** `Send + Sync`

The surface every execution backend shares.

Implementations must be `Send + Sync`: methods take `&self`, so a handle can
be shared across threads as long as the backend is internally thread-safe.

#### Methods

##### <span id="PluginExecutor.info"></span><code>info</code>

```rust
fn info(&self) -> &PluginInfo
```

Owned metadata describing the loaded plugin.

<details>
<summary>Source</summary>

```rust
    fn info(&self) -> &PluginInfo;
```

</details>

##### <span id="PluginExecutor.method_count"></span><code>method_​count</code>

```rust
fn method_count(&self) -> u32
```

Number of methods the plugin exposes, in interface (vtable) order.

<details>
<summary>Source</summary>

```rust
    fn method_count(&self) -> u32;
```

</details>

##### <span id="PluginExecutor.call_raw"></span><code>call_​raw</code>

```rust
fn call_raw(
    &self,
    method: usize,
    input: &[u8],
) -> Result<Vec<u8>, CallError>
```

Raw bulk-bytes dispatch for `#[wire(raw)]` methods: opaque bytes in, opaque bytes out, no per-element marshalling. Opaque bytes are language-neutral (a WIT `list<u8>`), so this is uniform across backends.

<details>
<summary>Source</summary>

```rust
    fn call_raw(&self, method: usize, input: &[u8]) -> Result<Vec<u8>, CallError>;
```

</details>

### <span id="trait.ValueExecutor"></span><code style="font-size: 0.85em">trait ValueExecutor</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

**Supertraits:** `PluginExecutor`

Backends whose typed boundary is the self-describing `Value` model — Python today, and the Phase-2 WASM component executor.

cdylib deliberately does **not** implement this: its typed path is
concrete-type bincode, which `Value` cannot reproduce. `PluginHandle`
dispatches cdylib typed calls directly via bincode instead.

#### Methods

##### <span id="ValueExecutor.call"></span><code>call</code>

```rust
fn call(&self, method: usize, args: Value) -> Result<Value, CallError>
```

Typed dispatch by method index. Arguments and returns cross as a self-describing `Value`; the backend maps it to its native form (Python → `PyObject`, WASM → `component::Val`).

<details>
<summary>Source</summary>

```rust
    fn call(&self, method: usize, args: Value) -> Result<Value, CallError>;
```

</details>
