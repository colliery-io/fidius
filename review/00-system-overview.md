# System Overview: Fidius Plugin Framework

## Summary

Fidius is a Rust plugin framework that converts annotated Rust traits into dynamically loaded plugin libraries (cdylibs) with a stable C ABI. It is developed by Colliery, Inc. and licensed under Apache 2.0. The framework consists of five workspace crates plus a test fixture, organized around a clear separation between plugin authoring (macros + core types), host loading (runtime validation + FFI calling), and developer tooling (CLI).

The system is at version `0.0.0-alpha.1`. It currently supports only the `PluginAllocated` buffer strategy; `CallerAllocated` and `Arena` are defined in the type system but rejected at compile time. The repository name on disk is `fides` but all crate names use the `fidius` prefix. The upstream repository is `github.com/colliery-io/fidius`.

---

## Repository Structure

```
fides/                          # workspace root (git repo)
├── Cargo.toml                  # workspace manifest (resolver v2)
├── Cargo.lock                  # committed lockfile
├── LICENSE                     # Apache 2.0
├── .gitignore                  # ignores target/, Cargo.lock (note: Cargo.lock IS tracked)
├── .pre-commit-config.yaml     # license-header, rustfmt, clippy hooks
├── plissken.toml               # API doc generation config (plissken tool)
├── .angreal/                   # angreal task runner scripts (Python)
│   ├── task_build.py
│   ├── task_check.py
│   ├── task_license_header.py
│   ├── task_lint.py
│   └── task_test.py
├── .metis/                     # Project management documents
│   ├── code-index.md           # Auto-generated code index
│   ├── adrs/
│   │   └── FIDIUS-A-0001.md    # ADR: Package Format and Schema Validation
│   └── specifications/
│       └── FIDIUS-S-0001/
│           └── specification.md # ABI and Wire Protocol Specification
├── docs/                       # Diataxis-structured documentation
│   ├── index.md
│   ├── tutorials/
│   ├── how-to/
│   ├── reference/
│   ├── explanation/
│   └── api/
├── fidius-core/                # Shared types crate
├── fidius-macro/               # Proc-macro crate
├── fidius-host/                # Host-side loading/calling crate
├── fidius-cli/                 # CLI binary crate
├── fidius/                     # Facade crate (re-exports core + macro)
├── tests/
│   └── test-plugin-smoke/      # Standalone cdylib test fixture (excluded from workspace)
└── review/                     # Architecture review output (this document)
```

**Organizational principle**: The workspace is split by role in the plugin lifecycle. `fidius-core` holds shared ABI types, `fidius-macro` generates code at compile time, `fidius-host` handles runtime loading, and `fidius-cli` provides developer tooling. The `fidius` crate is a thin facade for plugin/interface authors to depend on a single crate.

The `test-plugin-smoke` fixture is explicitly excluded from the workspace (`exclude = ["tests/test-plugin-smoke"]`) because it must compile as a standalone cdylib.

---

## Key Entrypoints

### Binary Entrypoint
- **`fidius-cli/src/main.rs`** -- The `fidius` CLI binary. Uses clap derive to parse `Commands` enum, dispatches to functions in `commands.rs`. Exits with code 1 on error. No logging framework; output is via `println!`/`eprintln!`.

### Library Entrypoints
- **`fidius/src/lib.rs`** -- Facade crate. Re-exports `fidius_macro::{plugin_interface, plugin_impl}`, `fidius_core::descriptor::*`, `fidius_core::wire`, `fidius_core::status`, `fidius_core::hash`, `fidius_core::error::PluginError`, `fidius_core::registry`, `fidius_core::inventory`, and conditionally `fidius_core::async_runtime`.
- **`fidius-core/src/lib.rs`** -- Exports all modules. Re-exports `descriptor::*`, `error::PluginError`, `status::*`, `inventory`. The `async_runtime` module is behind the `async` feature flag.
- **`fidius-host/src/lib.rs`** -- Exports all modules. Re-exports key types: `LoadError`, `CallError`, `PluginHandle`, `PluginHost`, `LoadedLibrary`, `LoadedPlugin`, `LoadPolicy`, `PluginInfo`.
- **`fidius-macro/src/lib.rs`** -- Proc-macro crate. Exports two attribute macros: `plugin_interface` and `plugin_impl`.

