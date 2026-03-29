<!--
Copyright 2026 Colliery, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
-->

# Signing Plugins

This tutorial walks through the full plugin signing workflow: generating an
Ed25519 (a public-key signature algorithm) keypair, signing a compiled plugin
dylib, verifying the signature from the command line, and configuring the host
to require valid signatures at load time.

Signing ensures that only plugins produced by a trusted party can be loaded.
If a plugin file is tampered with after signing, or if it was signed with an
untrusted key, the host will reject it.

## Prerequisites

- Completed [Your First Plugin](your-first-plugin.md)
- The `fidius` CLI installed (`cargo install fidius-cli`)
- A compiled plugin dylib (e.g. `libcalculator_plugin.dylib` from the previous
  tutorials)

## What you will learn

1. Generate an Ed25519 signing keypair
2. Sign a plugin dylib
3. Verify the signature from the CLI
4. Configure the host to require signatures
5. Understand what happens with wrong keys, missing signatures, and tampered
   files

## Step 1: Generate a keypair

The `fidius keygen` command generates an Ed25519 keypair and writes two files:

```bash
fidius keygen --out mykey
```

Output:

```
Generated keypair:
  Secret: mykey.secret
  Public: mykey.public
```

- `mykey.secret` -- 32-byte Ed25519 secret key. You need it to sign plugins.
- `mykey.public` -- 32-byte Ed25519 public key. Distribute this to hosts that
  need to verify your plugins.

> **Security note:** Treat `mykey.secret` like a password. Store it outside
> version control, restrict file permissions (`chmod 600 mykey.secret`), and
> consider using a secrets manager in CI/CD pipelines. Anyone with access to
> the secret key can sign plugins that your host will trust.

## Step 2: Build the plugin

If you haven't already, build the plugin from the
[Your First Plugin](your-first-plugin.md) tutorial:

```bash
cargo build -p calculator-plugin
```

The dylib will be at a path like:

| Platform | Path |
|---|---|
| macOS | `target/debug/libcalculator_plugin.dylib` |
| Linux | `target/debug/libcalculator_plugin.so` |
| Windows | `target/debug/calculator_plugin.dll` |

The examples below use the macOS path. Substitute your platform's extension as
needed.

## Step 3: Sign the plugin

```bash
fidius sign --key mykey.secret target/debug/libcalculator_plugin.dylib
```

Output:

```
Signed: target/debug/libcalculator_plugin.dylib -> target/debug/libcalculator_plugin.dylib.sig
```

The signature is written to a detached `.sig` file next to the dylib. The
naming convention appends `.sig` to the full filename including extension
(e.g. `libcalculator_plugin.dylib.sig`).

The signature covers the entire contents of the dylib file. If even a single
byte changes after signing, verification will fail.

## Step 4: Verify the signature from the CLI

```bash
fidius verify --key mykey.public target/debug/libcalculator_plugin.dylib
```

Output on success:

```
Signature valid: target/debug/libcalculator_plugin.dylib
```

The `verify` command reads the dylib, reads the `.sig` file from the expected
location, and checks the Ed25519 signature against the provided public key. If
verification fails, it prints `Signature INVALID` and exits with code 1.

## Step 5: Configure the host to require signatures

First, add `ed25519-dalek` to the host's `Cargo.toml`:

```toml
[dependencies]
fidius-host = { version = "0.1" }
serde = { version = "1", features = ["derive"] }
ed25519-dalek = "2"
```

Then update `calculator-host/src/main.rs` to load the public key and require
signature verification:

