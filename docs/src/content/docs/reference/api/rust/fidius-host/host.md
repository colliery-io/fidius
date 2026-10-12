---
title: "fidius-host::host"
description: "PluginHost builder and plugin discovery."
editUrl: false
sidebar:
  order: 38
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>

PluginHost builder and plugin discovery.

## Structs

### <span id="struct.PluginHost"></span><code style="font-size: 0.85em">struct PluginHost</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

Host for loading and managing plugins.

#### Methods

##### <span id="PluginHost.builder"></span><code>builder</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn builder() -> PluginHostBuilder
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

##### <span id="PluginHost.discover"></span><code>discover</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn discover(&self) -> Result<Vec<PluginInfo>, LoadError>
```

Discover all valid plugins in the configured search paths.

Scans each path for both:
- dylib files (cdylib plugins, the existing path), and
- subdirectories containing a `package.toml` with `runtime = "python"`
  (when the `python` feature is enabled).

Returns owned `PluginInfo` for every valid plugin found, with
`PluginInfo::runtime` distinguishing the two kinds.

<details>
<summary>Source</summary>

```rust
    pub fn discover(&self) -> Result<Vec<PluginInfo>, LoadError> {
        #[cfg(feature = "tracing")]
        tracing::info!(search_paths = ?self.search_paths, "discovering plugins");

        let mut plugins = Vec::new();

        for search_path in &self.search_paths {
            if !search_path.is_dir() {
                continue;
            }

            let entries = std::fs::read_dir(search_path)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if is_dylib(&path) {
                    self.discover_cdylib(&path, &mut plugins);
                } else if path.is_dir() && path.join("package.toml").exists() {
                    self.discover_package(&path, &mut plugins);
                }
            }
        }

        Ok(plugins)
    }
```

</details>

##### <span id="PluginHost.load"></span><code>load</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn load(&self, name: &str) -> Result<LoadedPlugin, LoadError>
```

Load a specific plugin by name.

Searches all configured paths for a dylib containing a plugin
with the given name. Returns the loaded plugin ready for calling.

<details>
<summary>Source</summary>

```rust
    pub fn load(&self, name: &str) -> Result<LoadedPlugin, LoadError> {
        #[cfg(feature = "tracing")]
        tracing::info!(plugin_name = name, "loading plugin");

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

                // Verify signature if required — always enforced regardless of LoadPolicy
                if self.require_signature {
                    signing::verify_signature(&path, &self.trusted_keys)?;
                }

                match loader::load_library(&path) {
                    Ok(loaded) => {
                        for plugin in loaded.plugins {
                            if plugin.info.name == name {
                                loader::validate_against_interface(
                                    &plugin,
                                    self.expected_hash,
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

##### <span id="PluginHost.find_python_package"></span><code>find_​python_​package</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn find_python_package(&self, name: &str) -> Result<PathBuf, LoadError>
```

Find a python plugin package directory by name across the configured search paths. The plugin name is matched against `package.toml`'s `[package].name`. Returns the directory path on success.

<details>
<summary>Source</summary>

```rust
    pub fn find_python_package(&self, name: &str) -> Result<PathBuf, LoadError> {
        for search_path in &self.search_paths {
            if !search_path.is_dir() {
                continue;
            }
            let entries = std::fs::read_dir(search_path)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                if !path.join("package.toml").exists() {
                    continue;
                }
                let Ok(manifest) = fidius_core::package::load_manifest_untyped(&path) else {
                    continue;
                };
                if matches!(
                    manifest.package.runtime(),
                    fidius_core::package::PackageRuntime::Python
                ) && manifest.package.name == name
                {
                    return Ok(path);
                }
            }
        }
        Err(LoadError::PluginNotFound {
            name: name.to_string(),
        })
    }
```

</details>

##### <span id="PluginHost.load_python"></span><code>load_​python</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn load_python(
    &self,
    name: &str,
    descriptor: &'static fidius_core::python_descriptor::PythonInterfaceDescriptor,
) -> Result<crate::handle::PluginHandle, LoadError>
```

Load a Python plugin package by name and validate it against the supplied interface descriptor.

The caller passes the static `<TraitName>_PYTHON_DESCRIPTOR` emitted
by the interface crate's `#[plugin_interface]` macro — that's the
out-of-band hint the loader needs to map method names to vtable
indices and to check the interface hash.

Available only when fidius-host is built with the `python` feature.

<details>
<summary>Source</summary>