### Runtime Entrypoint (Plugin Side)
- **`fidius_core::fidius_plugin_registry!()` macro** -- Must be called once in every plugin cdylib's `lib.rs`. Emits a `#[no_mangle] pub extern "C" fn fidius_get_registry()` symbol that the host resolves via `dlsym`.

### Runtime Entrypoint (Host Side)
- **`fidius_host::PluginHost::builder()`** -- Builder pattern entry point. Configure search paths, load policy, signature requirements, trusted keys, expected interface hash/wire format/buffer strategy, then `.build()`.
- **`fidius_host::loader::load_library(path)`** -- Low-level: opens a dylib, calls `fidius_get_registry()`, validates registry and descriptors.

---

## Architecture

### Crate Roles

| Crate | Type | Role |
|-------|------|------|
| `fidius-core` | lib | Shared ABI types (descriptors, registry, wire format, status codes, hashing, errors, package manifest). The only crate both plugin and host depend on. |
| `fidius-macro` | proc-macro | Two attribute macros: `#[plugin_interface]` generates vtable struct, interface hash, capability constants, descriptor builder function. `#[plugin_impl]` generates FFI shims, static vtable, descriptor, inventory registration. |
| `fidius` | lib (facade) | Re-exports `fidius-core` + `fidius-macro` so interface crates have a single dependency. |
| `fidius-host` | lib | Host-side: dynamic library loading via `libloading`, architecture detection, registry/descriptor validation, signature verification (Ed25519), `PluginHandle` for type-safe method calling, package management (manifest loading, discovery, building). |
| `fidius-cli` | bin | CLI tooling: scaffolding (`init-interface`, `init-plugin`), signing (`keygen`, `sign`, `verify`), inspection (`inspect`), package management (`package validate/build/inspect/sign/verify`). |

### Dependency Graph (Internal)

```
fidius-cli
├── fidius-core
├── fidius-host
│   └── fidius-core
├── clap, ed25519-dalek, ureq, serde, serde_json, rand

fidius-macro
├── fidius-core (compile-time hash computation)
├── syn, quote, proc-macro2

fidius (facade)
├── fidius-core
├── fidius-macro

fidius-core
├── serde, serde_json, bincode, thiserror, inventory, toml
├── tokio (optional, behind "async" feature)

test-plugin-smoke (standalone, excluded from workspace)
├── fidius (with "async" feature)
├── fidius-core (with "async" feature)
├── fidius-macro
├── serde
```

### Data Flow

**Plugin Compilation Flow:**
```
Trait definition (#[plugin_interface])
  → fidius-macro parses trait → InterfaceIR
  → Generates: cleaned trait, companion module (__fidius_{TraitName})
    containing: VTable struct, hash/version/strategy constants,
    capability bit constants, descriptor builder fn

Impl block (#[plugin_impl(TraitName)])
  → fidius-macro parses impl → MethodInfo[]
  → Generates: original impl, static instance, extern "C" shims (one per method),
    free_buffer fn, static vtable (using companion module's constructor),
    static PluginDescriptor (using companion module's builder fn),
    inventory::submit! registration

fidius_plugin_registry!()
  → Emits fidius_get_registry export
  → At runtime: inventory collects all DescriptorEntry submissions,
    builds PluginRegistry (OnceLock), returns pointer
```

**Plugin Loading Flow:**
```
Host calls PluginHost::load("PluginName")
  → Scans search_paths for *.dylib/*.so/*.dll files
  → For each dylib:
    1. check_architecture (reads binary header bytes)
    2. If require_signature: verify Ed25519 signature (.sig file)
       - Strict: reject on failure
       - Lenient: warn on failure, continue
    3. dlopen via libloading::Library::new
    4. dlsym("fidius_get_registry") → call → *const PluginRegistry
    5. Validate magic bytes (b"FIDIUS\0\0")
    6. Validate registry_version == 1
    7. For each descriptor in registry:
       a. Validate abi_version == 1
       b. Copy FFI strings to owned PluginInfo
       c. If interface hash/wire/strategy expectations set, validate
    8. If plugin.info.name matches requested name → return LoadedPlugin
  → Wrap in PluginHandle for method calling
```

