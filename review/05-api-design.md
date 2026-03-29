# API Design Review: Fidius Plugin Framework

**Reviewer lens**: Is the exposed interface right for its consumers?

**Finding prefix**: API-
**Date**: 2026-03-28
**Codebase Version**: 0.0.0-alpha.1

---

## Summary

The fidius framework exposes four major API surfaces: the macro API (`#[plugin_interface]`, `#[plugin_impl]`), the host API (`PluginHost`, `PluginHandle`), the CLI (`fidius`), and the package API. The macro API is the strongest surface -- it provides a natural, Rust-idiomatic annotation model that makes the common case (define trait, implement trait, register) straightforward. The CLI follows standard conventions and produces clean scaffolded projects.

The most significant API design weakness is the host-side calling convention. `PluginHandle::call_method` requires callers to supply raw integer vtable indices and manually specify input/output types via turbofish syntax, creating a large gap between the type-safe trait the plugin author sees and the untyped, index-based dispatch the host author must use. This gap is the framework's primary ergonomic and safety liability.

Secondary concerns include: error types that display raw discriminant values instead of human-readable names, inconsistent behavior between `discover()` and `load()` regarding signature verification, APIs that panic on invalid input rather than returning errors, and a `PluginError` design that stores structured details as a JSON string rather than a typed structure. The package API is clean but contains aspirational fields (`source_hash`, `dependencies`) that are parsed but never acted upon, creating false expectations.

**Finding count**: 3 Critical, 5 Major, 7 Minor, 4 Observations

---

## Interface Inventory

### Macro API (Plugin/Interface Author)

| Surface | Entry Point | Consumer |
|---------|-------------|----------|
| `#[plugin_interface(version, buffer)]` | Attribute on trait | Interface crate author |
| `#[plugin_impl(TraitName)]` | Attribute on impl block | Plugin crate author |
| `fidius_plugin_registry!()` | Macro in lib.rs | Plugin crate author |
| `#[optional(since = N)]` | Attribute on trait method | Interface crate author |

### Host API (Host Application Developer)

| Surface | Entry Point | Consumer |
|---------|-------------|----------|
| `PluginHost::builder()` | Builder pattern | Host application |
| `PluginHost::discover()` | Scan for plugins | Host application |
| `PluginHost::load(name)` | Load specific plugin | Host application |
| `PluginHandle::from_loaded(plugin)` | Wrap for calling | Host application |
| `PluginHandle::call_method::<I,O>(index, &input)` | Invoke method | Host application |
| `PluginHandle::has_capability(bit)` | Check optional method | Host application |
| `PluginHandle::info()` | Read metadata | Host application |
| `loader::load_library(path)` | Low-level loading | Advanced host |
| `package::load_package_manifest::<M>(dir)` | Load package manifest | Host application |
| `package::discover_packages(dir)` | Find packages | Host application |
| `package::build_package(dir, release)` | Build package | Host application |

### CLI (Developer Tooling)

| Command | Consumer |
|---------|----------|
| `fidius init-interface` | Interface author |
| `fidius init-plugin` | Plugin author |
| `fidius keygen` | Release engineer |
| `fidius sign` / `fidius verify` | Release engineer |
| `fidius inspect` | Debugger / operator |
| `fidius package {validate,build,inspect,sign,verify}` | Package manager |

### Core Types (Cross-Cutting)

| Type | Consumer |
|------|----------|
| `PluginError` | Plugin author, host, wire boundary |
| `PluginInfo` | Host application |
| `LoadError` (14 variants) | Host application |
| `CallError` (6 variants) | Host application |
| `PackageError` (4 variants) | Host, CLI |
| `WireError` (2 variants) | Internal (not directly exposed to users) |

---

## Consistency Assessment

### Naming Conventions

