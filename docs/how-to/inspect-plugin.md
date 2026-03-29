<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# How to inspect a compiled plugin dylib

This guide shows how to use `fidius inspect` to examine a compiled plugin
dylib's registry metadata without running any plugin code.

## Prerequisites

- The `fidius` CLI installed
- A compiled plugin `.dylib` / `.so` / `.dll`

## 1. Run `fidius inspect`

Pass the path to your compiled plugin dylib:

```
$ fidius inspect path/to/libmy_plugin.dylib
```

## 2. Read the output

The command loads the dylib, calls `fidius_get_registry()` via `dlsym`,
validates the registry, and prints metadata for every plugin found. Here is an
example with two plugins:

```
Plugin Registry: path/to/libmy_plugin.dylib
  Plugins: 2

  [0] HelloGreeter
      Interface: Greeter
      Interface hash: 0x1a2b3c4d5e6f7890
      Interface version: 1
      Buffer strategy: PluginAllocated
      Wire format: Bincode
      Capabilities: 0x0000000000000000

  [1] GoodbyeGreeter
      Interface: Greeter
      Interface hash: 0x1a2b3c4d5e6f7890
      Interface version: 1
      Buffer strategy: PluginAllocated
      Wire format: Bincode
      Capabilities: 0x0000000000000000
```

### Field reference

| Field | Meaning |
|---|---|
| **Plugins** | Total number of `#[plugin_impl]` blocks in this dylib. |
| **[N] Name** | The struct name passed to `#[plugin_impl]` (e.g., `HelloGreeter`). |
| **Interface** | The trait name from `#[plugin_interface]` (e.g., `Greeter`). |
| **Interface hash** | FNV-1a hash of the required method signatures. Two plugins are compatible only when their hashes match. |
| **Interface version** | The `version = N` value from `#[plugin_interface(version = N, ...)]`. |
| **Buffer strategy** | One of `CallerAllocated`, `PluginAllocated`, or `Arena`. Determines how output memory is managed at the FFI boundary. |
| **Wire format** | `Json` (debug builds) or `Bincode` (release builds). The host rejects plugins compiled with a different wire format. |
| **Capabilities** | Bitfield indicating which optional methods this plugin implements. `0x0` means no optional methods, or the interface has none. Each bit corresponds to an `#[optional]` method in declaration order. |

## 3. Common use cases

### Verify a release build before deployment

```
$ cargo build --release
$ fidius inspect target/release/libmy_plugin.dylib
```

Check that:

- The plugin count matches what you expect.
- The wire format is `Bincode` (release) not `Json` (debug).
- The interface hash has not changed unexpectedly.

### Debug interface hash mismatches

If the host rejects a plugin with an interface hash error, inspect both sides:

```
$ fidius inspect libplugin_old.dylib
$ fidius inspect libplugin_new.dylib
```

Compare the `Interface hash` values. A difference means the required method
signatures changed between builds.

### Confirm multi-plugin dylibs

When shipping multiple plugins per dylib (see
[How to ship multiple plugins per dylib](multiple-plugins-per-dylib.md)),
inspect confirms all implementations are present and correctly registered.

For details on how the inspect command loads and validates the registry, see the [ABI specification](../reference/abi-specification.md).

## See also

- [How to scaffold a project](scaffold-project.md) -- create interface and
  plugin crates
- [How to ship multiple plugins per dylib](multiple-plugins-per-dylib.md) --
  inspect shows all plugins in one dylib
- [How to add async methods](async-methods.md) -- async does not affect
  inspect output; the FFI layer is always synchronous
