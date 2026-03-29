# Evolvability Review: Fidius Plugin Framework

**Lens**: Can this be changed safely and confidently?

**Reviewer**: Evolvability Agent
**Date**: 2026-03-28
**Codebase Version**: 0.0.0-alpha.1

---

## Summary

The fidius framework has a well-chosen crate decomposition that separates concerns along the plugin lifecycle (define, implement, load, call, package). The dependency graph is acyclic and shallow. The core ABI types are compact and centralized. Layout assertion tests protect the most critical invariant -- the FFI struct layout.

However, several areas resist safe evolution. The wire format is hardcoded to `cfg(debug_assertions)` with no override mechanism, creating a compile-time coupling that will impede any migration to a different serialization strategy. The vtable calling convention relies on unsafe raw pointer arithmetic with no bounds checking, making any method-ordering change silently dangerous. The proc macro generates code that couples tightly to exact module paths in the `fidius` facade crate, meaning the generated code's assumptions form a hidden contract that is tested only through end-to-end compilation rather than isolated unit tests. The CLI's scaffolding functions embed hardcoded dependency versions and template strings, making them fragile to version bumps.

The test suite covers the critical path well (layout, roundtrip, macro generation, host loading), but several tests rebuild the test plugin on every invocation, creating slow and brittle tests. There is no mechanism for gradual migration of the ABI version, wire format, or buffer strategy.

Overall severity: **3 Critical**, **5 Major**, **4 Minor**, **3 Observation** findings.

---

## Architecture Assessment

### Modularity

The five-crate decomposition is sound. Each crate has a clear role:

| Crate | Responsibility | Size | Assessment |
|-------|---------------|------|------------|
| `fidius-core` | ABI types, wire format, hashing, package manifest | ~400 LOC | Right-sized, cohesive |
| `fidius-macro` | Proc macros (interface + impl codegen) | ~550 LOC | Right-sized, well-separated IR/codegen |
| `fidius-host` | Loading, validation, calling, package ops | ~500 LOC | Right-sized, good internal module split |
| `fidius-cli` | CLI binary, scaffolding, signing commands | ~400 LOC | Right-sized, though commands.rs is a monolith |
| `fidius` | Facade re-exports | ~50 LOC | Thin, appropriate |

Within crates, modules are generally well-scoped. The macro crate's separation into `ir.rs` (parsing), `interface.rs` (interface codegen), and `impl_macro.rs` (impl codegen) is clean. The host crate's separation of `loader.rs`, `handle.rs`, `host.rs`, `signing.rs`, `arch.rs`, and `package.rs` keeps responsibilities clear.

### Coupling

**Low coupling (good)**:
- Crate dependency graph is acyclic: `fidius-core` is the leaf, everything depends inward.
- `fidius-host` and `fidius-macro` do not depend on each other.
- The facade crate (`fidius`) is purely re-exports.

**High coupling (concerning)**:
- **Generated code to facade paths**: The proc macro generates code referencing `fidius::descriptor::`, `fidius::wire::`, `fidius::status::`, `fidius::inventory`, `fidius::registry::`, and `fidius::async_runtime::`. Any restructuring of the facade crate's re-export layout breaks all generated code.
- **Wire format to build profile**: `cfg(debug_assertions)` selects JSON vs bincode. This couples wire format selection to the Rust build system's debug/release concept. No override is possible.
- **VTable index to method declaration order**: Host code uses `call_method(0, ...)`, `call_method(1, ...)`. The index is determined by trait declaration order. There is no symbolic dispatch.
- **Test plugin builds**: Multiple test files in `fidius-host` and `fidius-cli` rebuild `test-plugin-smoke` via `cargo build --manifest-path`. Changes to the test plugin directory structure, naming, or dependencies can break all host tests simultaneously.

### Cohesion

Generally high. Each module does one thing. Minor concerns:
- `fidius-cli/src/commands.rs` (408 lines) is a flat file containing all command implementations. As commands grow, this will benefit from splitting into per-command modules.
- `fidius-host/src/package.rs` contains both manifest loading (delegating to core) and `build_package` (shelling out to cargo). The build functionality is operationally distinct from manifest parsing.