```rust
use fidius_host::{PluginHost, PluginHandle};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct AddInput { a: i64, b: i64 }

#[derive(Deserialize, Debug)]
struct AddOutput { result: i64 }

fn main() {
    let plugin_dir = std::env::args()
        .nth(1)
        .expect("usage: calculator-host <plugin-dir>");

    // Load the public key (32 bytes).
    let key_bytes: [u8; 32] = std::fs::read("mykey.public")
        .expect("could not read mykey.public")
        .try_into()
        .expect("public key must be exactly 32 bytes");

    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&key_bytes)
        .expect("invalid public key");

    let host = PluginHost::builder()
        .search_path(&plugin_dir)
        .require_signature(true)
        .trusted_keys(&[verifying_key])
        .build()
        .expect("failed to build plugin host");

    let loaded = host
        .load("BasicCalculator")
        .expect("failed to load BasicCalculator");

    let handle = PluginHandle::from_loaded(loaded);

    let output: AddOutput = handle
        .call_method(0, &AddInput { a: 3, b: 7 })
        .expect("add() failed");

    println!("add(3, 7) = {}", output.result);
}
```

The two builder methods that control signing:

- `.require_signature(true)` -- the host will call `verify_signature()` on
  every dylib before loading it. If the `.sig` file is missing, loading fails
  with `LoadError::SignatureRequired`.
- `.trusted_keys(&[verifying_key])` -- provides the Ed25519 public keys that
  the host trusts. The signature must verify against at least one of these
  keys; otherwise loading fails with `LoadError::SignatureInvalid`.

## Step 6: Build and run

```bash
cargo build
cargo run --bin calculator-host -- target/debug/
```

Expected output:

```
add(3, 7) = 10
```

## Step 7: See what happens when things go wrong

### Wrong key

Generate a second keypair and try to load a plugin signed with the first key
while the host trusts only the second key:

```bash
fidius keygen --out otherkey
```

Update the host to load `otherkey.public` instead of `mykey.public`, rebuild,
and run:

```
Error: signature verification failed for target/debug/libcalculator_plugin.dylib
```

The host returns `LoadError::SignatureInvalid`. The signature itself is valid
(it was produced with a real key), but it does not match any key in the
`trusted_keys` list.

### Missing signature file

Delete the `.sig` file and try to load:

```bash
rm target/debug/libcalculator_plugin.dylib.sig
cargo run --bin calculator-host -- target/debug/
```

```
Error: signature required but no .sig file found for target/debug/libcalculator_plugin.dylib
```

The host returns `LoadError::SignatureRequired`. When `require_signature(true)`
is set, every dylib must have a corresponding `.sig` file.

### Tampered file

Sign the plugin, then modify the dylib after signing:

```bash
fidius sign --key mykey.secret target/debug/libcalculator_plugin.dylib

# Append a byte to simulate tampering
echo -n "x" >> target/debug/libcalculator_plugin.dylib

cargo run --bin calculator-host -- target/debug/
```

```
Error: signature verification failed for target/debug/libcalculator_plugin.dylib
```

The host returns `LoadError::SignatureInvalid`. The signature was valid for the
original file contents, but the dylib has changed since signing.

Re-sign after any legitimate rebuild:

```bash
cargo build -p calculator-plugin
fidius sign --key mykey.secret target/debug/libcalculator_plugin.dylib
```

### No signature requirement

If you do not call `.require_signature(true)` on the builder, unsigned plugins
load normally. The default is `require_signature: false`:

```rust
let host = PluginHost::builder()
    .search_path(&plugin_dir)
    .build()
    .unwrap();
```

This loads any valid plugin regardless of whether a `.sig` file exists.

## Multiple trusted keys

You can trust multiple keys by passing a slice:

```rust
let host = PluginHost::builder()
    .search_path(&plugin_dir)
    .require_signature(true)
    .trusted_keys(&[key_a, key_b, key_c])
    .build()
    .unwrap();
```

A plugin signed with any one of these keys will load successfully. This is
useful for key rotation: add the new key to the trusted set before retiring the
old one.

For a complete list of signing-related error cases, see [Errors reference](../reference/errors.md).

## Next steps

- [Your First Plugin](your-first-plugin.md) -- review the basics
- [Optional Methods](optional-methods.md) -- evolve interfaces with
  backward-compatible optional methods
