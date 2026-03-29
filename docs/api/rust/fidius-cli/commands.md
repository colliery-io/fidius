# fidius-cli::commands <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


## Functions

### `fidius-cli::commands::resolve_dep`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn resolve_dep (value : & str , version_override : Option < & str >) -> String
```

Resolve a dependency string to a Cargo.toml dependency value.

Logic:
1. If `value` is a path that exists on disk → `{ path = "..." }`
2. If `version_override` is set → `"<version>"`
3. Check crates.io for `value` → if found, use latest version
4. Warn and fall back to `{ path = "<value>" }`

<details>
<summary>Source</summary>

```rust
fn resolve_dep(value: &str, version_override: Option<&str>) -> String {
    // Check if it's a filesystem path
    if Path::new(value).exists() {
        return format!("{{ path = \"{}\" }}", value);
    }

    // If version explicitly pinned, use it
    if let Some(ver) = version_override {
        return format!("\"{}\"", ver);
    }

    // Try crates.io
    if let Some(ver) = check_crates_io(value) {
        return format!("\"{}\"", ver);
    }

    // Warn and fall back to path dep
    eprintln!(
        "warning: could not find '{}' as a local path or on crates.io, using path dep",
        value
    );
    format!("{{ path = \"{}\" }}", value)
}
```

</details>



### `fidius-cli::commands::check_crates_io`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


```rust
fn check_crates_io (name : & str) -> Option < String >
```

Check crates.io for a crate and return its latest version, if found.

<details>
<summary>Source</summary>

```rust
fn check_crates_io(name: &str) -> Option<String> {
    let url = format!("https://crates.io/api/v1/crates/{}", name);
    let mut response = ureq::get(&url)
        .header(
            "User-Agent",
            "fidius-cli (https://github.com/fidius-rs/fidius)",
        )
        .call()
        .ok()?;

    let body_str = response.body_mut().read_to_string().ok()?;
    let body: serde_json::Value = serde_json::from_str(&body_str).ok()?;
    body["crate"]["max_stable_version"]
        .as_str()
        .map(String::from)
}
```

</details>



### `fidius-cli::commands::init_interface`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn init_interface (name : & str , trait_name : & str , path : Option < & Path > , version : Option < & str > ,) -> Result
```

<details>
<summary>Source</summary>

```rust
pub fn init_interface(
    name: &str,
    trait_name: &str,
    path: Option<&Path>,
    version: Option<&str>,
) -> Result {
    let base = path.unwrap_or_else(|| Path::new("."));
    let crate_dir = base.join(name);

    if crate_dir.exists() {
        return Err(format!("directory '{}' already exists", crate_dir.display()).into());
    }

    let src_dir = crate_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    // Resolve fidius dependency
    let fidius_dep = resolve_dep("fidius", version);

    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
fidius = {fidius_dep}
"#
    );

    let lib_rs = format!(
        r#"pub use fidius::{{plugin_impl, PluginError}};

#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
pub trait {trait_name}: Send + Sync {{
    fn process(&self, input: String) -> String;
}}
"#
    );

    std::fs::write(crate_dir.join("Cargo.toml"), cargo_toml)?;
    std::fs::write(src_dir.join("lib.rs"), lib_rs)?;

    println!("Created interface crate: {}", crate_dir.display());
    Ok(())
}
```

</details>



### `fidius-cli::commands::init_plugin`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn init_plugin (name : & str , interface : & str , trait_name : & str , path : Option < & Path > , version : Option < & str > ,) -> Result
```

<details>
<summary>Source</summary>

```rust
pub fn init_plugin(
    name: &str,
    interface: &str,
    trait_name: &str,
    path: Option<&Path>,
    version: Option<&str>,
) -> Result {
    let base = path.unwrap_or_else(|| Path::new("."));
    let crate_dir = base.join(name);

    if crate_dir.exists() {
        return Err(format!("directory '{}' already exists", crate_dir.display()).into());
    }

    let src_dir = crate_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    // Resolve interface dependency
    let interface_dep = resolve_dep(interface, version);

    // Extract the crate name from the interface value (strip path components)
    let interface_crate = Path::new(interface)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(interface);

    // Convert crate name to Rust identifier (hyphens → underscores)
    let interface_mod = interface_crate.replace('-', "_");

    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
{interface_crate} = {interface_dep}
fidius-core = {{ version = "0.1" }}
"#
    );

    let struct_name = format!("My{trait_name}");

    let lib_rs = format!(
        r#"use {interface_mod}::{{plugin_impl, {trait_name}, PluginError}};

pub struct {struct_name};

#[plugin_impl({trait_name})]
impl {trait_name} for {struct_name} {{
    fn process(&self, input: String) -> String {{
        todo!("implement {trait_name}")
    }}
}}

fidius_core::fidius_plugin_registry!();
"#
    );

    std::fs::write(crate_dir.join("Cargo.toml"), cargo_toml)?;
    std::fs::write(src_dir.join("lib.rs"), lib_rs)?;

    println!("Created plugin crate: {}", crate_dir.display());
    Ok(())
}
```

</details>



### `fidius-cli::commands::keygen`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn keygen (out : & str) -> Result
```