### Abstraction Boundaries

**Well-placed**:
- `PluginDescriptor` / `PluginRegistry` as the ABI contract is clean and minimal.
- `PluginInfo` as the owned, safe representation of descriptor data is a good abstraction.
- `PluginHandle` wraps raw FFI calling behind a typed API.
- `InterfaceIR` / `MethodIR` decouple parsing from code generation in the macro crate.

**Leaky or misplaced**:
- `PluginHandle::call_method` exposes raw vtable indices to callers. There is no type-safe dispatch layer.
- `detect_architecture` reads the entire file into memory to inspect 20 bytes of header. The abstraction does not limit I/O to what is needed.
- The `PluginHost::builder()` returns `Result<PluginHost, LoadError>` from `.build()`, but `.build()` cannot actually fail -- it always returns `Ok`. This is a leaky future-proofing pattern.

### Dependency Management

**External dependencies are well-managed**: workspace-level version pinning ensures consistency. The choices are mainstream and stable (`serde 1`, `syn 2`, `clap 4`, `libloading 0.8`, `ed25519-dalek 2`).

**Risks**:
- `bincode 1` is used, but `bincode 2` has been available since 2023 with a significantly different API. Migration will touch `fidius-core/src/wire.rs` and all serialized data formats.
- `fidius-macro` depends on `fidius-core` at compile time (for `interface_hash`). This creates a cross-compilation-stage dependency: the proc macro (compiled for the host) links `fidius-core` (which may be compiled for a different target). This works but is unusual and constrains `fidius-core` from using any target-specific code in the hash module.
- No circular dependencies exist.

---

## Change Cost Analysis

### Change 1: Add a new buffer strategy (e.g., CallerAllocated)

**Cost: High**

Files requiring changes:
1. `fidius-macro/src/interface.rs` -- Remove the error return for `CallerAllocated`, generate different FFI signatures in the vtable (different function pointer type).
2. `fidius-macro/src/impl_macro.rs` -- Generate different shim function signatures (buffer sizing, retry logic). The entire `generate_shims` function must branch on strategy.
3. `fidius-host/src/handle.rs` -- `call_method` must branch on buffer strategy for the calling convention. The `FfiFn` type alias is currently hardcoded to the PluginAllocated signature.
4. `fidius-core/src/descriptor.rs` -- No structural changes needed, but `free_buffer` optionality semantics change.
5. `fidius-core/src/status.rs` -- `STATUS_BUFFER_TOO_SMALL` becomes relevant.
6. Tests: All integration tests need CallerAllocated variants.

The vtable function pointer type is baked into the generated `#[repr(C)]` struct. Different strategies need different signatures, meaning the vtable struct itself varies by strategy. This is a deep architectural change.

### Change 2: Add a new CLI command

**Cost: Low**

1. `fidius-cli/src/main.rs` -- Add a variant to `Commands` enum.
2. `fidius-cli/src/commands.rs` -- Add the implementation function.
3. Tests: Add a test in `fidius-cli/tests/cli.rs`.

This is straightforward thanks to clap derive. No other crates are affected.

### Change 3: Add a new field to `PluginDescriptor`

**Cost: High**

1. `fidius-core/src/descriptor.rs` -- Add the field (must be at the end to preserve layout).
2. `fidius-core/tests/layout_and_roundtrip.rs` -- Update size/offset assertions.
3. `fidius-macro/src/interface.rs` -- Generate the new field value in `generate_descriptor_builder`.
4. `fidius-macro/src/impl_macro.rs` -- Pass the new field in `generate_descriptor`.
5. `fidius-host/src/loader.rs` -- Read and validate the new field in `validate_descriptor`.
6. `fidius-host/src/types.rs` -- Add to `PluginInfo`.
7. Increment `ABI_VERSION` -- but there is no migration strategy. Old plugins become incompatible immediately.

The layout assertion tests will catch size/offset drift, which is exactly their purpose. But there is no versioned reading logic -- the host cannot load plugins built with the old descriptor layout.

