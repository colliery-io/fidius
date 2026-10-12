---
title: "fidius-test"
description: "Testing helpers for Fidius plugin authors and hosts."
editUrl: false
sidebar:
  order: 51
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Testing helpers for Fidius plugin authors and hosts.

This crate provides the infrastructure the Fidius codebase uses internally
for its own tests, now exposed so downstream users don't have to reinvent
the wheel. Add it under `[dev-dependencies]` and you get:

- [`dylib_fixture`](../../rust/fidius-test/dylib/#fn.dylib_fixture) — build a plugin crate's cdylib via `cargo build`,
  cached across tests in the same process. Optional signing via
  [`DylibFixtureBuilder::signed_with`](../../rust/fidius-test/dylib/#DylibFixtureBuilder.signed_with).
- [`signing::fixture_keypair`](../../rust/fidius-test/signing/#fn.fixture_keypair) — deterministic Ed25519 keypair for tests.
- [`signing::sign_dylib`](../../rust/fidius-test/signing/#fn.sign_dylib) — produce a `.sig` file next to a dylib.

**Examples:**

```rust
use fidius_test::dylib_fixture;
use fidius_host::PluginHost;

#[test]
fn loads_plugin() {
    let fixture = dylib_fixture("./path/to/my-plugin").build();
    let host = PluginHost::builder()
        .search_path(fixture.dir())
        .build()
        .unwrap();
    let plugins = host.discover().unwrap();
    assert!(!plugins.is_empty());
}
```

## Re-exports

- [`dylib_fixture`](../../rust/fidius-test/dylib/#fn.dylib_fixture)
- [`DylibFixture`](../../rust/fidius-test/dylib/#struct.DylibFixture)
- [`DylibFixtureBuilder`](../../rust/fidius-test/dylib/#struct.DylibFixtureBuilder)
- [`fixture_keypair`](../../rust/fidius-test/signing/#fn.fixture_keypair)
- [`fixture_keypair_with_seed`](../../rust/fidius-test/signing/#fn.fixture_keypair_with_seed)
- [`sign_dylib`](../../rust/fidius-test/signing/#fn.sign_dylib)
- [`collect`](../../rust/fidius-test/stream/#fn.collect)
- [`pump`](../../rust/fidius-test/stream/#fn.pump)
- [`stream_of`](../../rust/fidius-test/stream/#fn.stream_of)
- [`CollectSink`](../../rust/fidius-test/stream/#struct.CollectSink)
- [`StreamSink`](../../rust/fidius-test/stream/#trait.StreamSink)
