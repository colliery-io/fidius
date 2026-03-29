# fidius-host::host <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


PluginHost builder and plugin discovery.

## Structs

### `fidius-host::host::PluginHost`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Host for loading and managing plugins.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `search_paths` | `Vec < PathBuf >` |  |
| `load_policy` | `LoadPolicy` |  |
| `require_signature` | `bool` |  |
| `trusted_keys` | `Vec < VerifyingKey >` |  |
| `expected_hash` | `Option < u64 >` |  |
| `expected_wire` | `Option < WireFormat >` |  |
| `expected_strategy` | `Option < BufferStrategyKind >` |  |

#### Methods

##### `builder` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn builder () -> PluginHostBuilder
```

Create a new builder.

<details>
<summary>Source</summary>

```rust
    pub fn builder() -> PluginHostBuilder {
        PluginHostBuilder::new()
    }
```

</details>



##### `discover` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn discover (& self) -> Result < Vec < PluginInfo > , LoadError >
```

Discover all valid plugins in the configured search paths.

Scans directories for dylib files, loads each, validates,
and returns metadata for all valid plugins found.

<details>
<summary>Source</summary>

```rust
    pub fn discover(&self) -> Result<Vec<PluginInfo>, LoadError> {
        let mut plugins = Vec::new();

        for search_path in &self.search_paths {
            if !search_path.is_dir() {
                continue;
            }

            let entries = std::fs::read_dir(search_path)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if !is_dylib(&path) {
                    continue;
                }

                match loader::load_library(&path) {
                    Ok(loaded) => {
                        for plugin in &loaded.plugins {
                            if let Ok(()) = loader::validate_against_interface(
                                plugin,
                                self.expected_hash,
                                self.expected_wire,
                                self.expected_strategy,
                            ) {
                                plugins.push(plugin.info.clone());
                            }
                        }
                    }
                    Err(_) => {
                        // Skip invalid dylibs during discovery
                        continue;
                    }
                }
            }
        }

        Ok(plugins)
    }
```

</details>



##### `load` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn load (& self , name : & str) -> Result < LoadedPlugin , LoadError >
```

Load a specific plugin by name.

Searches all configured paths for a dylib containing a plugin
with the given name. Returns the loaded plugin ready for calling.

<details>
<summary>Source</summary>

```rust
    pub fn load(&self, name: &str) -> Result<LoadedPlugin, LoadError> {
        for search_path in &self.search_paths {
            if !search_path.is_dir() {
                continue;
            }

            let entries = std::fs::read_dir(search_path)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if !is_dylib(&path) {
                    continue;
                }

                // Verify signature if required
                if self.require_signature {
                    match signing::verify_signature(&path, &self.trusted_keys) {
                        Ok(()) => {}
                        Err(e) if self.load_policy == LoadPolicy::Lenient => {
                            eprintln!("fidius warning: {e}");
                        }
                        Err(e) => return Err(e),
                    }
                }

                match loader::load_library(&path) {
                    Ok(loaded) => {
                        for plugin in loaded.plugins {
                            if plugin.info.name == name {
                                loader::validate_against_interface(
                                    &plugin,
                                    self.expected_hash,
                                    self.expected_wire,
                                    self.expected_strategy,
                                )?;
                                return Ok(plugin);
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        }

        Err(LoadError::PluginNotFound {
            name: name.to_string(),
        })
    }
```

</details>





### `fidius-host::host::PluginHostBuilder`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


Builder for configuring a PluginHost.

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `search_paths` | `Vec < PathBuf >` |  |
| `load_policy` | `LoadPolicy` |  |
| `require_signature` | `bool` |  |
| `trusted_keys` | `Vec < VerifyingKey >` |  |
| `expected_hash` | `Option < u64 >` |  |
| `expected_wire` | `Option < WireFormat >` |  |
| `expected_strategy` | `Option < BufferStrategyKind >` |  |

#### Methods

##### `new` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn new () -> Self
```

<details>
<summary>Source</summary>

```rust
    fn new() -> Self {
        Self {
            search_paths: Vec::new(),
            load_policy: LoadPolicy::Strict,
            require_signature: false,
            trusted_keys: Vec::new(),
            expected_hash: None,
            expected_wire: None,
            expected_strategy: None,
        }
    }
```

</details>



##### `search_path` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn search_path (mut self , path : impl Into < PathBuf >) -> Self
```

Add a directory to search for plugin dylibs.

<details>
<summary>Source</summary>

```rust
    pub fn search_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.search_paths.push(path.into());
        self
    }
```

</details>



##### `load_policy` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn load_policy (mut self , policy : LoadPolicy) -> Self
```

Set the load policy (Strict or Lenient).

<details>
<summary>Source</summary>

```rust
    pub fn load_policy(mut self, policy: LoadPolicy) -> Self {
        self.load_policy = policy;
        self
    }
```

</details>



##### `require_signature` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn require_signature (mut self , require : bool) -> Self
```

Require plugins to have valid signatures.

<details>
<summary>Source</summary>

```rust
    pub fn require_signature(mut self, require: bool) -> Self {
        self.require_signature = require;
        self
    }
```

</details>



##### `trusted_keys` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn trusted_keys (mut self , keys : & [VerifyingKey]) -> Self
```

Set trusted Ed25519 public keys for signature verification.

<details>
<summary>Source</summary>

```rust
    pub fn trusted_keys(mut self, keys: &[VerifyingKey]) -> Self {
        self.trusted_keys = keys.to_vec();
        self
    }
```

</details>



##### `interface_hash` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn interface_hash (mut self , hash : u64) -> Self
```

Set the expected interface hash for validation.

<details>
<summary>Source</summary>

```rust
    pub fn interface_hash(mut self, hash: u64) -> Self {
        self.expected_hash = Some(hash);
        self
    }
```

</details>



##### `wire_format` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn wire_format (mut self , format : WireFormat) -> Self
```

Set the expected wire format for validation.

<details>
<summary>Source</summary>

```rust
    pub fn wire_format(mut self, format: WireFormat) -> Self {
        self.expected_wire = Some(format);
        self
    }
```

</details>



##### `buffer_strategy` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn buffer_strategy (mut self , strategy : BufferStrategyKind) -> Self
```

Set the expected buffer strategy for validation.

<details>
<summary>Source</summary>

```rust
    pub fn buffer_strategy(mut self, strategy: BufferStrategyKind) -> Self {
        self.expected_strategy = Some(strategy);
        self
    }
```

</details>



##### `build` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn build (self) -> Result < PluginHost , LoadError >
```

Build the PluginHost.

<details>
<summary>Source</summary>

```rust
    pub fn build(self) -> Result<PluginHost, LoadError> {
        Ok(PluginHost {
            search_paths: self.search_paths,
            load_policy: self.load_policy,
            require_signature: self.require_signature,
            trusted_keys: self.trusted_keys,
            expected_hash: self.expected_hash,
            expected_wire: self.expected_wire,
            expected_strategy: self.expected_strategy,
        })
    }
```

</details>





## Functions

### `fidius-host::host::is_dylib`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn is_dylib (path : & Path) -> bool
```

Check if a path has a platform-appropriate dylib extension.

<details>
<summary>Source</summary>

```rust
fn is_dylib(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if cfg!(target_os = "macos") {
        ext == "dylib"
    } else if cfg!(target_os = "windows") {
        ext == "dll"
    } else {
        ext == "so"
    }
}
```

</details>



