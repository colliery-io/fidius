use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "fidius", about = "Fidius plugin framework CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold a new plugin interface crate
    InitInterface {
        /// Crate name for the interface
        name: String,
        /// Trait name to generate
        #[arg(long = "trait")]
        trait_name: String,
        /// Output directory (default: current dir)
        #[arg(long)]
        path: Option<PathBuf>,
        /// Pin fidius dependency version (checks crates.io if not a local path)
        #[arg(long)]
        version: Option<String>,
    },
    /// Scaffold a new plugin implementation crate
    InitPlugin {
        /// Crate name for the plugin
        name: String,
        /// Interface crate (local path, crates.io name, or crate name)
        #[arg(long)]
        interface: String,
        /// Trait name from the interface crate
        #[arg(long = "trait")]
        trait_name: String,
        /// Output directory (default: current dir)
        #[arg(long)]
        path: Option<PathBuf>,
        /// Pin interface dependency version (overrides auto-detection)
        #[arg(long)]
        version: Option<String>,
    },
    /// Generate an Ed25519 signing keypair
    Keygen {
        /// Output file base name (writes <name>.secret and <name>.public)
        #[arg(long)]
        out: String,
    },
    /// Sign a plugin dylib
    Sign {
        /// Path to the secret key file
        #[arg(long)]
        key: PathBuf,
        /// Path to the dylib to sign
        dylib: PathBuf,
    },
    /// Verify a plugin dylib signature
    Verify {
        /// Path to the public key file
        #[arg(long)]
        key: PathBuf,
        /// Path to the dylib to verify
        dylib: PathBuf,
    },
    /// Inspect a plugin dylib's registry
    Inspect {
        /// Path to the dylib to inspect
        dylib: PathBuf,
    },
}

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
