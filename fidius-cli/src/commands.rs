// Copyright 2026 Colliery, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::path::Path;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

// ─── Dependency resolution ───────────────────────────────────────────────────

/// Resolve a dependency string to a Cargo.toml dependency value.
///
/// Logic:
/// 1. If `value` is a path that exists on disk → `{ path = "..." }`
/// 2. If `version_override` is set → `"<version>"`
/// 3. Check crates.io for `value` → if found, use latest version
/// 4. Warn and fall back to `{ path = "<value>" }`
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

/// Check crates.io for a crate and return its latest version, if found.
fn check_crates_io(name: &str) -> Option<String> {
    let url = format!("https://crates.io/api/v1/crates/{}", name);
    let mut response = ureq::get(&url)
        .header("User-Agent", "fidius-cli (https://github.com/fidius-rs/fidius)")
        .call()
        .ok()?;

    let body_str = response.body_mut().read_to_string().ok()?;
    let body: serde_json::Value = serde_json::from_str(&body_str).ok()?;
    body["crate"]["max_stable_version"]
        .as_str()
        .map(String::from)
}

// ─── init-interface ──────────────────────────────────────────────────────────

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

// ─── init-plugin ─────────────────────────────────────────────────────────────

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

// ─── keygen ──────────────────────────────────────────────────────────────────

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

// ─── sign ────────────────────────────────────────────────────────────────────

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

// ─── verify ──────────────────────────────────────────────────────────────────

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

// ─── inspect ─────────────────────────────────────────────────────────────────

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
        println!(
            "      Buffer strategy: {:?}",
            info.buffer_strategy
        );
        println!("      Wire format: {:?}", info.wire_format);
        println!("      Capabilities: {:#018x}", info.capabilities);
    }

    Ok(())
}
