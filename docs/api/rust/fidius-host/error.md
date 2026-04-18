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