**Positive patterns:**
- Crate names consistently use `fidius-{role}` with hyphens.
- Generated symbols follow a consistent `__FIDIUS_{KIND}_{Type}` pattern (`__FIDIUS_VTABLE_`, `__FIDIUS_DESCRIPTOR_`, `__FIDIUS_INSTANCE_`).
- Error types consistently use `thiserror` derive with `#[error(...)]` formatting.
- Builder methods on `PluginHostBuilder` use the conventional `self`-consuming pattern.
- CLI commands use idiomatic kebab-case (`init-interface`, `init-plugin`).

**Inconsistencies found:**
- Error format inconsistency: `LoadError::InterfaceHashMismatch` displays hashes as hex (`{got:#018x}`), `WireFormatMismatch` and `BufferStrategyMismatch` display raw `u8` discriminants, and `IncompatibleRegistryVersion` displays plain `u32`. Three different formatting strategies for the same class of "mismatch" error.
- Naming gap: the `PluginHandle::new()` constructor takes 5 raw parameters while `PluginHandle::from_loaded()` takes a `LoadedPlugin`. Both are public, but `new()` is an internal construction detail that leaks implementation to consumers.
- Module path inconsistency: generated code references `fidius::descriptor::PluginDescriptor` (via facade), but the `fidius_plugin_registry!()` macro references `fidius_core::descriptor::PluginRegistry` (direct). Two different path conventions for generated code.
- Function naming: `load_manifest` in `fidius-core` vs `load_package_manifest` in `fidius-host` for what is essentially a re-export. The host function adds no logic.

### Error Format Consistency

All error enums use `thiserror` derive, which is good. However:
- CLI uses `Box<dyn Error>` -- no structured error type for CLI-specific failures.
- `CallError::Serialization` and `CallError::Deserialization` store error messages as `String`, losing the original error type. An unknown status code is reported via `CallError::Serialization`, which is semantically wrong.
- `PluginError` implements both `Display` and `Error` manually, while all other error types use `thiserror` derive.

---

## Findings

### API-01: `call_method` requires raw vtable indices, creating an unsafe and unergonomic call site (Critical)

**Severity**: Critical
**Location**: `fidius-host/src/handle.rs:93-106`

**Description**: The primary method-calling API requires callers to pass a raw `usize` index and manually specify input/output types via turbofish:

```rust
let result = handle.call_method::<AddInput, AddOutput>(0, &input)?;
```

The caller must: (1) know the correct positional index of each method in the trait declaration order, (2) correctly pair input/output types for that index, (3) ensure index stays synchronized if the trait adds methods. None of these constraints are compiler-enforced. An incorrect index causes undefined behavior; an incorrect type causes deserialization failure at runtime. There is no generated typed proxy that would allow `handle.add(&input)`.

**Impact**: Every host application interacting with plugins must maintain a manual mapping between method names and indices. This is the most common operation in the framework and it provides zero type safety. The `CallError::NotImplemented` variant exists but is never returned -- `call_method` does not check capabilities for optional methods before invoking them.

**Cross-cutting**: Correctness (COR-01, COR-02), Legibility (LEG-02), Evolvability (EVO-01).

**Recommendation**: Generate a typed host-side proxy from `#[plugin_interface]`. For example, the companion module could contain a `GreeterClient` struct that wraps `PluginHandle` and exposes `fn greet(&self, input: GreetInput) -> Result<GreetOutput, CallError>` with the correct index baked in. At minimum, generate index constants (`const METHOD_GREET: usize = 0`) in the companion module. Additionally, `call_method` should check `has_capability` for optional method indices and return `CallError::NotImplemented` rather than calling a null function pointer.

---

### API-02: Error types display raw discriminant values instead of human-readable names (Critical)

**Severity**: Critical
**Location**: `fidius-host/src/error.rs:40-44`, `fidius-host/src/loader.rs:170-184`

**Description**: `LoadError::WireFormatMismatch` and `LoadError::BufferStrategyMismatch` store and display raw `u8` values:

```rust
#[error("wire format mismatch: got {got}, expected {expected}")]
WireFormatMismatch { got: u8, expected: u8 },
```

