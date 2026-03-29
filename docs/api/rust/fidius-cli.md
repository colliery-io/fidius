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
- **`Keygen`** - Generate an Ed25519 signing keypair
- **`Sign`** - Sign a plugin dylib
- **`Verify`** - Verify a plugin dylib signature
- **`Inspect`** - Inspect a plugin dylib's registry



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
        } => commands::init_interface(&name, &trait_name, path.as_deref(), version.as_deref()),
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
        Commands::Keygen { out } => commands::keygen(&out),
        Commands::Sign { key, dylib } => commands::sign(&key, &dylib),
        Commands::Verify { key, dylib } => commands::verify(&key, &dylib),
        Commands::Inspect { dylib } => commands::inspect(&dylib),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
```

</details>



