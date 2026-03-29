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

use std::path::PathBuf;
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
    /// Package management commands
    Package {
        #[command(subcommand)]
        command: PackageCommands,
    },
}

#[derive(Subcommand)]
enum PackageCommands {
    /// Validate a package manifest
    Validate {
        /// Path to the package directory
        dir: PathBuf,
    },
    /// Build a package (compile the cdylib)
    Build {
        /// Path to the package directory
        dir: PathBuf,
        /// Build in debug mode instead of release
        #[arg(long)]
        debug: bool,
    },
    /// Inspect a package manifest
    Inspect {
        /// Path to the package directory
        dir: PathBuf,
    },
    /// Sign a package manifest
    Sign {
        /// Path to the secret key file
        #[arg(long)]
        key: PathBuf,
        /// Path to the package directory
        dir: PathBuf,
    },
    /// Verify a package manifest signature
    Verify {
        /// Path to the public key file
        #[arg(long)]
        key: PathBuf,
        /// Path to the package directory
        dir: PathBuf,
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
        Commands::Package { command } => match command {
            PackageCommands::Validate { dir } => commands::package_validate(&dir),
            PackageCommands::Build { dir, debug } => commands::package_build(&dir, !debug),
            PackageCommands::Inspect { dir } => commands::package_inspect(&dir),
            PackageCommands::Sign { key, dir } => commands::package_sign(&key, &dir),
            PackageCommands::Verify { key, dir } => commands::package_verify(&key, &dir),
        },
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