A user sees: `"wire format mismatch: got 0, expected 1"`. They must look up that `0 = Json` and `1 = Bincode`. This is especially problematic because the most common trigger is a debug/release mismatch, where the user needs actionable guidance, not raw numbers.

The mismatch is introduced at `loader.rs:171` where validated `WireFormat` enum values are cast back to `u8` for the error:

```rust
return Err(LoadError::WireFormatMismatch {
    got: plugin.info.wire_format as u8,
    expected: wire as u8,
});
```

**Impact**: Every wire format or buffer strategy mismatch produces an opaque error message. This is likely the most common error users will encounter during development (debug plugin + release host, or vice versa).

**Cross-cutting**: Legibility (LEG-06, LEG-12).

**Recommendation**: Store enum values in the error variants, not `u8`:

```rust
WireFormatMismatch { got: WireFormat, expected: WireFormat },
BufferStrategyMismatch { got: BufferStrategyKind, expected: BufferStrategyKind },
```

Add `Display` implementations for both enums (they already have `Debug`). Consider adding contextual guidance in the error message: `"wire format mismatch: plugin uses Json (debug build) but host expects Bincode (release build). Ensure both are built with the same profile."`.

---

### API-03: `discover()` silently skips signature verification, contradicting `load()` behavior (Critical)

**Severity**: Critical
**Location**: `fidius-host/src/host.rs:128-167` vs `fidius-host/src/host.rs:173-221`

**Description**: When `require_signature` is `true`, `load()` verifies signatures before accepting a plugin. However, `discover()` skips signature verification entirely -- it only checks interface hash/wire/strategy. A host that calls `discover()` to list available plugins and then presents them to a user creates a false expectation: plugins appear "available" but `load()` will reject them.

```rust
// discover() - no signature check
match loader::load_library(&path) {
    Ok(loaded) => {
        for plugin in &loaded.plugins {
            if let Ok(()) = loader::validate_against_interface(...) {
                plugins.push(plugin.info.clone());  // accepted without sig check
            }
        }
    }
    ...
}
```

**Impact**: Any host workflow of "discover, display, let user select, load" will show plugins that cannot actually be loaded. There is no `PluginInfo` field indicating signature status, so the host cannot distinguish signed from unsigned plugins in discovery results.

**Cross-cutting**: Correctness (COR-05).

**Recommendation**: Either (a) apply the same signature check in `discover()` as in `load()`, (b) add a `signature_status` field to `PluginInfo` populated during discovery, or (c) document the inconsistency prominently in the `discover()` doc comment and explain the design rationale.

---

### API-04: `has_capability` and descriptor accessor methods panic on invalid input (Major)

**Severity**: Major
**Location**: `fidius-host/src/handle.rs:170-173`, `fidius-core/src/descriptor.rs:174-196`

**Description**: Three public methods panic on invalid input rather than returning a graceful error or default:

```rust
// handle.rs:171 -- panics if bit >= 64
pub fn has_capability(&self, bit: u32) -> bool {
    assert!(bit < 64, "capability bit must be < 64");
    ...
}

// descriptor.rs:179 -- panics on unknown buffer strategy
pub fn buffer_strategy_kind(&self) -> BufferStrategyKind {
    match self.buffer_strategy {
        ...
        _ => panic!("invalid buffer_strategy value: {}", self.buffer_strategy),
    }
}

// descriptor.rs:188 -- panics on unknown wire format
pub fn wire_format_kind(&self) -> WireFormat {
    match self.wire_format {
        ...
        _ => panic!("invalid wire_format value: {}", self.wire_format),
    }
}
```

For a host-side API processing potentially untrusted plugin data, panicking is inappropriate. A malicious or corrupted plugin with `wire_format = 99` will crash the host process during `load_library`.

**Impact**: Host application crash from a malformed plugin, rather than graceful rejection. The `has_capability` panic is less critical (programmer error) but still inappropriate for a library API.

**Cross-cutting**: Correctness (COR-13, COR-14).