### Change 4: Replace the serialization library (e.g., bincode 1 to bincode 2, or to msgpack)

**Cost: Medium**

1. `fidius-core/src/wire.rs` -- Replace serialize/deserialize implementations.
2. `fidius-core/Cargo.toml` -- Update dependency.
3. Potentially `fidius-core/src/error.rs` -- If `PluginError` serialization changes.

The `wire.rs` module is a clean abstraction point, so the change is localized. However, the binary format changes, so all existing plugins become incompatible. There is no wire format negotiation.

### Change 5: Support stateful plugins (non-unit structs)

**Cost: Very High**

The entire architecture assumes stateless plugins. `#[plugin_impl]` generates `static INSTANCE: Type = Type;` which requires a unit struct. Supporting state requires:
1. Instance lifecycle management (construction, destruction).
2. `&mut self` support (currently rejected at parse time).
3. Thread safety model (the static instance is `Send + Sync` by construction).
4. Changes to the registry model (factory function instead of static instance).

This would be a near-complete rewrite of the macro and host calling convention.

---

## Findings

### EVO-01: VTable index-based dispatch is fragile and unsafe (Critical)

**Location**: `fidius-host/src/handle.rs:103-106`, generated shims
**Description**: Method calls use raw integer indices (`call_method(0, &input)`, `call_method(1, &input)`) with unchecked pointer arithmetic into the vtable. There is no bounds checking -- an out-of-bounds index reads arbitrary memory as a function pointer. Additionally, the vtable is cast from `*const c_void` to `*const FfiFn` as a flat array, but optional methods are stored as `Option<fn>`, not bare `fn`. This relies on the nullable pointer optimization to maintain layout compatibility.
**Impact**: Any change to method ordering in a trait silently breaks all host code using hardcoded indices. Adding a method in the middle shifts all subsequent indices. An out-of-bounds index causes undefined behavior. The `Option<fn>` layout assumption is not validated.
**Recommendation**: Introduce a named dispatch mechanism (e.g., method name string -> index lookup table in the descriptor) or at minimum add a method count field to the descriptor and bounds-check at call time. Document and test the `Option<fn>` layout assumption explicitly.

### EVO-02: Wire format selection is irrevocably coupled to build profile (Critical)

**Location**: `fidius-core/src/wire.rs:29-33`
**Description**: `cfg(debug_assertions)` determines whether JSON or bincode is used. There is no way to override this. A debug-built plugin cannot be loaded by a release-built host, and vice versa. The detection and rejection logic works, but there is no path to migration or coexistence.
**Impact**: Users cannot debug a release-built plugin, cannot use bincode in debug for performance testing, and cannot gradually migrate from one wire format to another. The coupling will become painful as the ecosystem grows and mixed-profile scenarios become common.
**Recommendation**: Make wire format an explicit configuration option (e.g., a feature flag or runtime parameter) rather than implicitly coupling it to debug assertions. At minimum, allow a `cfg` override.

### EVO-03: No ABI versioning or migration strategy (Critical)

**Location**: `fidius-core/src/descriptor.rs:29`, `fidius-host/src/loader.rs:97-100`
**Description**: `ABI_VERSION` and `REGISTRY_VERSION` are checked for exact equality. There is no concept of backward-compatible evolution, version ranges, or negotiation. Any change to `PluginDescriptor` layout requires incrementing `ABI_VERSION`, which immediately makes all existing plugins incompatible.
**Impact**: The framework cannot evolve its ABI without a flag-day migration. All plugins and hosts must be recompiled simultaneously. This is acceptable at alpha stage but will become a blocker for any production use.
**Recommendation**: Design an ABI evolution strategy now, even if not implemented. Options include: trailing fields with a size field for forward compatibility, version ranges, or a descriptor "extensions" pointer.

### EVO-04: Generated code couples tightly to facade crate module paths (Major)

