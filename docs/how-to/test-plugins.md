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

# Test Plugins

Fidius ships a dedicated testing crate, `fidius-test`, with helpers for the
three workflows plugin and host authors need: in-process unit tests, full
dylib integration tests, and signed-plugin flows. This guide walks through
each layer.

All helpers are `pub` in `fidius-test`. Add it under `[dev-dependencies]`:

```toml
[dev-dependencies]
fidius-test = "0.0.5"
```

The `fidius init-plugin` and `fidius init-host` scaffolds add this automatically.

## Layer 1: in-process tests (fastest)

The generated `{Trait}Client::in_process(name)` constructor invokes a plugin
method **without** compiling a cdylib or loading a dylib. Uses inventory to
find the plugin descriptor in the current test binary's address space.

When to use: quick feedback during plugin development. A `cargo test` cycle
finishes in milliseconds because there's no dylib build and no `dlopen`.

```rust
// In your plugin's src/lib.rs
use my_plugin_api::{plugin_impl, MyTrait};

pub struct MyPlugin;

#[plugin_impl(MyTrait)]
impl MyTrait for MyPlugin {
    fn process(&self, input: String) -> String {
        format!("processed: {input}")
    }
}

fidius::fidius_plugin_registry!();

#[cfg(test)]
mod tests {
    use my_plugin_api::MyTraitClient;

    #[test]
    fn process_works() {
        let client = MyTraitClient::in_process("MyPlugin").expect("registered");
        let out = client.process(&"hello".to_string()).unwrap();
        assert_eq!(out, "processed: hello");
    }
}
```

The test imports the client from the **interface crate**. For the client to
be visible in tests, declare the interface crate as a `[dev-dependencies]`
entry with the `host` feature enabled (the `fidius init-plugin` scaffold
does this automatically):

```toml
[dependencies]
my-plugin-api = { path = "..." }              # cdylib build: no host feature
fidius = { version = "..." }

[dev-dependencies]
my-plugin-api = { path = "...", features = ["host"] }   # tests: host feature on
fidius-test = "0.0.5"
```

Cargo unifies features per target, so the cdylib build does not pull
`libloading` through the `host` feature.

`Client::in_process` returns `Err(LoadError::PluginNotFound)` if no
`#[plugin_impl]` with that struct name is linked into the test binary.

## Layer 2: dylib integration tests

For end-to-end tests that exercise the full FFI path — including dylib
compilation, `dlopen`, descriptor validation, and signature verification —
use `fidius_test::dylib_fixture`.

```rust
use fidius_host::{PluginHandle, PluginHost};
use fidius_test::dylib_fixture;
use my_plugin_api::MyTraitClient;

#[test]
fn loads_and_calls_plugin() {
    let fixture = dylib_fixture("../my-plugin").build();

    let host = PluginHost::builder()
        .search_path(fixture.dir())
        .build()
        .unwrap();

    let loaded = host.load("MyPlugin").unwrap();
    let handle = PluginHandle::from_loaded(loaded);
    let client = MyTraitClient::from_handle(handle);

    let out = client.process(&"hello".to_string()).unwrap();
    assert_eq!(out, "processed: hello");
}
```

`dylib_fixture` invokes `cargo build` on the plugin crate once per test
binary process — subsequent calls with the same path return the cached
build. Multiple tests in the same file share one build cycle.

## Layer 3: signed-plugin flows

Signature verification is normally painful to test because it needs a real
keypair. `fidius_test` provides deterministic fixtures:

```rust
use fidius_host::PluginHost;
use fidius_test::{dylib_fixture, fixture_keypair};

#[test]
fn signed_plugin_with_trusted_key() {
    let (signing_key, verifying_key) = fixture_keypair();

    let fixture = dylib_fixture("../my-plugin")
        .signed_with(&signing_key)
        .build();

    let host = PluginHost::builder()
        .search_path(fixture.dir())
        .require_signature(true)
        .trusted_keys(&[verifying_key])
        .build()
        .unwrap();

    host.load("MyPlugin").expect("signed with trusted key");
}
```

Available helpers:

| Helper | Purpose |
|---|---|
| `fixture_keypair()` | Deterministic `(SigningKey, VerifyingKey)` derived from a fixed seed |
| `fixture_keypair_with_seed(u8)` | Deterministic keypair with a caller-chosen seed — use different seeds for "wrong key" tests |
| `sign_dylib(path, &key)` | Write a `.sig` file next to a dylib (same convention as `fidius sign`) |
| `DylibFixtureBuilder::signed_with(&key)` | Sign the dylib during fixture build |

These keys are **not secure** — they exist only so tests can exercise the
signing verification path deterministically. Never use them in production.

## Layer 4: CLI smoke test

For a zero-setup smoke test of a plugin package, use `fidius test`:

```bash
$ fidius test ./my-plugin
Built: ./my-plugin/target/debug/libmy_plugin.dylib

Plugin: MyPlugin (interface MyTrait v1, 2 methods)
  [0] — needs input (method takes args)
  [1] ✓ invoked (output decoded as JSON)

Smoke passed: 1 plugin(s), 1 zero-arg method(s) invoked cleanly
```

This builds the package, loads it, and attempts to invoke each method with
a zero-argument input. It's a smoke test — it verifies the FFI round-trip
works, not that your method returns correct values. Methods with arguments
are reported but not failed.

Useful as a CI pre-check and during local development to catch descriptor
or registry issues without writing code.

## Which layer should I write?

| Question | Answer |
|---|---|
| Fastest feedback while writing plugin logic | Layer 1 (in-process) |
| Verify the full FFI round-trip works | Layer 2 (dylib fixture) |
| Test signed-plugin host-side policy | Layer 3 (signing fixtures) |
| "Does my plugin even build and load?" | Layer 4 (`fidius test`) |

Write Layer 1 tests for nearly every method — they're nearly free. Add
Layer 2 integration tests for critical paths. Use Layer 3 when testing
signature policy. Use Layer 4 as a CI smoke gate.