**Method Call Flow:**
```
handle.call_method::<I, O>(index, &input)
  → wire::serialize(input) → input_bytes
  → Read vtable[index] as FfiFn pointer
  → Call FFI: fn(in_ptr, in_len, &mut out_ptr, &mut out_len) -> i32
  → Inside plugin (generated shim):
    - catch_unwind wraps everything
    - wire::deserialize(in_slice) → args
    - Call impl method: instance.method(args)
    - If returns Result: match Ok/Err, serialize accordingly
    - Allocate output via Vec, forget it (plugin-allocated)
    - Return STATUS_OK or STATUS_PLUGIN_ERROR
  → Host checks status code:
    - STATUS_OK → wire::deserialize(out_slice) → O
    - STATUS_PLUGIN_ERROR → deserialize PluginError
    - STATUS_PANIC → CallError::Panic
  → Free plugin-allocated buffer via free_buffer fn
  → Return Result<O, CallError>
```

---

## Primary Workflows

### 1. Plugin Interface Authoring

**CLI path**: `fidius init-interface <name> --trait <TraitName> [--path <dir>] [--version <ver>]`
- Creates a new crate directory with `Cargo.toml` (depends on `fidius` and `fidius-core`) and `src/lib.rs` (contains `#[plugin_interface]` annotated trait with a stub `process` method).
- Dependency resolution: checks if value is a local path, then tries crates.io API (`ureq` GET to `https://crates.io/api/v1/crates/{name}`), falls back to path dep with warning.

**Manual path**: Author writes a trait with `#[fidius::plugin_interface(version = N, buffer = PluginAllocated)]` and re-exports `fidius::plugin_impl` and `fidius::PluginError`.

### 2. Plugin Implementation Authoring

**CLI path**: `fidius init-plugin <name> --interface <crate> --trait <TraitName> [--path <dir>] [--version <ver>]`
- Creates a cdylib crate with `Cargo.toml` and `src/lib.rs` containing `#[plugin_impl(TraitName)]` and `fidius_core::fidius_plugin_registry!()`.

**Manual path**: Author implements the trait on a struct, annotates with `#[plugin_impl(TraitName)]`, calls `fidius_core::fidius_plugin_registry!()` once in lib.rs.

### 3. Host Loading and Calling

- Construct `PluginHost` via builder: set search paths, optional hash/wire/strategy expectations, optional signature requirements.
- `host.discover()` scans and returns `Vec<PluginInfo>` for all valid plugins.
- `host.load("PluginName")` returns a `LoadedPlugin`, wrap with `PluginHandle::from_loaded()`.
- `handle.call_method::<I, O>(index, &input)` serializes, calls FFI, deserializes.
- `handle.has_capability(bit)` checks optional method support.

### 4. Package Management

- **Validate**: `fidius package validate <dir>` -- loads `package.toml`, displays fixed header fields and metadata field count.
- **Build**: `fidius package build <dir> [--debug]` -- validates manifest, runs `cargo build [--release]` in the package directory.
- **Inspect**: `fidius package inspect <dir>` -- displays all manifest fields including metadata key-values.
- **Sign**: `fidius package sign --key <secret> <dir>` -- signs `package.toml` (not the source tree or binary).
- **Verify**: `fidius package verify --key <public> <dir>` -- verifies `package.toml` signature.

### 5. Signing and Verification

- `fidius keygen --out <name>` -- generates Ed25519 keypair (32-byte raw files: `<name>.secret`, `<name>.public`).
- `fidius sign --key <secret> <dylib>` -- reads dylib, signs with Ed25519, writes detached `.sig` file (e.g., `libfoo.dylib.sig`).
- `fidius verify --key <public> <dylib>` -- reads dylib + `.sig`, verifies against public key.
- Host-side verification: `PluginHost` can require signatures, iterates trusted keys until one matches.

---

## Public Interface Surface

### Proc Macros

