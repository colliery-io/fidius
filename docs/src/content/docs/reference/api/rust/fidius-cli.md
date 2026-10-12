---
title: "fidius-cli"
description: "API reference of the Rust module fidius-cli."
sidebar:
  order: 2
  hidden: true
---

<span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-orange-low); color: var(--sl-color-orange-high);">Rust</span>


## Structs

### <span id="struct.Cli"></span>`struct Cli`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


**Derives:** `Parser`

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `command` | `Commands` |  |



## Enums

### <span id="enum.Commands"></span>`enum Commands`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


#### Variants

- **`InitInterface`** - Scaffold a new plugin interface crate
- **`InitPlugin`** - Scaffold a new plugin implementation crate
- **`InitHost`** - Scaffold a new host application crate that uses the typed Client
- **`Keygen`** - Generate an Ed25519 signing keypair
- **`Sign`** - Sign a plugin dylib
- **`Verify`** - Verify a plugin dylib signature
- **`Inspect`** - Inspect a plugin dylib's registry
- **`Test`** - Smoke-test a plugin: build, load, and invoke each method with a zero-arg input
- **`Package`** - Package management commands
- **`PythonStub`** - Generate a Python stub for an interface trait so a Python plugin
author has type-hinted method signatures plus the matching
`__interface_hash__` constant.
- **`Wit`** - Generate the WIT for a WASM plugin crate from its source (the `build.rs`
`fidius_build::emit_wit()` does this automatically; this is for CI/manual use).



### <span id="enum.PackageCommands"></span>`enum PackageCommands`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


#### Variants

- **`Validate`** - Validate a package manifest
- **`Build`** - Build a package (compile the cdylib)
- **`Inspect`** - Inspect a package manifest
- **`Sign`** - Sign a package manifest
- **`Verify`** - Verify a package manifest signature
- **`Pack`** - Pack a package directory into a .fid archive
- **`Unpack`** - Unpack a .fid archive



## Functions

### <span id="fn.main"></span>`fn main`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.05em 0.6em; font-family: var(--sl-font-mono); font-size: var(--sl-text-xs); line-height: 1.6; border-radius: var(--radius-pill, 1rem); vertical-align: middle; background: var(--sl-color-gray-6); color: var(--sl-color-gray-2);">private</span>


```rust
fn main ()
```

<details>
<summary>Source</summary>

```rust
fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::InitInterface {
            name,
            trait_name,
            path,
            version,
            extension,
        } => commands::init_interface(
            &name,
            &trait_name,
            path.as_deref(),
            version.as_deref(),
            extension.as_deref(),
        ),
        Commands::InitPlugin {
            name,
            interface,
            trait_name,
            path,
            version,
        } => commands::init_plugin(
            &name,
            &interface,
            &trait_name,
            path.as_deref(),
            version.as_deref(),
        ),
        Commands::InitHost {
            name,
            interface,
            trait_name,
            path,
            version,
        } => commands::init_host(
            &name,
            &interface,
            &trait_name,
            path.as_deref(),
            version.as_deref(),
        ),
        Commands::Keygen { out } => commands::keygen(&out),
        Commands::Sign { key, dylib } => commands::sign(&key, &dylib),
        Commands::Verify { key, dylib } => commands::verify(&key, &dylib),
        Commands::Inspect { dylib } => commands::inspect(&dylib),
        Commands::Test { dir, debug } => commands::test(&dir, !debug),
        Commands::Package { command } => match command {
            PackageCommands::Validate { dir } => commands::package_validate(&dir),
            PackageCommands::Build { dir, debug } => commands::package_build(&dir, !debug),
            PackageCommands::Inspect { dir } => commands::package_inspect(&dir),
            PackageCommands::Sign { key, dir } => commands::package_sign(&key, &dir),
            PackageCommands::Verify { key, dir } => commands::package_verify(&key, &dir),
            PackageCommands::Pack {
                dir,
                output,
                precompile,
            } => commands::package_pack(&dir, output.as_deref(), precompile),
            PackageCommands::Unpack { archive, dest } => {
                commands::package_unpack(&archive, dest.as_deref())
            }
        },
        Commands::PythonStub {
            interface,
            out,
            trait_name,
        } => commands::python_stub(&interface, &out, trait_name.as_deref()),
        Commands::Wit { dir } => commands::wit(dir.as_deref()),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
```

</details>



