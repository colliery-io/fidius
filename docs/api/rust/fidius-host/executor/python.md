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



##### `call_client_streaming` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn call_client_streaming (& self , method : usize , items : Box < dyn Iterator < Item = serde_json :: Value > + Send > , args : Value ,) -> Result < Value , CallError >
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



##### `call_bidi_streaming` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn call_bidi_streaming (& self , method : usize , items : Box < dyn Iterator < Item = serde_json :: Value > + Send > , args : Value ,) -> Result < crate :: stream :: ChunkStream , CallError >
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





## Functions

### `fidius-host::executor::python::pump_python_stream`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn pump_python_stream (stream : fidius_python :: PythonStream) -> crate :: stream :: ChunkStream
```

Pump a `PythonStream` (a guest generator) into a [`crate::stream::ChunkStream`] on a dedicated GIL-holding thread (blocking_send = backpressure; native `Value` items, no framing). Shared by server-streaming ([`Pyo3Executor::call_streaming`]) and bidirectional ([`Pyo3Executor::call_bidi_streaming`]).

<details>
<summary>Source</summary>

```rust
fn pump_python_stream(stream: fidius_python::PythonStream) -> crate::stream::ChunkStream {
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Value, CallError>>(STREAM_CHANNEL_CAP);

    std::thread::spawn(move || {
        use fidius_python::PyStreamStep;
        loop {
            match stream.next() {
                // Clean end: drop `tx` → the host stream ends (None).
                PyStreamStep::End => break,
                // Producer error: surface one Err, then end.
                PyStreamStep::Error(pe) => {
                    let _ = tx.blocking_send(Err(CallError::Plugin(pe)));
                    break;
                }
                PyStreamStep::Item(jv) => {
                    // JSON is self-describing, so `Value` reconstructs fine here.
                    let item = match serde_json::from_value::<Value>(jv) {
                        Ok(v) => Ok(v),
                        Err(e) => Err(CallError::Deserialization(e.to_string())),
                    };
                    let is_err = item.is_err();
                    // blocking_send parks the GIL-free thread when the channel is full →
                    // backpressure. `Err` means the consumer dropped → cancel the generator.
                    if tx.blocking_send(item).is_err() {
                        stream.cancel();
                        break;
                    }
                    if is_err {
                        break;
                    }
                }
            }
        }
    });

    let body = futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|item| (item, rx))
    });
    crate::stream::ChunkStream::new(body)
}
```

</details>



