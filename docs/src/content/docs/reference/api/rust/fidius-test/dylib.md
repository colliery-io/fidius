---
title: "fidius-test::dylib"
description: "Build-and-cache helpers for plugin cdylib fixtures."
editUrl: false
sidebar:
  order: 52
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

Build-and-cache helpers for plugin cdylib fixtures.

Integration tests frequently need to invoke `cargo build` on a plugin
crate, locate the produced `.dylib`/`.so`/`.dll`, and point a
`PluginHost` at its containing directory. Doing this from scratch in
every test is noisy and slow — each test re-builds the plugin even
though the source hasn't changed.

[`dylib_fixture`](../../../rust/fidius-test/dylib/#fn.dylib_fixture) returns a process-wide cached build result: the first
call builds the plugin; subsequent calls in the same test binary return
the existing path without re-running cargo. Fresh `cargo test`
invocations still rebuild (on cache miss in cargo's own target dir).

**Examples:**

```rust
let fixture = dylib_fixture("./path/to/my-plugin").build();
let host = PluginHost::builder()
    .search_path(fixture.dir())
    .build()?;
```

## Structs

### <span id="struct.DylibFixtureBuilder"></span><code style="font-size: 0.85em">struct DylibFixtureBuilder</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

Builder for [`DylibFixture`](../../../rust/fidius-test/dylib/#struct.DylibFixture). See [`dylib_fixture`](../../../rust/fidius-test/dylib/#fn.dylib_fixture).

#### Methods

##### <span id="DylibFixtureBuilder.with_release"></span><code>with_​release</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn with_release(mut self, release: bool) -> Self
```

Build in release mode. Defaults to the test binary's own profile (release if tests are built with `--release`, otherwise debug).

<details>
<summary>Source</summary>

```rust
    pub fn with_release(mut self, release: bool) -> Self {
        self.release = release;
        self
    }
```

</details>

##### <span id="DylibFixtureBuilder.signed_with"></span><code>signed_​with</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn signed_with(mut self, key: &SigningKey) -> Self
```

Sign the produced dylib with `key`, writing a `.sig` file alongside it.

Only takes effect on the first (un-cached) build — subsequent cached
fixtures are returned unchanged. For tests that need re-signing,
re-sign via [`crate::signing::sign_dylib`](../../../rust/fidius-test/signing/#fn.sign_dylib) on the returned
[`DylibFixture::dylib_path`](../../../rust/fidius-test/dylib/#DylibFixture.dylib_path).

<details>
<summary>Source</summary>

```rust
    pub fn signed_with(mut self, key: &SigningKey) -> Self {
        self.signing_key = Some(key.clone());
        self
    }
```

</details>

##### <span id="DylibFixtureBuilder.build"></span><code>build</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn build(self) -> DylibFixture
```

Execute the build (or return cached result) and produce the fixture.

Panics on build failure — tests should not attempt recovery from a
plugin that won't compile.

<details>
<summary>Source</summary>

```rust
    pub fn build(self) -> DylibFixture {
        let cache = cache();
        let key = CacheKey {
            plugin_dir: self.plugin_dir.clone(),
            release: self.release,
        };

        {
            let guard = cache.lock().expect("dylib fixture cache poisoned");
            if let Some(existing) = guard.get(&key) {
                // Cached. Signing was handled on first build; ignore new key.
                return existing.clone();
            }
        }

        let fixture = build_uncached(&self.plugin_dir, self.release);
        if let Some(signing_key) = &self.signing_key {
            sign_dylib(&fixture.dylib_path, signing_key)
                .expect("sign_dylib failed for fixture plugin");
        }

        cache
            .lock()
            .expect("dylib fixture cache poisoned")
            .insert(key, fixture.clone());
        fixture
    }
```

</details>

### <span id="struct.DylibFixture"></span><code style="font-size: 0.85em">struct DylibFixture</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

**Derives:** `Debug`, `Clone`

A built plugin ready to be loaded by a `PluginHost`.

#### Methods

##### <span id="DylibFixture.dir"></span><code>dir</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn dir(&self) -> &Path
```

Directory containing the built dylib — `search_path` for `PluginHost`.

<details>
<summary>Source</summary>

```rust
    pub fn dir(&self) -> &Path {
        &self.plugin_output_dir
    }
```

</details>

##### <span id="DylibFixture.dylib_path"></span><code>dylib_​path</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn dylib_path(&self) -> &Path
```

Full path to the dylib file itself. Use this to sign, inspect, or load the dylib directly (e.g., `fidius_host::loader::load_library`).

<details>
<summary>Source</summary>

```rust
    pub fn dylib_path(&self) -> &Path {
        &self.dylib_path
    }
```

</details>

## Functions

### <span id="fn.dylib_fixture"></span><code style="font-size: 0.85em">fn dylib_​fixture</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn dylib_fixture(plugin_dir: impl Into<PathBuf>) -> DylibFixtureBuilder
```

Start building a dylib fixture for the plugin crate at `plugin_dir`.

`plugin_dir` must contain a `Cargo.toml` with `crate-type = ["cdylib"]`.
The resulting fixture caches the build across the current test binary
process; subsequent calls with the same `plugin_dir` return the cached
fixture without re-running cargo.

<details>
<summary>Source</summary>

```rust
pub fn dylib_fixture(plugin_dir: impl Into<PathBuf>) -> DylibFixtureBuilder {
    DylibFixtureBuilder {
        plugin_dir: plugin_dir.into(),
        release: !cfg!(debug_assertions),
        signing_key: None,
    }
}
```

</details>
