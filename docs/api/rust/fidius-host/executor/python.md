# fidius-host::executor::python <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


`Pyo3Executor` — the Python execution backend, behind the `python` feature.

Thin host-side wrapper over `fidius_python::PythonPluginHandle` (which owns
the embedded-interpreter dispatch) that adapts it to the
[`crate::executor`] traits so Python plugins flow through the same
[`crate::handle::PluginHandle`] as cdylib plugins.
Typed calls cross as a self-describing [`Value`]. The Python layer already
speaks self-describing JSON (`call_typed_json`), so the adapter bridges
`Value ↔ JSON` with `serde_json` — `Value` serialises to exactly the JSON
the Python `value_bridge` expects, so this is behaviour-identical to the
pre-unification path (`serde_json::to_vec(input) → call_typed_json`), just
routed through the neutral `Value` currency.

## Structs

### `fidius-host::executor::python::Pyo3Executor`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Python-backed executor: an embedded-interpreter plugin handle plus the host-facing [`PluginInfo`] (built from the package manifest + interface descriptor at load time).

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `py` | `PythonPluginHandle` |  |
| `info` | `PluginInfo` |  |

#### Methods

##### `new` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn new (py : PythonPluginHandle , info : PluginInfo) -> Self
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