**`#[plugin_interface(version = N, buffer = Strategy)]`** on a trait:
- Required attributes: `version` (u32), `buffer` (one of `CallerAllocated`, `PluginAllocated`, `Arena`; only `PluginAllocated` is currently implemented).
- Trait requirements: methods must take `&self` (not `&mut self`), trait must be `Send + Sync`.
- Optional methods: annotate with `#[optional(since = N)]`. Up to 64 optional methods per interface.
- Async methods: supported, use `async fn`.
- Generates companion module `__fidius_{TraitName}` containing:
  - `{TraitName}_VTable` -- `#[repr(C)]` struct with function pointers
  - `{TraitName}_INTERFACE_HASH` -- u64 constant
  - `{TraitName}_INTERFACE_VERSION` -- u32 constant
  - `{TraitName}_BUFFER_STRATEGY` -- u8 constant
  - `{TraitName}_CAP_{METHOD}` -- u64 constants for each optional method
  - `{TraitName}_OPTIONAL_METHODS` -- `&[&str]` constant
  - `new_{traitname}_vtable()` -- const constructor
  - `__fidius_build_{traitname}_descriptor()` -- unsafe const descriptor builder

**`#[plugin_impl(TraitName)]`** on an impl block:
- Generates for each method: `unsafe extern "C" fn __fidius_shim_{Type}_{method}(...)` with `catch_unwind`, serialization, async runtime bridging.
- Generates: `static __FIDIUS_INSTANCE_{Type}` (unit struct instance), `static __FIDIUS_VTABLE_{Type}`, `__fidius_free_buffer_{Type}`, `static __FIDIUS_DESCRIPTOR_{Type}`, `inventory::submit!` registration.
- Handles `Result<T, PluginError>` return types (serializes error to output buffer with `STATUS_PLUGIN_ERROR`).

### Host API (`fidius-host`)

- `PluginHost::builder()` -- builder with methods: `search_path()`, `load_policy()`, `require_signature()`, `trusted_keys()`, `interface_hash()`, `wire_format()`, `buffer_strategy()`, `build()`.
- `PluginHost::discover()` -> `Result<Vec<PluginInfo>, LoadError>`
- `PluginHost::load(name)` -> `Result<LoadedPlugin, LoadError>`
- `PluginHandle::from_loaded(plugin)` -> `PluginHandle`
- `PluginHandle::call_method::<I, O>(index, &input)` -> `Result<O, CallError>`
- `PluginHandle::has_capability(bit)` -> `bool`
- `PluginHandle::info()` -> `&PluginInfo`
- `loader::load_library(path)` -> `Result<LoadedLibrary, LoadError>` (low-level)
- `loader::validate_against_interface(plugin, hash, wire, strategy)` -> `Result<(), LoadError>`
- `package::load_package_manifest::<M>(dir)` -> `Result<PackageManifest<M>, PackageError>`
- `package::discover_packages(dir)` -> `Result<Vec<PathBuf>, PackageError>`
- `package::build_package(dir, release)` -> `Result<PathBuf, PackageError>`
- `signing::verify_signature(dylib_path, trusted_keys)` -> `Result<(), LoadError>`
- `arch::detect_architecture(path)` -> `Result<BinaryInfo, LoadError>`
- `arch::check_architecture(path)` -> `Result<(), LoadError>`

### CLI Commands (`fidius`)

| Command | Arguments |
|---------|-----------|
| `init-interface` | `<name> --trait <T> [--path <dir>] [--version <ver>]` |
| `init-plugin` | `<name> --interface <crate> --trait <T> [--path <dir>] [--version <ver>]` |
| `keygen` | `--out <base>` |
| `sign` | `--key <secret> <dylib>` |
| `verify` | `--key <public> <dylib>` |
| `inspect` | `<dylib>` |
| `package validate` | `<dir>` |
| `package build` | `<dir> [--debug]` |
| `package inspect` | `<dir>` |
| `package sign` | `--key <secret> <dir>` |
| `package verify` | `--key <public> <dir>` |

### Package Manifest Format (`package.toml`)

```toml
[package]
name = "plugin-name"
version = "0.1.0"
interface = "interface-crate-name"
interface_version = 1
source_hash = "sha256:..."  # optional

[dependencies]
other-package = ">=1.0"     # optional section

[metadata]
# Host-defined fields, validated via serde deserialization against host's schema type
```

### Core Types