**Location**: `fidius-macro/src/interface.rs:231`, `fidius-macro/src/impl_macro.rs:142-148,157-158,266,314-316`
**Description**: The proc macro generates code that references `fidius::descriptor::PluginDescriptor`, `fidius::wire::serialize`, `fidius::wire::deserialize`, `fidius::status::STATUS_OK`, `fidius::inventory`, `fidius::registry::DescriptorEntry`, and `fidius::async_runtime::FIDIUS_RUNTIME`. These are hardcoded paths in the code generation. If the facade crate restructures its re-exports, all generated code breaks.
**Impact**: Any refactoring of the `fidius` facade crate (renaming modules, changing re-export structure) requires synchronized changes to the macro crate. The macro cannot be tested in isolation from the exact facade layout. This coupling is invisible to users -- it appears as compilation errors in generated code.
**Recommendation**: Consider using `$crate`-style resolution where possible, or define a stable, documented "generated code API" contract for the facade crate. Alternatively, generate code that references `fidius_core::` directly (since that is the actual source of truth) and only use the facade for user-facing re-exports.

### EVO-05: CLI scaffolding embeds hardcoded version strings and template code (Major)

**Location**: `fidius-cli/src/commands.rs:164` (`fidius-core = { version = "0.1" }`), lines 93-119 (interface template), lines 153-191 (plugin template)
**Description**: The `init_plugin` command generates `fidius-core = { version = "0.1" }` in the plugin's Cargo.toml, but the actual version is `0.0.0-alpha.1`. The interface and plugin templates are embedded as `format!` strings in the command implementation, making them difficult to test, update, or customize.
**Impact**: Scaffolded projects will fail to compile if `fidius-core` version 0.1 is not published on crates.io. Template changes require modifying Rust code and recompiling the CLI. Templates cannot be customized by users.
**Recommendation**: Fix the version string to match the actual crate version (or use the same resolution logic as `resolve_dep`). Consider externalizing templates or at least moving them to a dedicated module.

### EVO-06: Test suite rebuilds test plugin on every test invocation (Major)

**Location**: `fidius-host/tests/integration.rs:24-39`, `fidius-host/tests/e2e.rs`, `fidius-cli/tests/full_pipeline.rs`
**Description**: Multiple test files call `build_test_plugin()` which runs `cargo build --manifest-path` on the test-plugin-smoke crate. Each test invocation triggers a separate cargo build. Tests that run in parallel will contend on the same target directory.
**Impact**: Test suite is slow (multiple redundant builds), fragile (concurrent builds may conflict), and sensitive to changes in the test plugin's directory structure. Any change to `test-plugin-smoke` triggers rebuilds across all test files.
**Recommendation**: Use a build script or test fixture that builds the test plugin once per test run. Consider using `cargo`'s `--target-dir` to isolate concurrent builds, or use a `Once` guard.

### EVO-07: `commands.rs` is a monolithic file with no internal structure (Major)

**Location**: `fidius-cli/src/commands.rs` (408 lines)
**Description**: All CLI command implementations live in a single file with no module structure. Each command is a standalone function, but they share helper functions (`resolve_dep`, `check_crates_io`) and patterns (signing logic used by both `sign` and `package_sign`).
**Impact**: As commands are added, the file will grow linearly. There is no encapsulation -- all helpers are visible to all commands. Testing individual commands requires compiling the entire file.
**Recommendation**: Split into `commands/` directory with per-command or per-concern modules (e.g., `commands/scaffold.rs`, `commands/signing.rs`, `commands/package.rs`).

### EVO-08: `detect_architecture` reads entire file into memory (Major)

**Location**: `fidius-host/src/arch.rs:69`
**Description**: `std::fs::read(path)` reads the entire dylib file (potentially tens of MB) just to inspect the first 16-20 bytes of the binary header.
**Impact**: Wasteful memory allocation during plugin discovery (which scans all dylibs in search paths). For large dylibs, this creates unnecessary memory pressure and latency.
**Recommendation**: Use `std::fs::File::open()` + `read_exact()` to read only the first 20 bytes. This is a simple change with no API impact.

### EVO-09: Package dependency declarations exist but are inert (Minor)