```rust
    pub fn load_python(
        &self,
        name: &str,
        descriptor: &'static fidius_core::python_descriptor::PythonInterfaceDescriptor,
    ) -> Result<crate::handle::PluginHandle, LoadError> {
        let dir = self.find_python_package(name)?;
        // Signature policy — enforced identically to cdylib/WASM loads.
        if self.require_signature {
            signing::verify_package_signature(&dir, &self.trusted_keys)?;
        }
        let manifest = fidius_core::package::load_manifest_untyped(&dir)
            .map_err(|e| LoadError::PythonLoad(e.to_string()))?;
        let py = fidius_python::load_python_plugin(&dir, descriptor)
            .map_err(|e| LoadError::PythonLoad(e.to_string()))?;
        // Build the host-facing metadata from the manifest header + the
        // interface descriptor. `capabilities`/`buffer_strategy` are cdylib
        // concepts and take their no-op defaults for Python.
        let info = crate::types::PluginInfo {
            name: manifest.package.name.clone(),
            interface_name: descriptor.interface_name.to_string(),
            interface_hash: descriptor.interface_hash,
            interface_version: manifest.package.interface_version,
            capabilities: 0,
            buffer_strategy: fidius_core::descriptor::BufferStrategyKind::PluginAllocated,
            runtime: crate::types::PluginRuntimeKind::Python,
        };
        Ok(crate::handle::PluginHandle::from_python(py, info))
    }
```

</details>

##### <span id="PluginHost.load_python_configured"></span><code>load_​python_​configured</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn load_python_configured<C: serde::Serialize>(
    &self,
    name: &str,
    descriptor: &'static fidius_core::python_descriptor::PythonInterfaceDescriptor,
    config: &C,
) -> Result<crate::handle::PluginHandle, LoadError>
```

Load a **configured** Python plugin (FIDIUS-A-0006 / CI.4): serialize `config` and bind it once via the module's `__fidius_configure__(config) -> instance`; methods then run on the configured instance. N differently-configured instances coexist. Available only with the `python` feature.

<details>
<summary>Source</summary>

```rust
    pub fn load_python_configured<C: serde::Serialize>(
        &self,
        name: &str,
        descriptor: &'static fidius_core::python_descriptor::PythonInterfaceDescriptor,
        config: &C,
    ) -> Result<crate::handle::PluginHandle, LoadError> {
        let dir = self.find_python_package(name)?;
        if self.require_signature {
            signing::verify_package_signature(&dir, &self.trusted_keys)?;
        }
        let manifest = fidius_core::package::load_manifest_untyped(&dir)
            .map_err(|e| LoadError::PythonLoad(e.to_string()))?;
        let cfg = serde_json::to_value(config)
            .map_err(|e| LoadError::PythonLoad(format!("config serialize: {e}")))?;
        let py = fidius_python::load_python_plugin_configured(&dir, descriptor, &cfg)
            .map_err(|e| LoadError::PythonLoad(e.to_string()))?;
        let info = crate::types::PluginInfo {
            name: manifest.package.name.clone(),
            interface_name: descriptor.interface_name.to_string(),
            interface_hash: descriptor.interface_hash,
            interface_version: manifest.package.interface_version,
            capabilities: 0,
            buffer_strategy: fidius_core::descriptor::BufferStrategyKind::PluginAllocated,
            runtime: crate::types::PluginRuntimeKind::Python,
        };
        Ok(crate::handle::PluginHandle::from_python(py, info))
    }
```

</details>

##### <span id="PluginHost.find_wasm_package"></span><code>find_​wasm_​package</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn find_wasm_package(&self, name: &str) -> Result<PathBuf, LoadError>
```

Find a WASM package directory by name across the search paths (matches `package.toml` `[package].name` with `runtime = "wasm"`).

<details>
<summary>Source</summary>

```rust
    pub fn find_wasm_package(&self, name: &str) -> Result<PathBuf, LoadError> {
        for search_path in &self.search_paths {
            if !search_path.is_dir() {
                continue;
            }
            for entry in std::fs::read_dir(search_path)? {
                let entry = entry?;
                let path = entry.path();
                if !path.is_dir() || !path.join("package.toml").exists() {
                    continue;
                }
                let Ok(manifest) = fidius_core::package::load_manifest_untyped(&path) else {
                    continue;
                };
                if matches!(
                    manifest.package.runtime(),
                    fidius_core::package::PackageRuntime::Wasm
                ) && manifest.package.name == name
                {
                    return Ok(path);
                }
            }
        }
        Err(LoadError::PluginNotFound {
            name: name.to_string(),
        })
    }
```

</details>

