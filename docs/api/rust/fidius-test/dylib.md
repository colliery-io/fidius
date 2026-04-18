# fidius-test::dylib <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


Build-and-cache helpers for plugin cdylib fixtures.

Integration tests frequently need to invoke `cargo build` on a plugin
crate, locate the produced `.dylib`/`.so`/`.dll`, and point a
`PluginHost` at its containing directory. Doing this from scratch in
every test is noisy and slow — each test re-builds the plugin even
though the source hasn't changed.
[`dylib_fixture`] returns a process-wide cached build result: the first
call builds the plugin; subsequent calls in the same test binary return
the existing path without re-running cargo. Fresh `cargo test`
invocations still rebuild (on cache miss in cargo's own target dir).

**Examples:**

```ignore
let fixture = dylib_fixture("./path/to/my-plugin").build();
let host = PluginHost::builder()
    .search_path(fixture.dir())
    .build()?;
```

## Structs

### `fidius-test::dylib::DylibFixtureBuilder`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Builder for [`DylibFixture`]. See [`dylib_fixture`].

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `plugin_dir` | `PathBuf` |  |
| `release` | `bool` |  |
| `signing_key` | `Option < SigningKey >` |  |

#### Methods

##### `with_release` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn with_release (mut self , release : bool) -> Self
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



##### `signed_with` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn signed_with (mut self , key : & SigningKey) -> Self
```

Sign the produced dylib with `key`, writing a `.sig` file alongside it.

Only takes effect on the first (un-cached) build — subsequent cached
fixtures are returned unchanged. For tests that need re-signing,
re-sign via [`crate::signing::sign_dylib`] on the returned
[`DylibFixture::dylib_path`].

<details>
<summary>Source</summary>

```rust
    pub fn signed_with(mut self, key: &SigningKey) -> Self {
        self.signing_key = Some(key.clone());
        self
    }
```

</details>



##### `build` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn build (self) -> DylibFixture
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





### `fidius-test::dylib::DylibFixture`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


**Derives:** `Debug`, `Clone`

A built plugin ready to be loaded by a `PluginHost`.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `plugin_output_dir` | `PathBuf` | Directory containing the built dylib. Pass this to
`PluginHost::builder().search_path(...)`. |
| `dylib_path` | `PathBuf` | Full path to the built dylib file itself. |

#### Methods

##### `dir` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn dir (& self) -> & Path
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



##### `dylib_path` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn dylib_path (& self) -> & Path
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





### `fidius-test::dylib::CacheKey`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


**Derives:** `Hash`, `PartialEq`, `Eq`, `Clone`

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `plugin_dir` | `PathBuf` |  |
| `release` | `bool` |  |



## Functions

### `fidius-test::dylib::dylib_fixture`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn dylib_fixture (plugin_dir : impl Into < PathBuf >) -> DylibFixtureBuilder
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



### `fidius-test::dylib::cache`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn cache () -> & 'static Mutex < HashMap < CacheKey , DylibFixture > >
```

<details>
<summary>Source</summary>

```rust
fn cache() -> &'static Mutex<HashMap<CacheKey, DylibFixture>> {
    static CACHE: OnceLock<Mutex<HashMap<CacheKey, DylibFixture>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}
```

</details>



### `fidius-test::dylib::dylib_extension`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn dylib_extension () -> & 'static str
```

<details>
<summary>Source</summary>

```rust
fn dylib_extension() -> &'static str {
    if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(target_os = "windows") {
        "dll"
    } else {
        "so"
    }
}
```

</details>



### `fidius-test::dylib::build_uncached`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn build_uncached (plugin_dir : & Path , release : bool) -> DylibFixture
```

<details>
<summary>Source</summary>

```rust
fn build_uncached(plugin_dir: &Path, release: bool) -> DylibFixture {
    let manifest = plugin_dir.join("Cargo.toml");
    assert!(manifest.exists(), "no Cargo.toml at {}", manifest.display());

    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--manifest-path").arg(&manifest);
    if release {
        cmd.arg("--release");
    }

    let output = cmd.output().expect("failed to spawn cargo build");
    assert!(
        output.status.success(),
        "cargo build of {} failed:\n{}",
        plugin_dir.display(),
        String::from_utf8_lossy(&output.stderr),
    );

    let profile = if release { "release" } else { "debug" };
    let plugin_output_dir = plugin_dir.join("target").join(profile);

    // Find the dylib — first file with the right extension
    let ext = dylib_extension();
    let dylib_path = std::fs::read_dir(&plugin_output_dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", plugin_output_dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().and_then(|s| s.to_str()) == Some(ext))
        .unwrap_or_else(|| {
            panic!(
                "build succeeded but no .{} file found in {}",
                ext,
                plugin_output_dir.display()
            )
        });

    DylibFixture {
        plugin_output_dir,
        dylib_path,
    }
}
```

</details>