<details>
<summary>Source</summary>

```rust
pub fn keygen(out: &str) -> Result {
    use rand::rngs::OsRng;

    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    let secret_path = format!("{}.secret", out);
    let public_path = format!("{}.public", out);

    std::fs::write(&secret_path, signing_key.to_bytes())?;
    std::fs::write(&public_path, verifying_key.to_bytes())?;

    println!("Generated keypair:");
    println!("  Secret: {}", secret_path);
    println!("  Public: {}", public_path);
    Ok(())
}
```

</details>



### `fidius-cli::commands::sign`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn sign (key_path : & Path , dylib_path : & Path) -> Result
```

<details>
<summary>Source</summary>

```rust
pub fn sign(key_path: &Path, dylib_path: &Path) -> Result {
    let key_bytes: [u8; 32] = std::fs::read(key_path)?
        .try_into()
        .map_err(|_| "secret key must be exactly 32 bytes")?;

    let signing_key = SigningKey::from_bytes(&key_bytes);
    let dylib_bytes = std::fs::read(dylib_path)?;
    let signature = signing_key.sign(&dylib_bytes);

    let sig_path = dylib_path.with_extension(format!(
        "{}.sig",
        dylib_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
    ));

    std::fs::write(&sig_path, signature.to_bytes())?;
    println!("Signed: {} -> {}", dylib_path.display(), sig_path.display());
    Ok(())
}
```

</details>



### `fidius-cli::commands::verify`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn verify (key_path : & Path , dylib_path : & Path) -> Result
```

<details>
<summary>Source</summary>

```rust
pub fn verify(key_path: &Path, dylib_path: &Path) -> Result {
    let key_bytes: [u8; 32] = std::fs::read(key_path)?
        .try_into()
        .map_err(|_| "public key must be exactly 32 bytes")?;

    let verifying_key =
        VerifyingKey::from_bytes(&key_bytes).map_err(|e| format!("invalid public key: {e}"))?;

    let dylib_bytes = std::fs::read(dylib_path)?;

    let sig_path = dylib_path.with_extension(format!(
        "{}.sig",
        dylib_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
    ));

    let sig_bytes: [u8; 64] = std::fs::read(&sig_path)
        .map_err(|_| format!("signature file not found: {}", sig_path.display()))?
        .try_into()
        .map_err(|_| "signature must be exactly 64 bytes")?;

    let signature = Signature::from_bytes(&sig_bytes);

    match verifying_key.verify(&dylib_bytes, &signature) {
        Ok(()) => {
            println!("Signature valid: {}", dylib_path.display());
            Ok(())
        }
        Err(_) => {
            eprintln!("Signature INVALID: {}", dylib_path.display());
            std::process::exit(1);
        }
    }
}
```

</details>



### `fidius-cli::commands::inspect`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn inspect (dylib_path : & Path) -> Result
```

<details>
<summary>Source</summary>

```rust
pub fn inspect(dylib_path: &Path) -> Result {
    let loaded = fidius_host::loader::load_library(dylib_path)
        .map_err(|e| format!("failed to load {}: {e}", dylib_path.display()))?;

    println!("Plugin Registry: {}", dylib_path.display());
    println!("  Plugins: {}", loaded.plugins.len());
    println!();

    for (i, plugin) in loaded.plugins.iter().enumerate() {
        let info = &plugin.info;
        println!("  [{}] {}", i, info.name);
        println!("      Interface: {}", info.interface_name);
        println!("      Interface hash: {:#018x}", info.interface_hash);
        println!("      Interface version: {}", info.interface_version);
        println!("      Buffer strategy: {:?}", info.buffer_strategy);
        println!("      Wire format: {:?}", info.wire_format);
        println!("      Capabilities: {:#018x}", info.capabilities);
    }

    Ok(())
}
```

</details>



### `fidius-cli::commands::package_validate`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn package_validate (dir : & Path) -> Result
```