##### <span id="PluginHost.load_wasm"></span><code>load_​wasm</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn load_wasm(
    &self,
    name: &str,
    descriptor: &'static fidius_core::wasm_descriptor::WasmInterfaceDescriptor,
) -> Result<crate::handle::PluginHandle, LoadError>
```

Load a WASM component plugin package by name and validate it against the supplied interface descriptor (the `<TraitName>_WASM_DESCRIPTOR` the interface crate emits). Returns a unified [`crate::handle::PluginHandle`](../../../rust/fidius-host/handle/#struct.PluginHandle).

The component is sandboxed: WASI is wired into the `Linker` but the guest
gets a zero-grant `WasiCtx` (no FS preopens, no env, no sockets). The
capability allow-list in `[wasm].capabilities` is applied in T-0104.

Outbound HTTP is governed by the host's egress policy: a guest that
declares the `http` capability gets `wasi:http` only when the host was
given a policy (via [`PluginHostBuilder::egress`](../../../rust/fidius-host/host/#PluginHostBuilder.egress) or
`Self::load_wasm_with_egress`) — otherwise it fails closed.

Available only with the `wasm` feature.

<details>
<summary>Source</summary>

```rust
    pub fn load_wasm(
        &self,
        name: &str,
        descriptor: &'static fidius_core::wasm_descriptor::WasmInterfaceDescriptor,
    ) -> Result<crate::handle::PluginHandle, LoadError> {
        self.load_wasm_impl(name, descriptor, self.egress.clone(), None, None)
    }
```

</details>

##### <span id="PluginHost.load_wasm_configured"></span><code>load_​wasm_​configured</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn load_wasm_configured<C: serde::Serialize>(
    &self,
    name: &str,
    descriptor: &'static fidius_core::wasm_descriptor::WasmInterfaceDescriptor,
    config: &C,
) -> Result<crate::handle::PluginHandle, LoadError>
```

Load a **configured** WASM plugin (FIDIUS-A-0006 / CI.3): serialize `config` and bind it once via the guest's `fidius-configure` export. The component then runs on a persistent store, so methods dispatch on the configured instance and config crosses the sandbox boundary exactly once. Available only with the `wasm` feature.

<details>
<summary>Source</summary>

```rust
    pub fn load_wasm_configured<C: serde::Serialize>(
        &self,
        name: &str,
        descriptor: &'static fidius_core::wasm_descriptor::WasmInterfaceDescriptor,
        config: &C,
    ) -> Result<crate::handle::PluginHandle, LoadError> {
        let cfg = fidius_core::wire::serialize(config)
            .map_err(|e| LoadError::WasmLoad(format!("config serialize: {e}")))?;
        self.load_wasm_impl(name, descriptor, self.egress.clone(), Some(&cfg), None)
    }
```

</details>

##### <span id="PluginHost.load_wasm_configured_with_grants"></span><code>load_​wasm_​configured_​with_​grants</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn load_wasm_configured_with_grants<C: serde::Serialize>(
    &self,
    name: &str,
    descriptor: &'static fidius_core::wasm_descriptor::WasmInterfaceDescriptor,
    config: &C,
    capabilities: Vec<String>,
    egress: Option<Arc<dyn crate::executor::wasm::EgressPolicy>>,
) -> Result<crate::handle::PluginHandle, LoadError>
```

Load a **configured** WASM plugin with a caller-supplied capability allow-list and egress policy, overriding the package manifest's `[wasm].capabilities` (FIDIUS-I-0033 / cloacina constructor grants).

This is the entry point for embedders that authorize a plugin's host
access *at load time* rather than trusting the (signed-but-author-chosen)
manifest caps — e.g. cloacina's tenant-granted constructor capabilities,
where the filesystem path / env key / http+tcp intent are decided by the
deploying tenant, not the plugin author. `capabilities` fully **replaces**
the manifest list (it is not merged): pass exactly what the tenant
granted. Default-closed falls out naturally — an empty `capabilities`
builds a zero-grant `WasiCtx`, and `egress: None` denies all brokered
HTTP/TCP (the two-key gate's host key is absent).

`capabilities` uses the same vocabulary as `[wasm].capabilities`
(`http`, `tcp`, `udp`, `network`/`sockets`, `fs:ro:<path>`/`fs:rw:<path>`,
`env:<NAME>`, …) and is validated identically; an unknown or coarse
(`env` / `fs`) entry fails the load.

Available only with the `wasm` feature.

<details>
<summary>Source</summary>

```rust
    pub fn load_wasm_configured_with_grants<C: serde::Serialize>(
        &self,
        name: &str,
        descriptor: &'static fidius_core::wasm_descriptor::WasmInterfaceDescriptor,
        config: &C,
        capabilities: Vec<String>,
        egress: Option<Arc<dyn crate::executor::wasm::EgressPolicy>>,
    ) -> Result<crate::handle::PluginHandle, LoadError> {
        let cfg = fidius_core::wire::serialize(config)
            .map_err(|e| LoadError::WasmLoad(format!("config serialize: {e}")))?;
        self.load_wasm_impl(name, descriptor, egress, Some(&cfg), Some(capabilities))
    }
