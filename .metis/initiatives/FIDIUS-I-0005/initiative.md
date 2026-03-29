---
id: fidius-cli-developer-tooling-and
level: initiative
title: "fidius-cli — Developer Tooling and Signing"
short_code: "FIDIUS-I-0005"
created_at: 2026-03-29T00:26:20.823874+00:00
updated_at: 2026-03-29T11:52:45.831914+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
initiative_id: fidius-cli-developer-tooling-and
---

# fidius-cli — Developer Tooling and Signing

## Context

The CLI makes fidius usable as a product, not just a library. `cargo install fidius-cli` gives developers everything they need: scaffold an interface crate, scaffold a plugin crate, generate signing keys, sign plugins, verify signatures, and inspect compiled dylibs. Without it, developers have to manually set up the crate topology, which is error-prone.

Depends on FIDIUS-I-0001 (core types for inspect), FIDIUS-I-0003 (host loading for inspect), and conceptually on FIDIUS-I-0004 (the facade crate that scaffolded projects depend on).

## Goals & Non-Goals

**Goals:**
- `fides init-interface <name> --trait <TraitName>` — scaffold an interface crate with correct Cargo.toml (depends on `fidius`) and lib.rs (re-exports + annotated trait stub)
- `fides init-plugin <name> --interface <crate>` — scaffold a plugin crate with `crate-type = ["cdylib"]`, correct dependency, and impl stub
- `fides keygen --out <name>` — generate Ed25519 keypair (`.secret` + `.public` files)
- `fides sign --key <secret> <dylib>` — produce detached `.sig` file
- `fides verify --key <public> <dylib>` — verify signature, exit 0/1
- `fides inspect <dylib>` — load registry, dump: plugin count, names, interface names, interface hashes, versions, capabilities, wire format, buffer strategy, signature status
- Good CLI UX: `clap` derive API, colored output, helpful error messages

**Non-Goals:**
- Plugin compilation (that's `cargo build`)
- Plugin distribution / registry (future work)
- Watch mode / hot-reload tooling

## Detailed Design

### Scaffolding

`init-interface` generates:
```
<name>/
├── Cargo.toml     # [dependencies] fidius = "0.1"
└── src/
    └── lib.rs     # pub use fides::{plugin_impl, PluginError};
                   # #[fides::plugin_interface(version = 1, buffer = PluginAllocated)]
                   # pub trait <TraitName>: Send + Sync { ... }
```

`init-plugin` generates:
```
<name>/
├── Cargo.toml     # [lib] crate-type = ["cdylib"]
│                  # [dependencies] <interface> = { path = "../<interface>" }
└── src/
    └── lib.rs     # use <interface>::{plugin_impl, <TraitName>, PluginError};
                   # pub struct MyPlugin;
                   # #[plugin_impl(<TraitName>)]
                   # impl <TraitName> for MyPlugin { ... }
```

Both accept `--path` to control output directory. Interface crate path in plugin Cargo.toml defaults to `../<interface>` but can be overridden.

### Signing

- `keygen`: Generate Ed25519 keypair via `ed25519-dalek`, write secret key to `<name>.secret`, public key to `<name>.public`
- `sign`: Read dylib bytes, sign with secret key, write `<dylib>.sig`
- `verify`: Read dylib bytes + `.sig`, verify against public key

### Inspect

Loads the dylib via fidius-host's registry loading (without calling any plugin methods), displays:
```
Plugin Registry: my_plugin.dylib
  Magic: FIDES (valid)
  Registry version: 1
  Plugins: 2

  [0] BlurFilter
      Interface: ImageFilter
      Interface hash: 0xA3F2...
      Interface version: 1
      Buffer strategy: PluginAllocated
      Wire format: bincode (release)
      Capabilities: process_with_metadata (bit 0)
      Signature: VALID (key: 7a3f...)

  [1] SharpenFilter
      ...
```

### Dependencies

- `clap` (derive) — CLI framework
- `ed25519-dalek` — key generation + signing
- `fidius-host` — for inspect (registry loading)
- `fidius-core` — shared types

## Testing Strategy

- Snapshot tests for scaffolded file contents (assert generated Cargo.toml and lib.rs match expected)
- Round-trip tests: keygen → sign → verify succeeds; sign → tamper → verify fails
- Inspect tests: build a test cdylib, run inspect, assert output contains expected fields
- CLI integration tests via `assert_cmd`

## Implementation Plan

1. Scaffold crate with `clap` derive
2. Implement `init-interface` scaffolding
3. Implement `init-plugin` scaffolding
4. Implement `keygen` command
5. Implement `sign` command
6. Implement `verify` command
7. Implement `inspect` command (depends on fidius-host loading)
8. CLI integration tests