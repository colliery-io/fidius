# fidius — Python SDK for authoring Fidius plugins

A Fidius plugin written in Python is a regular Python module whose plugin
functions are decorated with `@fidius.method`. The host imports the module,
looks up the registered callables by name, and dispatches calls into them.

## Quick start

```python
from fidius import method, PluginError

@method
def greet(name: str) -> str:
    return f"Hello, {name}!"

@method
def shout(text: str) -> str:
    if not text:
        raise PluginError("EMPTY_INPUT", "text must be non-empty")
    return text.upper()
```

Pair this with a `package.toml` declaring `runtime = "python"` and the
interface your module implements; pack with `fidius pack`; deploy alongside
any other Fidius plugin.

## Errors

Raise `PluginError(code, message, details=...)` for structured failures the
host should see as typed errors. Other exceptions become `code = "PYTHON_ERROR"`
with the formatted traceback in `details`.

## No runtime deps

Vendor this module alongside your plugin's own deps via
`pip install fidius --target vendor/`. It is pure Python, depends on
nothing else, and works on any Python ≥ 3.9.
