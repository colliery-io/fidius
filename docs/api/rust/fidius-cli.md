# fidius-cli <span class="plissken-badge plissken-badge-source" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: #ff5722; color: white;">Rust</span>


## Structs

### `fidius-cli::Cli`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


**Derives:** `Parser`

#### Fields

| Name | Type | Description |
|------|------|-------------|
| `command` | `Commands` |  |



## Enums

### `fidius-cli::Commands` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


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



### `fidius-cli::PackageCommands` <span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


#### Variants

- **`Validate`** - Validate a package manifest
- **`Build`** - Build a package (compile the cdylib)
- **`Inspect`** - Inspect a package manifest
- **`Sign`** - Sign a package manifest
- **`Verify`** - Verify a package manifest signature
- **`Pack`** - Pack a package directory into a .fid archive
- **`Unpack`** - Unpack a .fid archive



## Functions

### `fidius-cli::main`

<span class="plissken-badge plissken-badge-visibility" style="display: inline-block; padding: 0.1em 0.35em; font-size: 0.55em; font-weight: 600; border-radius: 0.2em; vertical-align: middle; background: var(--md-default-fg-color--light); color: white;">private</span>


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
            PackageCommands::Pack { dir, output } => {
                commands::package_pack(&dir, output.as_deref())
            }
            PackageCommands::Unpack { archive, dest } => {
                commands::package_unpack(&archive, dest.as_deref())
            }
        },
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
```

</details>