**Location**: `fidius-core/src/package.rs:38-39`, `fidius-host/src/package.rs`
**Description**: `PackageManifest` has a `dependencies: BTreeMap<String, String>` field that is parsed and displayed but never resolved or validated. The ADR acknowledges this as future work.
**Impact**: Users may expect dependency resolution to work when they add `[dependencies]` to `package.toml`. The data structure is in place but the semantics are undefined, which could lead to breaking changes when resolution is eventually implemented.
**Recommendation**: Document the field as reserved/unimplemented. Consider removing it from the schema until resolution logic exists, or add a warning when non-empty dependencies are encountered.

### EVO-10: `source_hash` field is parsed but never computed or validated (Minor)

**Location**: `fidius-core/src/package.rs:55`
**Description**: `PackageHeader.source_hash: Option<String>` is defined in the struct and displayed in CLI output, but no code computes or verifies source hashes.
**Impact**: The field gives a false sense of integrity checking. Users may manually set it to an incorrect value with no warning.
**Recommendation**: Either implement source hash computation and verification, or mark the field clearly as aspirational in documentation and CLI output.

### EVO-11: Signing logic is duplicated between CLI and host (Minor)

**Location**: `fidius-cli/src/commands.rs:215-274` (CLI sign/verify), `fidius-host/src/signing.rs:32-71` (host verify)
**Description**: The signature path construction logic (appending `.sig` to the full extension) is duplicated in three places: CLI sign, CLI verify, and host verify. The signing/verification core logic (read file, sign/verify with Ed25519) is reimplemented rather than shared.
**Impact**: Changes to the signature file naming convention or verification logic must be updated in multiple places. A bug fix in one location may not be applied to the others.
**Recommendation**: Extract a shared signing module in `fidius-core` or `fidius-host` that both the CLI and the host library use.

### EVO-12: `PluginHost::builder().build()` returns `Result` but cannot fail (Minor)

**Location**: `fidius-host/src/host.rs:105-115`
**Description**: `.build()` always returns `Ok(PluginHost { ... })`. The `Result<PluginHost, LoadError>` return type suggests construction can fail, but no validation occurs.
**Impact**: Callers must handle an error case that cannot occur. If validation is added later, existing callers already handle errors (which is good), but the current signature is misleading.
**Recommendation**: This is acceptable as a forward-compatibility measure. Add a comment noting that the `Result` is for future validation. Alternatively, return `PluginHost` directly and change to `Result` when validation is added (breaking change, but this is alpha).

### EVO-13: Proc macro IR layer enables safe refactoring (Observation)

**Location**: `fidius-macro/src/ir.rs`
**Description**: The macro crate separates parsing (IR construction) from code generation. `InterfaceIR` and `MethodIR` provide a stable intermediate representation. The IR has its own unit tests independent of code generation.
**Impact**: Positive. Code generation can be refactored or extended (e.g., for new buffer strategies) without changing the parsing layer. New code generators can consume the same IR.

### EVO-14: Layout assertion tests provide strong ABI stability guardrails (Observation)

**Location**: `fidius-core/tests/layout_and_roundtrip.rs:30-73`
**Description**: Tests assert exact struct sizes, alignments, and field offsets for `PluginRegistry` and `PluginDescriptor`. These tests will fail immediately if any field is added, removed, reordered, or if types change.
**Impact**: Positive. This is the right pattern for FFI stability. Any accidental ABI change is caught at test time before it can cause runtime crashes.

### EVO-15: `PluginInfo` as an owned copy of descriptor data enables safe evolution (Observation)

**Location**: `fidius-host/src/types.rs:23-38`, `fidius-host/src/loader.rs:131-148`
**Description**: `PluginInfo` copies all descriptor data to owned types at load time, decoupling the rest of the host from raw FFI pointers. The conversion happens in one place (`validate_descriptor`).
**Impact**: Positive. The descriptor's `#[repr(C)]` layout can evolve independently of the host's internal representation. New fields can be added to `PluginInfo` without changing the FFI contract, and vice versa (with appropriate defaulting).
