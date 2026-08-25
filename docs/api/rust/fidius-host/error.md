# fidius-host::error <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Error types for fidius-host plugin loading and calling.

## Enums

### `fidius-host::error::LoadError` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Errors that can occur when loading a plugin.

#### Variants

- **`LibraryNotFound`**
- **`SymbolNotFound`**
- **`InvalidMagic`**
- **`IncompatibleRegistryVersion`**
- **`IncompatibleAbiVersion`**
- **`InterfaceHashMismatch`**
- **`BufferStrategyMismatch`**
- **`ArchitectureMismatch`**
- **`UnknownBufferStrategy`**
- **`SignatureInvalid`**
- **`SignatureRequired`**
- **`PluginNotFound`**
- **`LibLoading`**
- **`Io`**
- **`PythonLoad`** - Python loader failed (only produced with the `python` feature on).
Wraps `fidius_python::PythonLoadError` as a string to keep the
fidius-host public error enum type-clean across feature gates.
- **`WasmLoad`** - WASM component loader failed (only produced with the `wasm` feature on).
Wraps wasmtime/instantiation errors as a string to keep the public enum
type-clean across feature gates.
- **`ConfigSerialization`** - Failed to serialize a plugin's config for construction (FIDIUS-A-0006 /
CI.2). Practically never happens for a well-formed config type.
- **`HostInterfaceVersionMismatch`** - The plugin was built against a different **version** of a host
interface than the host provides (plugin → host callback channel).
Raised at bind (load) time; the host-function table is never
installed, so a mismatched surface cannot mis-dispatch.
- **`HostInterfaceHashMismatch`** - The plugin was built against a different **signature set** of a host
interface than the host provides (same declared version, drifted
methods). Raised at bind (load) time; nothing is installed.
- **`HostImportRegistryInvalid`** - The plugin's `fidius_get_host_imports` registry was malformed (bad
magic, unsupported layout version, or invalid pointers).
- **`HostBindFailed`** - The plugin's bind shim rejected an offered host-function table (e.g.
the interface was already bound for this library).



### `fidius-host::error::CallError` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Errors that can occur when calling a plugin method.

#### Variants

- **`Serialization`**
- **`Deserialization`**
- **`Plugin`**
- **`Panic`**
- **`BufferTooSmall`**
- **`NotImplemented`** - Optional method is not implemented by this plugin — its capability bit is unset.
Returned when a method marked `#[optional]` is called on a plugin that chose not
to implement it. Not returned for out-of-range method indices; see `InvalidMethodIndex`.
- **`InvalidMethodIndex`**
- **`UnknownStatus`**
- **`WireModeMismatch`** - A method was dispatched through the wrong wire path — a `#[wire(raw)]`
method called via the typed path, or vice versa. Backend-agnostic: the
Python and (future) WASM executors both enforce the raw/typed split.
- **`Backend`** - A runtime-level fault originating inside an execution backend that is
*not* a plugin-raised [`PluginError`] — e.g. a future WASM trap
(unreachable, out-of-bounds) or an interpreter-level failure. Carries
the backend's runtime name and a message. Plugin-raised errors (Python
exceptions included) stay in [`CallError::Plugin`] so their structured
`code`/`message`/`details` (including tracebacks) round-trip.
- **`MalformedFrame`** - A streaming backend produced bytes that did not decode as a valid
[`fidius_core::frame::Frame`] (bad tag, truncated, malformed payload).
Distinct from [`CallError::Deserialization`], which is about an item's
*contents*; this is about the framing around items. (FIDIUS-I-0026.)
- **`StreamAborted`** - A stream ended without a terminal `END`/`ERROR` frame — the producer
went away mid-stream (e.g. a dropped backend task, a crashed
interpreter, a closed channel). (FIDIUS-I-0026.)