- `PluginRegistry` -- `#[repr(C)]` struct: magic, registry_version, plugin_count, descriptors pointer.
- `PluginDescriptor` -- `#[repr(C)]` struct: abi_version, interface_name, interface_hash, interface_version, capabilities, wire_format, buffer_strategy, plugin_name, vtable, free_buffer.
- `PluginError` -- Serializable error: code (String), message (String), details (Option<String> as JSON).
- `PluginInfo` -- Owned metadata copied from descriptor (no raw pointers).
- `LoadPolicy` -- `Strict` (default) | `Lenient`.
- `BufferStrategyKind` -- `CallerAllocated(0)` | `PluginAllocated(1)` | `Arena(2)`.
- `WireFormat` -- `Json(0)` | `Bincode(1)`.
- Status codes: `STATUS_OK(0)`, `STATUS_BUFFER_TOO_SMALL(-1)`, `STATUS_SERIALIZATION_ERROR(-2)`, `STATUS_PLUGIN_ERROR(-3)`, `STATUS_PANIC(-4)`.

---

## Dependency Graph (External)

| Crate | Key External Dependencies |
|-------|--------------------------|
| `fidius-core` | `serde` (derive), `serde_json`, `bincode 1`, `thiserror 2`, `inventory 0.3`, `toml 0.8`, `tokio 1` (optional) |
| `fidius-macro` | `syn 2` (full, extra-traits), `quote 1`, `proc-macro2 1`, `fidius-core` (for hash computation at macro expansion time) |
| `fidius-host` | `fidius-core`, `libloading 0.8`, `ed25519-dalek 2` (std, rand_core), `thiserror 2`, `serde`, `serde_json` |
| `fidius-cli` | `fidius-core`, `fidius-host`, `clap 4` (derive), `ed25519-dalek 2`, `ureq 3`, `serde`, `serde_json`, `rand 0.8` |
| `fidius` | `fidius-core`, `fidius-macro` |

**Dev dependencies**: `tempfile 3`, `assert_cmd 2`, `predicates 3`, `trybuild 1`, `libloading 0.8` (for macro tests), `ed25519-dalek 2` (for host e2e tests).

---

## Build and Deployment

### Build Process
- Standard Cargo workspace build: `cargo build --workspace`.
- The `test-plugin-smoke` fixture is excluded from the workspace and built separately (tests trigger `cargo build --manifest-path` on it).
- Wire format is selected at compile time via `cfg(debug_assertions)`: debug builds use JSON, release builds use bincode. This means debug-built plugins cannot be loaded by release-built hosts and vice versa.
- The `async` feature on `fidius-core` / `fidius` enables the `async_runtime` module (lazy tokio multi-thread runtime).

### Test Types
1. **Unit tests** (inline `#[cfg(test)]` modules):
   - `fidius-core/src/hash.rs` -- FNV-1a hash correctness
   - `fidius-core/src/package.rs` -- manifest parsing
   - `fidius-host/src/arch.rs` -- binary format detection
   - `fidius-host/src/signing.rs` -- Ed25519 signing
   - `fidius-macro/src/ir.rs` -- IR parsing

2. **Integration tests** (`*/tests/*.rs`):
   - `fidius-core/tests/layout_and_roundtrip.rs` -- ABI layout assertions (struct sizes, field offsets), wire format roundtrip, error roundtrip, hash regression vectors, magic bytes, version constants
   - `fidius-macro/tests/impl_basic.rs` -- registry + vtable generation, shim invocation
   - `fidius-macro/tests/interface_basic.rs` -- vtable struct, hash, version, buffer strategy, capability constants
   - `fidius-macro/tests/multi_plugin.rs` -- two plugins in one binary, combined registry
   - `fidius-macro/tests/async_plugin.rs` -- async method support
   - `fidius-macro/tests/smoke_cdylib.rs` -- builds cdylib, loads via libloading, calls method
   - `fidius-macro/tests/trybuild.rs` -- compile-fail tests (3 cases: missing version, mut self, unsupported buffer)
   - `fidius-host/tests/integration.rs` -- discover, load, call via PluginHost
   - `fidius-host/tests/e2e.rs` -- signing enforcement, Strict vs Lenient policy
   - `fidius-host/tests/package_e2e.rs` -- manifest schema validation, build + load + call, package discovery
   - `fidius-cli/tests/cli.rs` -- CLI integration via assert_cmd (help, init-interface, init-plugin, keygen-sign-verify, inspect)
   - `fidius-cli/tests/full_pipeline.rs` -- complete scaffold-to-call pipeline (scaffolds interface + plugin, writes package.toml, keygen, sign, validate, build, verify, load, call)

