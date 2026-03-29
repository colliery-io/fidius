<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# CLI Reference

Complete reference for the `fidius` command-line tool.

**Crate:** `fidius-cli`
**Source:** `fidius-cli/src/main.rs`, `fidius-cli/src/commands.rs`

---

## Synopsis

```
fidius <COMMAND> [OPTIONS]
```

Top-level options: `--help`, `--version`.

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success. |
| `1` | Error. An error message is printed to stderr in the format `error: <message>`. |

For `fidius verify`, signature validation failure exits with code `1` and prints `Signature INVALID: <path>` to stderr (exits directly, does not use the general error handler).

---

## Commands

### `init-interface`

Scaffold a new plugin interface crate.

```
fidius init-interface <NAME> --trait <TRAIT_NAME> [--path <DIR>] [--version <VERSION>]
```

| Argument / Flag | Type | Required | Description |
|-----------------|------|----------|-------------|
| `NAME` | positional | yes | Crate name for the interface (e.g., `my-interface`). |
| `--trait` | string | yes | Trait name to generate (e.g., `MyFilter`). |
| `--path` | path | no | Output directory. Default: current directory. |
| `--version` | string | no | Pin the `fidius` dependency version. Overrides auto-detection. |

**Generated files:**

```
<NAME>/
  Cargo.toml
  src/
    lib.rs
```

**`Cargo.toml`** contains a `[dependencies]` entry for `fidius`. **`src/lib.rs`** contains a skeleton trait with `#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]`, a single `fn process(&self, input: String) -> String` method, and re-exports of `fidius::plugin_impl` and `fidius::PluginError` so plugin crates only need to depend on the interface crate.

**Dependency resolution algorithm:**

The `fidius` dependency value is resolved as follows:

1. If `"fidius"` is a path that exists on disk relative to CWD (the directory where you run `fidius init-interface`), use `{ path = "fidius" }`.
2. If `--version` is set, use `"<version>"` (quoted version string).
3. Query `https://crates.io/api/v1/crates/fidius` for the latest stable version. If found, use `"<latest_version>"`.
4. Fall back to `{ path = "fidius" }` with a warning to stderr.

**Output on success:**

```
Created interface crate: <path>/<NAME>
```

**Error:** If the target directory already exists, prints `error: directory '<path>' already exists` and exits with code `1`.

---

### `init-plugin`

Scaffold a new plugin implementation crate.

```
fidius init-plugin <NAME> --interface <INTERFACE> --trait <TRAIT_NAME> [--path <DIR>] [--version <VERSION>]
```

| Argument / Flag | Type | Required | Description |
|-----------------|------|----------|-------------|
| `NAME` | positional | yes | Crate name for the plugin (e.g., `my-plugin`). |
| `--interface` | string | yes | Interface crate: a local path, crates.io name, or crate name. |
| `--trait` | string | yes | Trait name from the interface crate. |
| `--path` | path | no | Output directory. Default: current directory. |
| `--version` | string | no | Pin the interface dependency version. Overrides auto-detection. |

**Generated files:**

```
<NAME>/
  Cargo.toml
  src/
    lib.rs
```

**`Cargo.toml`** sets `crate-type = ["cdylib"]` and includes dependencies on the interface crate and `fidius-core`. The interface crate name is extracted from the `--interface` value (file name component), and hyphens are converted to underscores for the Rust module name.

**`src/lib.rs`** contains a struct `My{TraitName}`, a `#[plugin_impl(TraitName)]` block with a `todo!()` method, and `fidius_core::fidius_plugin_registry!()`.

**Dependency resolution** for the interface follows the same algorithm as `init-interface`.

**Output on success:**

```
Created plugin crate: <path>/<NAME>
```

**Error:** If the target directory already exists, prints `error: directory '<path>' already exists` and exits with code `1`.

---

### `keygen`

Generate an Ed25519 signing keypair.

```
fidius keygen --out <BASE_NAME>
```

| Argument / Flag | Type | Required | Description |
|-----------------|------|----------|-------------|
| `--out` | string | yes | Base name for output files. Writes `<out>.secret` (32 bytes) and `<out>.public` (32 bytes). |

Key bytes are raw Ed25519 key material (not PEM or any other encoding).

**Output on success:**

```
Generated keypair:
  Secret: <out>.secret
  Public: <out>.public
```

---

### `sign`

Sign a plugin dylib with an Ed25519 secret key.

```
fidius sign --key <SECRET_KEY_PATH> <DYLIB_PATH>
```

| Argument / Flag | Type | Required | Description |
|-----------------|------|----------|-------------|
| `--key` | path | yes | Path to the 32-byte secret key file. |
| `DYLIB_PATH` | positional | yes | Path to the dylib to sign. |

Reads the entire dylib, signs it, and writes the 64-byte signature to `<DYLIB>.sig` (the full filename including extension, with `.sig` appended). For example, signing `libplugin.dylib` produces `libplugin.dylib.sig`.

**Output on success:**

```
Signed: <dylib_path> -> <sig_path>
```

**Errors:**

- Secret key file is not exactly 32 bytes: `"secret key must be exactly 32 bytes"`.

---

### `verify`

Verify a plugin dylib's Ed25519 signature.

```
fidius verify --key <PUBLIC_KEY_PATH> <DYLIB_PATH>
```