**Recommendation**: Return `Result` from `buffer_strategy_kind()` and `wire_format_kind()`, propagating to `LoadError`. For `has_capability`, return `false` for `bit >= 64` rather than panicking, since the framework guarantees at most 64 optional methods.

---

### API-05: `PluginError` stores structured details as a JSON string (Major)

**Severity**: Major
**Location**: `fidius-core/src/error.rs:28-35`

**Description**: `PluginError::details` is `Option<String>` containing a JSON-encoded string, with `with_details` accepting `serde_json::Value` and immediately stringifying it:

```rust
pub fn with_details(
    code: impl Into<String>,
    message: impl Into<String>,
    details: serde_json::Value,
) -> Self {
    Self {
        ...
        details: Some(details.to_string()),  // stringify JSON
    }
}
```

The doc comment explains this is for bincode compatibility (bincode cannot serialize `serde_json::Value`). However, this design means: (1) plugin authors must construct `serde_json::Value` manually, adding a `serde_json` dependency, (2) host-side consumers must call `details_value()` to parse the string back, which can silently fail, (3) the round-trip is lossy if the JSON is malformed.

**Impact**: Awkward ergonomics for the most common error case. Plugin authors who want to return structured errors must deal with JSON construction/parsing at both ends. The type system does not prevent `details` from containing non-JSON garbage.

**Cross-cutting**: Evolvability -- changing this field type is a breaking change once the wire format is stable.

**Recommendation**: Consider making `details` a generic serializable type or using a `BTreeMap<String, String>` for common key-value details. If the JSON string approach is retained, document the rationale more prominently and add validation in `with_details` that the round-trip works.

---

### API-06: `PluginHandle::new()` exposes raw FFI internals in a public constructor (Major)

**Severity**: Major
**Location**: `fidius-host/src/handle.rs:58-72`

**Description**: `PluginHandle::new()` is public and accepts raw pointers:

```rust
pub fn new(
    library: Arc<Library>,
    vtable: *const c_void,
    free_buffer: Option<unsafe extern "C" fn(*mut u8, usize)>,
    capabilities: u64,
    info: PluginInfo,
) -> Self
```

This constructor exposes the full unsafe internals of the handle. Users can construct a `PluginHandle` with arbitrary function pointers, a null vtable, or mismatched capabilities. The safe path is `PluginHandle::from_loaded()`, which takes a validated `LoadedPlugin`.

**Impact**: API consumers can bypass all validation by constructing handles directly. The `new()` method should be crate-private (`pub(crate)`) since `from_loaded()` is the intended public API.

**Recommendation**: Make `new()` `pub(crate)` and document `from_loaded()` as the sole public constructor. If `new()` must remain public for advanced use cases, mark it `unsafe` since it has safety preconditions on the pointer arguments.

---

### API-07: `build_package` returns a directory path instead of erroring when cdylib is not found (Major)

**Severity**: Major
**Location**: `fidius-host/src/package.rs:119-121`

**Description**: The `build_package` function's doc comment says "Returns the path to the compiled cdylib on success." However, when it cannot find the cdylib, it silently returns the target directory path instead:

```rust
// Return the target dir even if we can't find the specific dylib
Ok(target_dir)
```

A caller expecting a dylib path receives a directory path. Any subsequent operation (e.g., loading the "dylib") will fail with a confusing error.

**Impact**: Violates the principle of least surprise. The function's return type (`Result<PathBuf, PackageError>`) and documentation promise a cdylib path. A silent fallback to a directory path is indistinguishable from success.

**Cross-cutting**: Correctness (LEG-15).

**Recommendation**: Return `Err(PackageError::BuildFailed("compiled cdylib not found in target directory"))` when the dylib cannot be located. The comment acknowledges the awkwardness, suggesting this is a known gap.

---

### API-08: Unknown FFI status code reported as `CallError::Serialization` (Major)

**Severity**: Major
**Location**: `fidius-host/src/handle.rs:149-153`

