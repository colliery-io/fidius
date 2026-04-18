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
      Capabilities: 0x0000000000000000

  [1] GoodbyeGreeter
      Interface: Greeter
      Interface hash: 0x1a2b3c4d5e6f7890
      Interface version: 1
      Buffer strategy: PluginAllocated
      Capabilities: 0x0000000000000000
```

For a description of each field in the output, see the [CLI reference](../reference/cli.md#inspect).

## 3. Common use cases

### Verify a release build before deployment

```
$ cargo build --release
$ fidius inspect target/release/libmy_plugin.dylib
```

Check that:

- The plugin count matches what you expect.
- The interface hash has not changed unexpectedly.
- The buffer strategy matches what the host expects.

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