| Argument / Flag | Type | Required | Description |
|-----------------|------|----------|-------------|
| `--key` | path | yes | Path to the 32-byte public key file. |
| `DYLIB_PATH` | positional | yes | Path to the dylib to verify. |

Reads the dylib, reads the signature from `<DYLIB>.sig`, and verifies.

**Output on success:**

```
Signature valid: <dylib_path>
```

**Output on failure:**

```
Signature INVALID: <dylib_path>
```

Exits with code `1` on invalid signature (bypasses the general error handler).

**Errors:**

- Public key not 32 bytes: `"public key must be exactly 32 bytes"`.
- Invalid public key bytes: `"invalid public key: <details>"`.
- Signature file not found: `"signature file not found: <sig_path>"`.
- Signature not 64 bytes: `"signature must be exactly 64 bytes"`.

---

### `inspect`

Inspect a plugin dylib's registry and descriptor metadata.

```
fidius inspect <DYLIB_PATH>
```

| Argument / Flag | Type | Required | Description |
|-----------------|------|----------|-------------|
| `DYLIB_PATH` | positional | yes | Path to the dylib to inspect. |

Loads the library via `fidius_host::loader::load_library` and prints metadata for all plugins in the registry.

**Output format:**

```
Plugin Registry: <dylib_path>
  Plugins: <count>

  [0] <plugin_name>
      Interface: <interface_name>
      Interface hash: 0x<16-digit hex>
      Interface version: <version>
      Buffer strategy: <Debug repr>
      Wire format: <Debug repr>
      Capabilities: 0x<16-digit hex>
```

One block per plugin, 0-indexed.

**Errors:**

- Load failure: `"failed to load <path>: <load_error>"`.

---

### `package`

Package management commands. All subcommands operate on a package directory
(a directory containing `package.toml`).

```
fidius package <SUBCOMMAND>
```

#### `package validate`

Validate a package manifest. Parses the `package.toml` without a host-defined
schema (accepts any `[metadata]` section) and prints a summary.

```
fidius package validate <DIR>
```

| Argument / Flag | Type | Required | Description |
|-----------------|------|----------|-------------|
| `DIR` | positional | yes | Path to the package directory. |

**Output on success:**

```
Package: <name> v<version>
  Interface: <interface> (version <interface_version>)
  Source hash: <hash>          # only if source_hash is set
  Dependencies:                # only if dependencies exist
    <name> = "<requirement>"
  Metadata: <N> field(s)

Manifest valid.
```

**Errors:** `PackageError::ManifestNotFound` if no `package.toml` exists;
`PackageError::ParseError` if the TOML is invalid.

---

#### `package build`

Build a package by running `cargo build` inside the package directory. Builds
in release mode by default.

```
fidius package build <DIR> [--debug]
```

| Argument / Flag | Type | Required | Description |
|-----------------|------|----------|-------------|
| `DIR` | positional | yes | Path to the package directory. |
| `--debug` | flag | no | Build in debug mode instead of release. |

**Output on success:**

```
Building package: <name> v<version>
Build successful. Output in <dir>/target/<profile>/
```

**Errors:** Fails if `package.toml` or `Cargo.toml` is missing, or if
`cargo build` returns a non-zero exit code.

---

#### `package inspect`

Inspect a package manifest. Prints all fields including individual `[metadata]`
key-value pairs.

```
fidius package inspect <DIR>
```

| Argument / Flag | Type | Required | Description |
|-----------------|------|----------|-------------|
| `DIR` | positional | yes | Path to the package directory. |

**Output format:**

```
Package: <dir>
  Name: <name>
  Version: <version>
  Interface: <interface>
  Interface version: <interface_version>
  Source hash: <hash>          # only if set
  Dependencies:                # only if present
    <name> = "<requirement>"
  Metadata:                    # only if metadata is a table
    <key> = <value>
```

---

#### `package sign`

Sign a package manifest with an Ed25519 secret key. Signs the `package.toml`
file and writes the signature to `package.toml.sig`.

```
fidius package sign --key <SECRET_KEY_PATH> <DIR>
```

| Argument / Flag | Type | Required | Description |
|-----------------|------|----------|-------------|
| `--key` | path | yes | Path to the 32-byte secret key file. |
| `DIR` | positional | yes | Path to the package directory. |

**Output on success:**

```
Signed: <dir>/package.toml -> <dir>/package.toml.sig
```

**Errors:** Fails if `package.toml` does not exist in the directory, or if
the secret key is not exactly 32 bytes.

---

#### `package verify`

Verify a package manifest's Ed25519 signature.

```
fidius package verify --key <PUBLIC_KEY_PATH> <DIR>
```

| Argument / Flag | Type | Required | Description |
|-----------------|------|----------|-------------|
| `--key` | path | yes | Path to the 32-byte public key file. |
| `DIR` | positional | yes | Path to the package directory. |

**Output on success:**

```
Signature valid: <dir>/package.toml
```

**Output on failure:**

```
Signature INVALID: <dir>/package.toml
```

Exits with code `1` on invalid signature.

**Errors:** Fails if `package.toml` does not exist in the directory, or if
the public key or signature file is malformed.

---

## See Also

- [Host API Reference](./host-api.md) -- programmatic API used by `inspect`
- [ABI Specification](./abi-specification.md) -- descriptor layout shown by `inspect`
- [Errors Reference](./errors.md) -- `LoadError` and `PackageError` variants
- [Package Manifest Reference](./package-manifest.md) -- `package.toml` format and Rust types