**Description**: When `call_method` receives an unknown status code from the FFI boundary, it wraps it in `CallError::Serialization`:

```rust
_ => {
    return Err(CallError::Serialization(format!(
        "unknown status code: {status}"
    )))
}
```

An unknown status code is not a serialization error. It could indicate a version mismatch, a corrupted plugin, or a new status code from a newer version of the framework. Misclassifying it as `Serialization` means host code that catches and handles serialization errors will incorrectly process this case.

**Impact**: Incorrect error categorization. Host applications that pattern-match on `CallError` variants will handle unknown status codes through the wrong branch.

**Recommendation**: Add a dedicated variant: `CallError::UnknownStatus { code: i32 }`. This preserves the original status code and enables proper error handling.

---

### API-09: `fidius_plugin_registry!()` macro lives in `fidius_core` but generated code references `fidius` facade paths (Minor)

**Severity**: Minor
**Location**: `fidius-core/src/registry.rs:68-76` vs `fidius-macro/src/impl_macro.rs:314`

**Description**: The `fidius_plugin_registry!()` macro is `#[macro_export]` in `fidius-core` and references `fidius_core::` paths. But the generated code from `#[plugin_impl]` references `fidius::` paths (the facade). Plugin authors must depend on both crates: `fidius` (for generated code paths) and `fidius-core` (for the registry macro). The scaffolded plugin code confirms this:

```rust
use interface_mod::{plugin_impl, TraitName, PluginError, __fidius_TraitName};
fidius_core::fidius_plugin_registry!();
```

This dual dependency is documented in the scaffold templates but is surprising. The facade crate exists to be a single dependency, yet plugin authors need both.

**Impact**: Plugin authors must add both `fidius` and `fidius-core` to their `Cargo.toml`. The purpose of the facade crate (single dependency) is partially undermined.

**Cross-cutting**: Evolvability (EVO-04).

**Recommendation**: Re-export `fidius_plugin_registry!()` through the `fidius` facade so plugin authors only need one dependency. The macro could reference `fidius::registry::get_registry` instead of `fidius_core::registry::get_registry`, or the facade could provide its own re-export wrapper.

---

### API-10: `PluginHostBuilder` duplicates all fields from `PluginHost` with no validation in `build()` (Minor)

**Severity**: Minor
**Location**: `fidius-host/src/host.rs:28-115`

**Description**: `PluginHost` and `PluginHostBuilder` declare identical field lists (7 fields each, same types). The `build()` method copies fields one-to-one with no validation or transformation, always returning `Ok`:

```rust
pub fn build(self) -> Result<PluginHost, LoadError> {
    Ok(PluginHost {
        search_paths: self.search_paths,
        load_policy: self.load_policy,
        ...
    })
}
```

The builder adds no value over direct construction. The `Result` return type implies construction can fail, but it cannot. A consumer seeing `build() -> Result<_, LoadError>` expects a `LoadError` is possible.

**Impact**: Unnecessary boilerplate and a misleading API signature. Consumers must handle an error case that never occurs.

**Cross-cutting**: Legibility (LEG-07), Evolvability (EVO-12).

**Recommendation**: Either (a) add validation in `build()` (e.g., warn if `require_signature` is true but `trusted_keys` is empty), or (b) remove the `Result` wrapper and return `PluginHost` directly, adding `Result` later when validation is needed. If keeping the `Result` for forward compatibility, add a doc comment explaining the rationale.

---

### API-11: `init_interface` resolves `fidius` and `fidius-core` to the same dependency string (Minor)

**Severity**: Minor
**Location**: `fidius-cli/src/commands.rs:91-102`

**Description**: The `init_interface` command resolves the `fidius` dependency and uses the same resolved string for `fidius-core`:

```rust
let fidius_dep = resolve_dep("fidius", version);
let cargo_toml = format!(
    ...
    fidius = {fidius_dep}
    fidius-core = {fidius_dep}
    ...
);
```

