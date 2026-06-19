# fidius-host::executor <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


`PluginExecutor` — the dispatch seam across execution backends.

Fidius historically carried one dispatch implementation per backend: the
cdylib vtable/FFI path lived inside `PluginHandle`, and the Python (PyO3)
path lived in a *separate* `PythonPluginHandle` in `fidius-python`. This
module collapses that duplication: each backend is an executor, and the
caller-facing [`crate::handle::PluginHandle`] wraps them in an **enum**
(`Backend`) so its generic `call_method<I, O>` can serialise with each
backend's native currency.

