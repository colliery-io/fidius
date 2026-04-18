# fidius-test <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Testing helpers for Fidius plugin authors and hosts.

This crate provides the infrastructure the Fidius codebase uses internally
for its own tests, now exposed so downstream users don't have to reinvent
the wheel. Add it under `[dev-dependencies]` and you get:
- [`dylib_fixture`] — build a plugin crate's cdylib via `cargo build`,
cached across tests in the same process. Optional signing via
[`DylibFixtureBuilder::signed_with`].
- [`signing::fixture_keypair`] — deterministic Ed25519 keypair for tests.
- [`signing::sign_dylib`] — produce a `.sig` file next to a dylib.

**Examples:**

```ignore
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