If `fidius` resolves to a local path (e.g., `{ path = "/some/path/fidius" }`), the same path is used for `fidius-core`, which is a different crate at a different directory. Similarly, `init_plugin` hardcodes `fidius-core = { version = "0.1" }` regardless of the actual version.

**Impact**: Scaffolded projects with local path dependencies will have incorrect `fidius-core` paths. Scaffolded plugins will fail to resolve `fidius-core` version 0.1 (actual is 0.0.0-alpha.1).

**Cross-cutting**: Correctness (COR-10, COR-11), Legibility (LEG-10), Evolvability (EVO-05).

**Recommendation**: Resolve `fidius-core` independently using `resolve_dep("fidius-core", version)`. For local path resolution, derive the `fidius-core` path relative to the `fidius` path.

---

### API-12: `verify` CLI command calls `process::exit(1)` instead of returning an error (Minor)

**Severity**: Minor
**Location**: `fidius-cli/src/commands.rs:269-272`

**Description**: The `verify` command calls `std::process::exit(1)` on verification failure, while all other commands return `Err(...)`:

```rust
Err(_) => {
    eprintln!("Signature INVALID: {}", dylib_path.display());
    std::process::exit(1);
}
```

This bypasses `main()`'s error handling, prevents destructors from running, and is inconsistent with every other command. The `package_verify` command delegates to `verify`, inheriting this behavior.

**Impact**: Inconsistent error handling. The `process::exit` prevents cleanup and makes the function unusable as a library (it terminates the process).

**Cross-cutting**: Correctness (COR-18).

**Recommendation**: Return `Err("Signature INVALID: ...".into())` like other commands.

---

### API-13: Signature path construction logic duplicated three times (Minor)

**Severity**: Minor
**Location**: `fidius-host/src/signing.rs:36-42`, `fidius-cli/src/commands.rs:224-230`, `fidius-cli/src/commands.rs:249-255`

**Description**: The logic to derive a `.sig` file path from a dylib path is duplicated in three locations with identical code:

```rust
let sig_path = dylib_path.with_extension(format!(
    "{}.sig",
    dylib_path.extension().and_then(|e| e.to_str()).unwrap_or("")
));
```

**Impact**: A change to the signature file naming convention requires updating three locations. The `with_extension` approach is subtly fragile: for a file without an extension, it produces just `.sig`; for `libfoo.dylib` it correctly produces `libfoo.dylib.sig` -- but this works by coincidence of the format string, not by clear intent.

**Cross-cutting**: Legibility (LEG-11), Evolvability (EVO-11).

**Recommendation**: Extract a `sig_path_for(path: &Path) -> PathBuf` utility in `fidius-core` or `fidius-host` and reuse it.

---

### API-14: `LoadedPlugin` exposes raw pointers in public fields (Minor)

**Severity**: Minor
**Location**: `fidius-host/src/loader.rs:36-45`

**Description**: `LoadedPlugin` has public fields including `vtable: *const c_void` and `free_buffer: Option<unsafe extern "C" fn(...)>`. This struct is returned by `PluginHost::load()` and intended to be passed to `PluginHandle::from_loaded()`. Exposing raw pointers in a public struct returned from a safe API is a design tension -- the struct itself is a safe Rust type, but its fields contain unsafe values.

**Impact**: Users can access and misuse the raw vtable pointer or free_buffer function directly. The intended usage pattern (immediately wrap in `PluginHandle`) is not enforced by the API.

**Recommendation**: Make `LoadedPlugin` fields `pub(crate)` and provide accessor methods, or combine `load()` and `PluginHandle::from_loaded()` into a single `load_handle()` method that returns `PluginHandle` directly.

---

### API-15: Wire format is determined by `cfg(debug_assertions)` with no override mechanism (Minor)

**Severity**: Minor
**Location**: `fidius-core/src/wire.rs:28-33`

**Description**: The wire format is hardcoded to JSON in debug builds and bincode in release builds via `cfg(debug_assertions)`. There is no feature flag, environment variable, or runtime option to override this. A developer cannot use bincode in debug for performance testing, or JSON in release for debugging.