3. **Compile-fail tests** (trybuild):
   - `missing_version.rs` -- `#[plugin_interface]` without `version` attribute
   - `mut_self.rs` -- method with `&mut self`
   - `unsupported_buffer.rs` -- `CallerAllocated` buffer strategy

### Pre-commit Hooks
- `license-header` -- via `angreal license-header --check` (checks all .rs files for Colliery copyright header)
- `rustfmt` -- `cargo fmt --all -- --check`
- `clippy` -- `cargo clippy --workspace -- -D warnings`

### Angreal Tasks
- `angreal build [--release]` -- `cargo build --workspace`
- `angreal check` -- `cargo check --workspace` then `cargo clippy --workspace -- -D warnings`
- `angreal lint` -- `cargo fmt --all --check` then clippy
- `angreal test [--release]` -- `cargo test --workspace` (release mode tests bincode wire)
- `angreal license-header [--check]` -- adds or checks Apache 2.0 license headers on all .rs files

### Documentation Generation
- `plissken.toml` configures the plissken tool to generate API docs in `docs/api/` from all five crates.
- Documentation follows the Diataxis framework: tutorials, how-to guides, reference, explanation.

---

## Conventions and Implicit Knowledge

### Naming Conventions
- Crate names: `fidius-{role}` with hyphens (Rust convention).
- Generated companion module: `__fidius_{TraitName}` (double-underscore prefix, trait name verbatim).
- Generated statics: `__FIDIUS_{KIND}_{ImplType}` (e.g., `__FIDIUS_VTABLE_BasicCalculator`).
- Generated shim functions: `__fidius_shim_{ImplType}_{method}`.
- Plugin name in descriptor: the impl type name as a string (e.g., `"BasicCalculator"`).
- Capability constants: `{TraitName}_CAP_{METHOD_UPPER}`.

### Architectural Patterns
- **Facade pattern**: The `fidius` crate exists solely to provide a single dependency for interface/plugin authors.
- **Builder pattern**: `PluginHost::builder()` for host configuration.
- **Inventory crate pattern**: `inventory::collect!` + `inventory::submit!` for distributed static registration of descriptors across compilation units, enabling multiple plugins per dylib.
- **OnceLock lazy initialization**: The plugin registry is built once on first access.
- **Stateless plugins**: Enforced at compile time -- methods must take `&self`, static instances are unit structs.
- **Wire format compilation boundary**: JSON in debug, bincode in release, determined by `cfg(debug_assertions)`. This is a hard boundary -- mismatched builds are rejected at load time.
- **Detached signature files**: `.sig` suffix appended to the full extension (e.g., `libfoo.dylib.sig`).

### Implicit Conventions
- Plugin structs are expected to be unit structs (no fields). The macro generates `static INSTANCE: Type = Type;` which only compiles for unit structs.
- The `fidius_plugin_registry!()` macro must be called exactly once per cdylib crate.
- Method vtable index is positional (declaration order in the trait). Host and plugin must agree on method ordering.
- Interface crates are expected to re-export `fidius::plugin_impl` and `fidius::PluginError` so plugin authors depend only on the interface crate.
- The CLI's `init-plugin` generates code that hardcodes `fidius-core = { version = "0.1" }` regardless of the actual version.
- Wire format features: the test-plugin-smoke fixture enables the `async` feature on both `fidius` and `fidius-core`, but not all tests require async.

### Error Handling Patterns
- CLI uses `Box<dyn Error>` throughout.
- Host uses `thiserror`-derived enums: `LoadError` (12 variants) and `CallError` (6 variants).
- Core uses `thiserror` for `WireError` and `PackageError`.
- Plugin errors cross the FFI boundary via `PluginError` (serde-serialized). The `details` field stores JSON as a String (not `serde_json::Value`) for bincode compatibility.