<details>
<summary>Source</summary>

```rust
pub fn package_validate(dir: &Path) -> Result {
    let manifest = fidius_core::package::load_manifest_untyped(dir)?;
    let pkg = &manifest.package;

    println!("Package: {} v{}", pkg.name, pkg.version);
    println!("  Interface: {} (version {})", pkg.interface, pkg.interface_version);
    if let Some(hash) = &pkg.source_hash {
        println!("  Source hash: {}", hash);
    }
    if !manifest.dependencies.is_empty() {
        println!("  Dependencies:");
        for (name, req) in &manifest.dependencies {
            println!("    {} = \"{}\"", name, req);
        }
    }
    println!(
        "  Metadata: {} field(s)",
        manifest.metadata.as_table().map_or(0, |t| t.len())
    );
    println!("\nManifest valid.");
    Ok(())
}
```

</details>



### `fidius-cli::commands::package_build`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn package_build (dir : & Path , release : bool) -> Result
```

<details>
<summary>Source</summary>

```rust
pub fn package_build(dir: &Path, release: bool) -> Result {
    let manifest = fidius_core::package::load_manifest_untyped(dir)?;
    let cargo_toml = dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(format!("Cargo.toml not found in {}", dir.display()).into());
    }

    println!(
        "Building package: {} v{}",
        manifest.package.name, manifest.package.version
    );

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build").arg("--manifest-path").arg(&cargo_toml);
    if release {
        cmd.arg("--release");
    }

    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("build failed:\n{}", stderr).into());
    }

    let profile = if release { "release" } else { "debug" };
    println!(
        "Build successful. Output in {}/target/{}/",
        dir.display(),
        profile
    );
    Ok(())
}
```

</details>



### `fidius-cli::commands::package_inspect`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn package_inspect (dir : & Path) -> Result
```

<details>
<summary>Source</summary>

```rust
pub fn package_inspect(dir: &Path) -> Result {
    let manifest = fidius_core::package::load_manifest_untyped(dir)?;
    let pkg = &manifest.package;

    println!("Package: {}", dir.display());
    println!("  Name: {}", pkg.name);
    println!("  Version: {}", pkg.version);
    println!("  Interface: {}", pkg.interface);
    println!("  Interface version: {}", pkg.interface_version);
    if let Some(hash) = &pkg.source_hash {
        println!("  Source hash: {}", hash);
    }
    if !manifest.dependencies.is_empty() {
        println!("  Dependencies:");
        for (name, req) in &manifest.dependencies {
            println!("    {} = \"{}\"", name, req);
        }
    }
    if let Some(table) = manifest.metadata.as_table() {
        println!("  Metadata:");
        for (key, value) in table {
            println!("    {} = {}", key, value);
        }
    }
    Ok(())
}
```

</details>



### `fidius-cli::commands::package_sign`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn package_sign (key_path : & Path , dir : & Path) -> Result
```

<details>
<summary>Source</summary>

```rust
pub fn package_sign(key_path: &Path, dir: &Path) -> Result {
    let manifest_path = dir.join("package.toml");
    if !manifest_path.exists() {
        return Err(format!("package.toml not found in {}", dir.display()).into());
    }
    sign(key_path, &manifest_path)
}
```

</details>



### `fidius-cli::commands::package_verify`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #4caf50; color: white;">pub</span>


```rust
fn package_verify (key_path : & Path , dir : & Path) -> Result
```

<details>
<summary>Source</summary>

```rust
pub fn package_verify(key_path: &Path, dir: &Path) -> Result {
    let manifest_path = dir.join("package.toml");
    if !manifest_path.exists() {
        return Err(format!("package.toml not found in {}", dir.display()).into());
    }
    verify(key_path, &manifest_path)
}
```

</details>