**Impact**: The debug/release coupling is the most likely cause of "plugin won't load" errors during development. A developer building their plugin in debug and their host in release (or using a CI-built release plugin locally) will get a `WireFormatMismatch` with no way to resolve it except rebuilding.

**Cross-cutting**: Evolvability (EVO-02).

**Recommendation**: Provide a feature flag (e.g., `force-json`, `force-bincode`) as an override mechanism. This preserves the default behavior while allowing explicit control. Document the debug/release coupling prominently.

---

### API-16: Facade crate doc examples clearly demonstrate both consumer roles (Observation)

**Severity**: Observation (positive)
**Location**: `fidius/src/lib.rs:15-46`

**Description**: The facade crate's doc comment provides complete, realistic examples for both interface authors and plugin authors. The examples show the full import set, annotation usage, and registry macro call. This is exactly the right level of documentation for a facade crate's entry point.

**Cross-cutting**: Legibility (LEG-18).

**Recommendation**: Consider adding a third example for host-side usage (builder, load, call), since the facade crate does not re-export host types and host authors need to know they depend on `fidius-host` separately.

---

### API-17: Builder pattern methods use consistent, idiomatic Rust conventions (Observation)

**Severity**: Observation (positive)
**Location**: `fidius-host/src/host.rs:49-115`

**Description**: The `PluginHostBuilder` methods follow standard Rust builder conventions: consume `self`, return `Self`, use `impl Into<T>` for path inputs. Method names match the field names. The default `LoadPolicy::Strict` is a sensible, conservative choice. The builder makes the common case simple (just `search_path` + `build()`) while allowing advanced configuration.

---

### API-18: Package manifest API uses generics effectively for schema validation (Observation)

**Severity**: Observation (positive)
**Location**: `fidius-core/src/package.rs:32-41`

**Description**: `PackageManifest<M>` is generic over the host's metadata schema type. This is an elegant design: the host defines a struct for its metadata requirements, and serde validates the manifest against it at load time. The `load_manifest_untyped` convenience function for CLI/tooling use is a good ergonomic addition.

---

### API-19: CLI command structure follows platform conventions (Observation)

**Severity**: Observation (positive)
**Location**: `fidius-cli/src/main.rs:22-132`

**Description**: The CLI uses clap derive with subcommands, nested subcommands for `package`, consistent `--long` flag naming, and positional arguments for primary targets. Command names use kebab-case. Help text is provided via doc comments. The error handling pattern (return `Result`, print in `main`) is clean. The `--debug` flag on `package build` is intuitive (inverts the default release build).

---

## Cross-Cutting Implications

### For Correctness Lens
- API-01 (raw vtable indices) directly enables COR-01 (UB on out-of-bounds) and COR-02 (null fn pointer for unimplemented optional methods). A typed dispatch layer would eliminate both.
- API-04 (panicking accessors) contributes to COR-13 and COR-14. Graceful errors would prevent host crashes from malformed plugins.

### For Evolvability Lens
- API-01 makes any trait method reordering a silent breaking change (EVO-01).
- API-09 (dual crate dependency) complicates the facade crate's purpose and makes crate restructuring harder (EVO-04).
- API-15 (no wire format override) prevents gradual migration between wire formats (EVO-02).

### For Performance Lens
- API-03 (discover loads everything but checks nothing for signatures) means `discover()` does expensive dlopen + dlclose cycles for plugins that will be rejected on `load()`. Signature checking during discovery would avoid wasted work.
- API-06 (public raw constructor) is not a performance issue but relates to the general architecture of `PluginHandle` as a thin unsafe wrapper.

### For Legibility Lens
- API-02 (raw discriminant errors) directly causes LEG-06 and LEG-12.
- API-05 (`PluginError` JSON string) creates a non-obvious pattern that requires documentation to understand.
- API-09 (dual dependency) creates the confusion noted in LEG-04 about which crate provides what.