```

</details>

##### <span id="PluginHost.load_wasm_with_egress"></span><code>load_​wasm_​with_​egress</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn load_wasm_with_egress(
    &self,
    name: &str,
    descriptor: &'static fidius_core::wasm_descriptor::WasmInterfaceDescriptor,
    egress: impl crate::executor::wasm::EgressPolicy,
) -> Result<crate::handle::PluginHandle, LoadError>
```

Like `Self::load_wasm` but with a **per-plugin** `wasi:http` egress policy that overrides any host-wide default (FIDIUS-I-0027). This is the right primitive for isolating connectors: a host-wide policy only sees the outbound *request*, not which plugin issued it, so per-plugin policies are how you bound connector A to one set of hosts and connector B to another.

<details>
<summary>Source</summary>

```rust
    pub fn load_wasm_with_egress(
        &self,
        name: &str,
        descriptor: &'static fidius_core::wasm_descriptor::WasmInterfaceDescriptor,
        egress: impl crate::executor::wasm::EgressPolicy,
    ) -> Result<crate::handle::PluginHandle, LoadError> {
        self.load_wasm_impl(name, descriptor, Some(Arc::new(egress)), None, None)
    }
```

</details>

### <span id="struct.PluginHostBuilder"></span><code style="font-size: 0.85em">struct PluginHostBuilder</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

Builder for configuring a PluginHost.

#### Methods

##### <span id="PluginHostBuilder.egress"></span><code>egress</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn egress(
    mut self,
    policy: impl crate::executor::wasm::EgressPolicy,
) -> Self
```

Set a host-wide default `wasi:http` egress policy (FIDIUS-I-0027). Every `load_wasm` then enables outbound HTTP for a guest that declares the `http` capability, routing each request through `policy`. Without this (and without a per-load policy), `wasi:http` is never linked and a guest that imports it fails closed. Available only with the `wasm` feature.

<details>
<summary>Source</summary>

```rust
    pub fn egress(mut self, policy: impl crate::executor::wasm::EgressPolicy) -> Self {
        self.egress = Some(Arc::new(policy));
        self
    }
```

</details>

##### <span id="PluginHostBuilder.egress_policy"></span><code>egress_​policy</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn egress_policy(
    mut self,
    policy: Arc<dyn crate::executor::wasm::EgressPolicy>,
) -> Self
```

Like `Self::egress` but accepts an already-erased `Arc<dyn EgressPolicy>` — for a policy that is shared across hosts, or selected/constructed at runtime (where the concrete type isn't known at the call site). Threads through `load_wasm` exactly like `egress`. Available only with the `wasm` feature.

<details>
<summary>Source</summary>

```rust
    pub fn egress_policy(mut self, policy: Arc<dyn crate::executor::wasm::EgressPolicy>) -> Self {
        self.egress = Some(policy);
        self
    }
```

</details>

##### <span id="PluginHostBuilder.search_path"></span><code>search_​path</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn search_path(mut self, path: impl Into<PathBuf>) -> Self
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

##### <span id="PluginHostBuilder.load_policy"></span><code>load_​policy</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn load_policy(mut self, policy: LoadPolicy) -> Self
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

##### <span id="PluginHostBuilder.require_signature"></span><code>require_​signature</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn require_signature(mut self, require: bool) -> Self
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

##### <span id="PluginHostBuilder.trusted_keys"></span><code>trusted_​keys</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn trusted_keys(mut self, keys: &[VerifyingKey]) -> Self
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

##### <span id="PluginHostBuilder.interface_hash"></span><code>interface_​hash</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn interface_hash(mut self, hash: u64) -> Self
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

##### <span id="PluginHostBuilder.buffer_strategy"></span><code>buffer_​strategy</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn buffer_strategy(mut self, strategy: BufferStrategyKind) -> Self
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

##### <span id="PluginHostBuilder.build"></span><code>build</code>

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-green-low); color: var(--sl-color-green-high);">pub</span>

```rust
fn build(self) -> Result<PluginHost, LoadError>
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
            expected_strategy: self.expected_strategy,
            #[cfg(feature = "wasm")]
            egress: self.egress,
        })
    }
```

</details>