### Copyright and Licensing
- All `.rs` files must have the Apache 2.0 header with "Copyright 2026 Colliery, Inc." (enforced by pre-commit hook and angreal task).
- The license header check explicitly skips `target/` and `.metis/` directories.

---

## Open Questions

1. **Repository vs. crate naming mismatch**: The repository directory is named `fides` but all crates use the `fidius` prefix. The GitHub URL in Cargo.toml metadata says `colliery-io/fidius`. The meaning or reason for the `fides` directory name is not documented anywhere in the repository.

2. **Only PluginAllocated is implemented**: `CallerAllocated` and `Arena` buffer strategies are defined in the type system and parseable from attributes, but both are rejected at code generation time with "not yet supported" errors. The specification describes their FFI signatures in detail, suggesting they are planned.

3. **Wire format debug/release boundary**: Debug-built plugins are incompatible with release-built hosts (different serialization format). The system detects and rejects this, but it creates a potential confusion vector for users who mix build profiles. There is no way to override the wire format at compile time.

4. **Hardcoded version in scaffolded plugin**: `init_plugin` generates `fidius-core = { version = "0.1" }` in the plugin's Cargo.toml, but the actual version is `0.0.0-alpha.1`. This will fail to resolve on crates.io.

5. **Interface crate dependency on both `fidius` and `fidius-core`**: The scaffolded interface crate depends on both `fidius` and `fidius-core`. The facade crate already re-exports `fidius-core`, so the direct `fidius-core` dependency may be redundant, though it may be needed for `fidius_plugin_registry!()` which is `#[macro_export]` in `fidius-core`.

6. **Unit struct assumption**: `#[plugin_impl]` generates `static INSTANCE: Type = Type;`. This silently requires the impl type to be a unit struct. Non-unit structs will produce a compile error, but the error message will not explain the fidius constraint.

7. **`.gitignore` includes `Cargo.lock`**: The `.gitignore` lists `Cargo.lock`, yet a `Cargo.lock` file exists in the repository root (39KB). This appears contradictory -- either the lock file is tracked despite the gitignore rule (perhaps added with `git add -f`), or the gitignore rule is a remnant.

8. **Package dependency declarations**: `package.toml` supports a `[dependencies]` section with package name/version constraints, but no dependency resolution logic exists anywhere in the codebase. ADR-1 notes this as "future work."

9. **`source_hash` field**: The `PackageHeader` has an optional `source_hash` field, but no code computes or validates it. It is documented as "SHA-256 hash of the source directory contents" in the ADR but is purely aspirational.

10. **vtable index safety**: Method calls use raw integer indices into the vtable (`call_method(0, ...)`, `call_method(1, ...)`). There is no compile-time or runtime bounds checking on the vtable index. An out-of-bounds index would read garbage memory as a function pointer.

11. **`PluginHandle` vtable casting**: The `call_method` implementation reads function pointers by casting `vtable` to `*const FfiFn` and offsetting by index. This treats the vtable as a flat array of function pointers. For vtables with `Option<fn>` fields (optional methods), this pointer arithmetic assumes `Option<fn>` has the same size and layout as a bare function pointer, which relies on the nullable pointer optimization.

12. **Multi-plugin naming**: When multiple `#[plugin_impl]` exist in one dylib, `inventory` collection order is not guaranteed. The tests verify both plugins are present but don't assert ordering.

13. **Async runtime per dylib**: Each cdylib with the `async` feature gets its own lazily-initialized tokio multi-thread runtime. If a host loads multiple async plugin dylibs, multiple runtimes will coexist.

14. **No logging/tracing**: The entire codebase uses only `println!`/`eprintln!` for output. There is no structured logging, tracing, or debug instrumentation.

15. **Lenient policy warning mechanism**: When `LoadPolicy::Lenient` encounters a signature error, it prints to stderr via `eprintln!("fidius warning: {e}")`. This is not configurable and goes directly to stderr with no structured format.

16. **`detect_architecture` reads entire file**: The architecture detection function (`arch.rs`) reads the entire dylib into memory (`std::fs::read(path)`) just to inspect the first 20 bytes of the header.
