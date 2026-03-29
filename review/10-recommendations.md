# Recommendations: Fidius Plugin Framework

**Date**: 2026-03-28
**Codebase Version**: 0.0.0-alpha.1

---

## Overview

This document provides prioritized, actionable recommendations derived from the architecture review findings in `09-report.md`. Recommendations are grouped by urgency:

- **Immediate Actions**: Must address before further development. These fix Critical findings that represent undefined behavior, security vulnerabilities, or data corruption on every method call.
- **Short-Term Actions**: Address in the next development cycle. These fix Major findings that affect usability, reliability, or correctness in common scenarios.
- **Structural Improvements**: Larger efforts to schedule across multiple cycles. These address architectural concerns that affect long-term evolvability.
- **Architectural Recommendations**: Systemic improvements beyond individual findings.

**How to read each recommendation**: Each entry identifies the findings it addresses, the severity of those findings, an effort estimate, what to do, why it matters, a suggested approach, and any dependencies on other recommendations.

**Prioritization rationale**: Immediate actions are ordered by impact (how many users/calls are affected) and effort (quick wins first). Short-term actions are ordered by user-facing impact. Structural improvements are ordered by the number of findings they address.

---

## Immediate Actions

*Must address before further development. All Critical findings.*

---

### R-01: Fix `free_buffer` Capacity Mismatch

**Addresses**: COR-03, SEC-08, PRF-006
**Severity of addressed findings**: Critical, Critical, Observation
**Effort estimate**: Hours (< 1 hour)

**What to do**: Add `output_bytes.shrink_to_fit()` before `std::mem::forget(output_bytes)` in the generated FFI shim code.

**Why it matters**: Every plugin method call currently triggers undefined behavior. The generated `free_buffer` reconstructs a `Vec` with `capacity == len`, but the original `Vec` from serialization may have excess capacity. This causes incorrect deallocation size, which is UB and can cause heap corruption under strict allocators (jemalloc, ASan). This is the highest-priority fix because it affects every single method call.

**Suggested approach**:
1. In `fidius-macro/src/impl_macro.rs`, in the `generate_shims` function, add `output_bytes.shrink_to_fit();` to the generated code immediately before the line that reads `let len = output_bytes.len();`.
2. Apply the same fix to both the normal return path and the `returns_result` Ok path.
3. Add a test that verifies `free_buffer` is called correctly (e.g., by checking that the method completes without sanitizer errors).

**Dependencies**: None. This is a self-contained one-line fix in the code generator.

---

### R-02: Add `method_count` to PluginDescriptor and Bounds-Check vtable Access

**Addresses**: COR-01, COR-02, SEC-04, LEG-02, LEG-03, EVO-01, PRF-008, API-01, OPS-10
**Severity of addressed findings**: Critical, Critical, Critical, Major, Major, Critical, Observation, Critical, Major
**Effort estimate**: Days (2-3 days)

**What to do**: Add a `method_count: u32` field to `PluginDescriptor`, generate it from the macro, validate it at load time, and bounds-check in `call_method`. Also check for null function pointers (unimplemented optional methods) before calling.

**Why it matters**: This is the single most cross-cutting issue, affecting 9+ findings across all 7 review lenses. An out-of-bounds vtable index currently causes undefined behavior. Calling an unimplemented optional method dereferences a null function pointer. These are the framework's most dangerous bugs.

**Suggested approach**:
1. Add `method_count: u32` to `PluginDescriptor` in `fidius-core/src/descriptor.rs` (must be appended at the end to minimize layout disruption).
2. Update `fidius-core/tests/layout_and_roundtrip.rs` with new size/offset assertions.
3. Increment `ABI_VERSION` to 2 (acceptable at alpha with no deployed plugins).
4. In `fidius-macro/src/interface.rs`, generate `method_count` in the descriptor builder by counting all methods (required + optional).
5. In `fidius-macro/src/impl_macro.rs`, pass the method count when constructing the descriptor.
6. In `fidius-host/src/handle.rs`, add `method_count` to `PluginHandle` (copied from descriptor at construction time).
7. In `call_method`, before the unsafe pointer read: `if index >= self.method_count as usize { return Err(CallError::MethodNotFound { index }); }`.
8. After reading the function pointer, check for null: `if fn_ptr.is_null() { return Err(CallError::NotImplemented { bit: index as u32 }); }`. This activates the existing dead `NotImplemented` variant.
9. Add a static assertion in generated code: `const _: () = assert!(std::mem::size_of::<Option<FfiFn>>() == std::mem::size_of::<FfiFn>());` to validate the nullable pointer optimization assumption.
10. Add tests: out-of-bounds index returns error, unimplemented optional method returns `NotImplemented`.

**Dependencies**: This is a breaking ABI change. Coordinate with any test fixtures. R-01 should be done first (simpler, independent).

---

### R-03: Add Null-Pointer Check on Output Buffer in `call_method`

**Addresses**: SEC-06, COR-06
**Severity of addressed findings**: Critical, Major
**Effort estimate**: Hours (< 1 hour)

**What to do**: Check `out_ptr` for null before creating a slice on the `STATUS_OK` path.

**Why it matters**: A malicious or buggy plugin that returns `STATUS_OK` without setting `out_ptr` causes the host to create a slice from a null pointer, which is undefined behavior.

**Suggested approach**:
1. In `fidius-host/src/handle.rs`, before `std::slice::from_raw_parts(out_ptr, ...)`, add:
   ```rust
   if out_ptr.is_null() {
       return Err(CallError::Serialization(
           "plugin returned null output buffer".into()
       ));
   }
   ```
2. Add a comment explaining the defensive check.

**Dependencies**: None.

---

### R-04: Move Signature Verification Before `dlopen` in `discover()` and `load()`

**Addresses**: COR-05, SEC-02, API-03, OPS-02 (partial)
**Severity of addressed findings**: Critical (adjusted), Major, Critical, Critical
**Effort estimate**: Days (1-2 days)

**What to do**: When `require_signature` is true, verify the detached `.sig` file before calling `load_library()` (which executes `dlopen`). Apply this to both `discover()` and `load()`.

**Why it matters**: `dlopen` executes constructor code in the dylib before any validation. Currently, `discover()` opens every dylib it finds with no signature check, enabling code execution from untrusted files placed in search paths. This is a code execution vulnerability.

**Suggested approach**:
1. Extract signature verification logic from `load()` into a standalone function that operates on file paths (it already does -- `signing::verify_signature`).
2. In both `discover()` and `load()`, call `verify_signature` before `load_library()` when `require_signature` is true.
3. For `discover()`, collect and optionally return per-file rejection reasons (see R-09 for full diagnostics).
4. Document that architecture checking (header byte inspection) is the only pre-signature gate, and that it does not execute code.

**Dependencies**: None, though R-09 (observability) would complement this by making rejection reasons visible.

---

### R-05: Replace Panics with Result Returns for Descriptor Field Parsing

**Addresses**: COR-13, COR-14, SEC-07, API-04
**Severity of addressed findings**: Major (adjusted), Major (adjusted), Major, Major
**Effort estimate**: Hours (2-3 hours)

**What to do**: Change `buffer_strategy_kind()`, `wire_format_kind()`, and `has_capability()` from panicking to returning `Result` or safe defaults.

**Why it matters**: These methods are called on data from loaded plugins. A single malformed dylib with an unknown `wire_format` or `buffer_strategy` byte crashes the host process. Combined with `discover()` scanning all dylibs, one corrupted file in a search path can DoS any host.

**Suggested approach**:
1. In `fidius-core/src/descriptor.rs`:
   - Change `buffer_strategy_kind()` to return `Result<BufferStrategyKind, u8>` (returning the unknown value on error).
   - Change `wire_format_kind()` to return `Result<WireFormat, u8>` (same).
   - Change `has_capability(bit)` to return `false` for `bit >= 64` instead of panicking.
2. In `fidius-host/src/loader.rs`, propagate the `Err` as a new `LoadError` variant (e.g., `LoadError::UnknownWireFormat { value: u8 }` and `LoadError::UnknownBufferStrategy { value: u8 }`).
3. Update callers in `fidius-host/src/loader.rs` to use `?` propagation.

**Dependencies**: None.

---

### R-06: Fix `detect_architecture` to Read Only Header Bytes

**Addresses**: PRF-001, COR-08, COR-12, LEG-08, EVO-08
**Severity of addressed findings**: Critical (adjusted), Major, Minor, Major (adjusted), Major
**Effort estimate**: Hours (1-2 hours)

**What to do**: Replace `std::fs::read(path)` with `File::open` + `read_exact` for a small fixed buffer (20 bytes). Preserve the original IO error instead of mapping everything to `LibraryNotFound`.

**Why it matters**: Discovery scans every dylib in all search paths. Each call to `detect_architecture` reads the entire file into memory just to inspect 20 header bytes. For a directory with many large dylibs, this causes massive unnecessary memory allocation. Additionally, IO errors (e.g., permission denied) are misclassified as "library not found," sending users on a wild goose chase.

**Suggested approach**:
1. In `fidius-host/src/arch.rs`, replace:
   ```rust
   let bytes = std::fs::read(path).map_err(|_| ...)?;
   ```
   with:
   ```rust
   let mut buf = [0u8; 20];
   let mut file = std::fs::File::open(path).map_err(|e| {
       if e.kind() == std::io::ErrorKind::NotFound {
           LoadError::LibraryNotFound { path: path.display().to_string() }
       } else {
           LoadError::Io { path: path.display().to_string(), source: e.to_string() }
       }
   })?;
   let n = file.read(&mut buf).map_err(|e| ...)?;
   ```
2. Add a `LoadError::Io` variant if one does not exist, or reuse an appropriate variant.
3. Update the rest of the function to use `&buf[..n]` instead of `&bytes`.

**Dependencies**: None.

---

## Short-Term Actions

*Address in the next development cycle. Major findings.*

---

### R-07: Improve Error Messages for Wire Format and Buffer Strategy Mismatches

**Addresses**: API-02, LEG-06, LEG-12, OPS-03
**Severity of addressed findings**: Critical, Major (adjusted), Minor, Major
**Effort estimate**: Hours (2-3 hours)

**What to do**: Store enum values in error variants instead of raw `u8` discriminants. Add `Display` implementations for `WireFormat` and `BufferStrategyKind`. Include build profile hints in the error message.

**Why it matters**: Wire format mismatch from mixing debug and release builds is the most likely error users will encounter during development. The current message (`"got 0, expected 1"`) provides no actionable guidance.

**Suggested approach**:
1. In `fidius-host/src/error.rs`, change:
   ```rust
   WireFormatMismatch { got: WireFormat, expected: WireFormat },
   BufferStrategyMismatch { got: BufferStrategyKind, expected: BufferStrategyKind },
   ```
2. Add `Display` impls for both enums in `fidius-core` (e.g., `WireFormat::Json => "Json (debug build)"`, `WireFormat::Bincode => "Bincode (release build)"`).
3. Update `fidius-host/src/loader.rs` to pass enum values instead of `as u8`.
4. Consider adding a hint: `"Ensure both plugin and host are compiled with the same build profile."`.

**Dependencies**: R-05 (descriptor accessors return Result) should be done first, as the accessor methods produce the values used here.

---

### R-08: Fix CLI Scaffolding to Produce Correct Dependencies

**Addresses**: COR-10, COR-11, LEG-10, EVO-05, API-11
**Severity of addressed findings**: Minor, Minor, Minor, Major, Minor
**Effort estimate**: Hours (2-3 hours)

**What to do**: Fix `init_interface` to resolve `fidius-core` independently. Fix `init_plugin` to use proper version resolution instead of hardcoded `"0.1"`.

**Why it matters**: The scaffolding commands are the primary onboarding path. Producing broken projects undermines first impressions and wastes newcomer time.

**Suggested approach**:
1. In `fidius-cli/src/commands.rs`, `init_interface`:
   - Call `resolve_dep("fidius-core", version)` separately from `resolve_dep("fidius", version)`.
   - For local path resolution, derive `fidius-core` path relative to the workspace root.
2. In `init_plugin`:
   - Replace hardcoded `fidius-core = { version = "0.1" }` with `resolve_dep("fidius-core", version)`.
3. Fix the User-Agent URL to reference `colliery-io/fidius` (LEG-14).
4. Consider using `env!("CARGO_PKG_VERSION")` as a fallback version.

**Dependencies**: None.

---

### R-09: Add Observability Infrastructure (tracing)

**Addresses**: OPS-01, OPS-02, OPS-04, OPS-05, OPS-07, OPS-08, OPS-09, OPS-12, OPS-13
**Severity of addressed findings**: Critical, Critical, Major, Major, Minor, Minor, Minor, Observation, Observation
**Effort estimate**: Days (3-5 days)

**What to do**: Add the `tracing` crate as an optional dependency (behind a feature flag) to `fidius-host`. Instrument key operations with spans and events. Add a `--verbose` flag to the CLI.

**Why it matters**: The framework has zero structured logging. Users cannot debug plugin loading failures without reading source code. `discover()` swallows all errors silently. This is the single change that addresses the most findings (~13).

**Suggested approach**:
1. Add `tracing = { version = "0.1", optional = true }` to `fidius-host/Cargo.toml`. Feature-gate all tracing calls behind `#[cfg(feature = "tracing")]` so there is zero cost when disabled.
2. Instrument `PluginHost::load()`:
   - `info!` span with plugin name
   - `debug!` event for each candidate dylib tried, with rejection reason
   - `warn!` for candidates that match the name but fail validation
3. Instrument `PluginHost::discover()`:
   - `info!` span with search paths
   - `debug!` event for each dylib found, with accept/reject status and reason
   - Change the return type or add `discover_with_diagnostics()` that returns `(Vec<PluginInfo>, Vec<(PathBuf, LoadError)>)`.
4. Instrument `load_library()`: `debug!` events for each validation step (arch check, magic, version, hash).
5. Instrument `verify_signature()`: `debug!` event with result.
6. Add file path context to `InvalidMagic` and other errors missing location info.
7. In the CLI, add a global `--verbose` flag that initializes a `tracing_subscriber` with `WARN` (default) or `DEBUG` (verbose) filter.

**Dependencies**: R-04 (signature before dlopen) should be done first so tracing captures the correct flow.

---

### R-10: Make `PluginHandle::new()` Crate-Private and Hide Raw Pointers

**Addresses**: API-06, API-14
**Severity of addressed findings**: Major, Minor
**Effort estimate**: Hours (1-2 hours)

**What to do**: Change `PluginHandle::new()` to `pub(crate)`. Make `LoadedPlugin` fields `pub(crate)` and provide accessor methods or combine `load()` + `from_loaded()` into a single API.

**Why it matters**: The public constructor exposes raw FFI internals, allowing users to bypass all validation and construct handles with arbitrary function pointers. The safe path (`from_loaded()`) should be the only public construction method.

**Suggested approach**:
1. Change `PluginHandle::new()` visibility to `pub(crate)`.
2. Change `LoadedPlugin` fields to `pub(crate)`.
3. Consider adding `PluginHost::load_handle(name) -> Result<PluginHandle, LoadError>` that combines `load()` + `from_loaded()` for the common case.
4. If `new()` must remain public for advanced use cases, mark it `unsafe` with documented safety preconditions.

**Dependencies**: None. May require updating downstream code if `LoadedPlugin` fields are accessed directly.

---

### R-11: Fix `build_package` to Return Error When cdylib Not Found

**Addresses**: API-07, LEG-15
**Severity of addressed findings**: Major, Observation
**Effort estimate**: Hours (< 1 hour)

**What to do**: Return `Err(PackageError::CdylibNotFound { ... })` instead of silently returning the target directory path.

**Why it matters**: The function's documented contract promises a cdylib path. Returning a directory path is indistinguishable from success and causes confusing downstream errors.

**Suggested approach**:
1. In `fidius-host/src/package.rs`, replace `Ok(target_dir)` with `Err(PackageError::CdylibNotFound { dir: target_dir })`.
2. Add the `CdylibNotFound` variant to `PackageError`.

**Dependencies**: None.

---

### R-12: Add `CallError::UnknownStatus` Variant

**Addresses**: API-08
**Severity of addressed findings**: Major
**Effort estimate**: Hours (< 1 hour)

**What to do**: Replace the `CallError::Serialization(format!("unknown status code: {status}"))` with a dedicated `CallError::UnknownStatus { code: i32 }` variant.

**Why it matters**: Unknown status codes are not serialization errors. Misclassifying them causes host code that pattern-matches on `CallError` to handle them through the wrong branch.

**Suggested approach**:
1. Add `UnknownStatus { code: i32 }` to `CallError` in `fidius-host/src/error.rs`.
2. Update the match in `call_method` to use the new variant.

**Dependencies**: None.

---

### R-13: Fix `verify` Command to Return Error Instead of `process::exit`

**Addresses**: COR-18, API-12, OPS-06
**Severity of addressed findings**: Observation, Minor, Minor
**Effort estimate**: Hours (< 30 minutes)

**What to do**: Replace `std::process::exit(1)` with `Err(...)` return in the `verify` command.

**Why it matters**: Inconsistent error handling. Bypasses destructors and prevents composability. The `package verify` command inherits this behavior.

**Suggested approach**:
1. In `fidius-cli/src/commands.rs`, change the `Err(_)` branch of verify to:
   ```rust
   Err(_) => Err(format!("Signature INVALID: {}", dylib_path.display()).into())
   ```

**Dependencies**: None.

---

### R-14: Preserve Panic Messages Across FFI Boundary

**Addresses**: OPS-09, OPS-13
**Severity of addressed findings**: Minor, Observation
**Effort estimate**: Hours (3-4 hours)

**What to do**: In generated shims, extract the panic payload from `catch_unwind` and pass it through the FFI boundary. On the host side, include the message in `CallError::Panic`.

**Why it matters**: When a plugin panics, the host receives "plugin panicked during method call" with no diagnostic information. The actual panic message is discarded.

**Suggested approach**:
1. In the generated shim (impl_macro.rs), after `catch_unwind`, extract the message:
   ```rust
   Err(panic_payload) => {
       let msg = panic_payload.downcast_ref::<&str>().map(|s| s.to_string())
           .or_else(|| panic_payload.downcast_ref::<String>().cloned())
           .unwrap_or_else(|| "unknown panic".to_string());
       // Serialize msg into the output buffer with STATUS_PANIC
   }
   ```
2. On the host side, when `STATUS_PANIC` is received, attempt to deserialize a panic message string from the output buffer.
3. Change `CallError::Panic` to `CallError::Panic(String)`.

**Dependencies**: R-03 (null-pointer check on output buffer) should be done first.

---

### R-15: Restrict Secret Key File Permissions

**Addresses**: SEC-01
**Severity of addressed findings**: Major
**Effort estimate**: Hours (< 1 hour)

**What to do**: Set file permissions to `0600` on Unix after writing the secret key file.

**Why it matters**: The secret key is written with default permissions (typically `0644`), making it readable by any user on the system. This is a direct compromise of the signing model on shared machines.

**Suggested approach**:
1. In `fidius-cli/src/commands.rs`, after `std::fs::write(&secret_path, ...)`:
   ```rust
   #[cfg(unix)]
   {
       use std::os::unix::fs::PermissionsExt;
       std::fs::set_permissions(&secret_path,
           std::fs::Permissions::from_mode(0o600))?;
   }
   ```
2. Consider emitting a warning on non-Unix platforms about manual permission setting.

**Dependencies**: None.

---

### R-16: Fix `LoadPolicy::Lenient` Signature Semantics

**Addresses**: SEC-03
**Severity of addressed findings**: Major
**Effort estimate**: Hours (2-3 hours)

**What to do**: When `require_signature` is true, always enforce signature verification regardless of load policy. `Lenient` should only affect non-security validation (hash mismatch, version mismatch).

**Why it matters**: The current behavior -- requiring signatures but ignoring failures under `Lenient` -- creates a false sense of security. A host configured with `require_signature = true` and `Lenient` policy appears to enforce signatures but does not.

**Suggested approach**:
1. In `fidius-host/src/host.rs`, remove the `Lenient` fallback for signature verification errors. If `require_signature` is true and verification fails, always return `Err`.
2. `Lenient` should only affect the behavior for `WireFormatMismatch`, `BufferStrategyMismatch`, and `InterfaceHashMismatch`.
3. Update documentation to clarify what `Lenient` controls.
4. Add tests: Lenient + require_signature + invalid sig = error (not warning).

**Dependencies**: R-04 (signature before dlopen) should be done first for a coherent signing story.

---

### R-17: Consolidate Signing Utility Functions

**Addresses**: LEG-11, EVO-11, API-13, LEG-17
**Severity of addressed findings**: Minor, Minor, Minor, Observation
**Effort estimate**: Hours (2-3 hours)

**What to do**: Extract shared utilities (sig path construction, build invocation) into `fidius-host` and have the CLI delegate to them.

**Why it matters**: Three identical sig-path construction blocks and two identical build invocations create maintenance burden and risk of divergence.

**Suggested approach**:
1. Add `pub fn sig_path_for(path: &Path) -> PathBuf` to `fidius-host/src/signing.rs`.
2. Have CLI's sign, verify, and package_sign/verify commands use this function.
3. Have CLI's `package_build` delegate to `fidius_host::package::build_package` instead of reimplementing.

**Dependencies**: None.

---

### R-18: Add Test Coverage for Error Paths

**Addresses**: COR-16, COR-15, COR-19
**Severity of addressed findings**: Observation, Observation, Observation
**Effort estimate**: Days (2-3 days)

**What to do**: Add tests for untested error paths: Result-returning plugin methods, plugin panics, out-of-bounds vtable index (after R-02), unimplemented optional methods, and hash regression vectors with hardcoded values.

**Why it matters**: The test suite has zero tests for any error path through `call_method`. The error propagation code for `STATUS_PLUGIN_ERROR` and `STATUS_PANIC` is completely untested.

**Suggested approach**:
1. Add a test fixture method that returns `Result<T, PluginError>`, test both Ok and Err paths.
2. Add a test fixture method that panics, verify `CallError::Panic` is returned.
3. After R-02 is implemented, add a test for out-of-bounds index returning an error.
4. Add a test for calling an unimplemented optional method.
5. Hardcode exact hash values in `hash_known_vectors` test.
6. Use per-test temporary directories for signing tests to eliminate flakiness (COR-15).

**Dependencies**: R-02 (method_count) for bounds-checking tests. R-14 (panic messages) for panic message verification.

---

## Structural Improvements

*Larger efforts to schedule across multiple cycles.*

---

### R-19: Design ABI Evolution Strategy

**Addresses**: EVO-03
**Severity of addressed findings**: Critical
**Effort estimate**: Weeks (1-2 weeks for design + implementation)

**What to do**: Design and implement a forward-compatible ABI evolution mechanism before moving beyond alpha.

**Why it matters**: Currently, any change to `PluginDescriptor` requires incrementing `ABI_VERSION`, making all existing plugins incompatible. There is no mechanism for gradual migration. This becomes a blocker for any production use.

**Suggested approach**:
1. **Design phase** (investigate): Evaluate options:
   - **Size-based extensibility**: Add `descriptor_size: u32` as the first field. Hosts read only up to the size they know, defaulting unknown trailing fields.
   - **Version ranges**: Allow hosts to accept a range of ABI versions with per-version reading logic.
   - **Extensions pointer**: Add an opaque `*const c_void` extensions pointer for future fields, read via typed accessor functions.
2. **Implementation**: Implement the chosen approach, update layout tests, update the macro to generate the size field.
3. **Document** the ABI stability contract: which changes are backward-compatible and which require a version bump.

**Dependencies**: R-02 (adds `method_count`, which is the first new field). The ABI evolution strategy should be designed before adding more fields.

---

### R-20: Add Wire Format Override Mechanism

**Addresses**: EVO-02, API-15
**Severity of addressed findings**: Critical, Minor
**Effort estimate**: Days (3-5 days)

**What to do**: Add feature flags (`force-json`, `force-bincode`) that override the `cfg(debug_assertions)` default. Optionally add a runtime override.

**Why it matters**: The current coupling prevents debugging release-built plugins, performance-testing with bincode in debug, and gradual migration between wire formats. The coupling will become increasingly painful as the ecosystem grows.

**Suggested approach**:
1. Add Cargo features to `fidius-core`: `force-json` and `force-bincode`.
2. Change `wire.rs` to check features first, then fall back to `cfg(debug_assertions)`.
3. Document the override mechanism and the default behavior.
4. Consider a compile-time warning when both features are enabled simultaneously.

**Dependencies**: R-07 (error messages) should be done first so any mismatch errors are human-readable.

---

### R-21: Generate Typed Host-Side Proxy from `#[plugin_interface]`

**Addresses**: API-01, LEG-02, EVO-01 (partial)
**Severity of addressed findings**: Critical, Major, Critical
**Effort estimate**: Weeks (1-2 weeks)

**What to do**: Generate a typed wrapper struct in the companion module that wraps `PluginHandle` and exposes named methods with correct types, eliminating raw index usage.

**Why it matters**: The current `call_method(0, &input)` API provides zero type safety -- wrong index, wrong types, or reordered methods all compile successfully but fail at runtime or cause UB. A typed proxy would make incorrect usage a compile-time error.

**Suggested approach**:
1. In the companion module generated by `#[plugin_interface]`, generate:
   ```rust
   pub struct TraitNameClient { handle: PluginHandle }
   impl TraitNameClient {
       pub fn from_handle(handle: PluginHandle) -> Self { ... }
       pub fn method_name(&self, input: InputType) -> Result<OutputType, CallError> {
           self.handle.call_method::<InputType, OutputType>(0, &input)
       }
   }
   ```
2. Generate index constants as an intermediate step: `pub const METHOD_GREET: usize = 0;`.
3. For optional methods, generate methods that check `has_capability` first and return `CallError::NotImplemented`.
4. The raw `call_method` API can remain for advanced/dynamic use cases.

**Dependencies**: R-02 (method_count and bounds checking) should be done first as the foundation.

---

### R-22: Decouple Generated Code from Facade Crate Paths

**Addresses**: EVO-04, API-09
**Severity of addressed findings**: Major, Minor
**Effort estimate**: Days (3-5 days)

**What to do**: Change generated code to reference `fidius_core::` directly instead of `fidius::`, or establish a stable "generated code API" contract for the facade. Re-export `fidius_plugin_registry!()` through the facade crate.

**Why it matters**: The proc macro generates code referencing `fidius::descriptor::`, `fidius::wire::`, `fidius::status::`, etc. Any restructuring of the facade crate breaks all generated code. Plugin authors must depend on both `fidius` and `fidius-core`, undermining the facade's purpose.

**Suggested approach**:
1. **Option A (recommended)**: Change generated code to use `fidius_core::` paths. The `fidius` facade remains for user-facing re-exports only. Plugin authors depend on the interface crate (which re-exports what they need) and `fidius-core` (for the registry macro).
2. **Option B**: Re-export `fidius_plugin_registry!()` through the `fidius` facade and change generated code to use `fidius::` consistently. This means plugin authors need only the interface crate and `fidius`.
3. Document the chosen convention and add a CI check that the generated code paths resolve correctly.

**Dependencies**: None, but coordinate with R-08 (scaffolding fixes) to ensure templates match the new path convention.

---

### R-23: Split CLI `commands.rs` into Per-Concern Modules

**Addresses**: EVO-07
**Severity of addressed findings**: Major
**Effort estimate**: Hours (3-4 hours)

**What to do**: Split `commands.rs` (408 lines) into a `commands/` module directory with separate files for related command groups.

**Why it matters**: All command implementations, helpers, and templates live in a single flat file. As commands grow, this becomes increasingly difficult to navigate and test.

**Suggested approach**:
1. Create `fidius-cli/src/commands/mod.rs`.
2. Move related functions into:
   - `commands/scaffold.rs` -- `init_interface`, `init_plugin`, `resolve_dep`, `check_crates_io`
   - `commands/signing.rs` -- `keygen`, `sign`, `verify`
   - `commands/inspect.rs` -- `inspect`
   - `commands/package.rs` -- `package_validate`, `package_build`, `package_inspect`, `package_sign`, `package_verify`
3. Re-export all public functions from `mod.rs`.

**Dependencies**: R-08 (scaffolding fixes) and R-17 (signing consolidation) are natural to do alongside this refactoring.

---

### R-24: Optimize Test Plugin Build Process

**Addresses**: EVO-06, COR-15
**Severity of addressed findings**: Major, Observation
**Effort estimate**: Days (1-2 days)

**What to do**: Build the test plugin once per test run rather than once per test file. Isolate per-test state (signature files) in temporary directories.

**Why it matters**: Multiple test files rebuild the same test plugin independently, causing slow and potentially flaky tests. Signing tests share mutable state in the same build directory.

**Suggested approach**:
1. Use a `std::sync::Once` or build script to build the test plugin once, storing the path in an environment variable or lazy static.
2. For signing tests, copy the dylib to a per-test temporary directory before creating/deleting `.sig` files.
3. Consider using `--target-dir` to isolate concurrent builds.

**Dependencies**: None.

---

### R-25: Document Implicit Contracts and Design Decisions

**Addresses**: LEG-04, LEG-05, LEG-09, SEC-16
**Severity of addressed findings**: Minor, Major, Minor, Observation
**Effort estimate**: Days (1-2 days)

**What to do**: Document the framework's implicit contracts either as compile-time checks (preferred) or prominent documentation.

**Why it matters**: Several critical invariants exist only as implicit knowledge, causing confusing errors for newcomers.

**Suggested approach**:
1. **Unit struct requirement (LEG-05)**: Add a compile-time check in `#[plugin_impl]` that verifies the impl type is a unit struct. Emit a clear error: "fidius plugins must be unit structs".
2. **Companion module naming (LEG-04)**: Add a "Generated Items" section to the `#[plugin_interface]` doc comment.
3. **Const-eval string comparison (LEG-09)**: Add a comment explaining why manual byte comparison is needed.
4. **Send + Sync invariants (SEC-16)**: Document in `PluginHandle` that plugins must be thread-safe if called from multiple threads.

**Dependencies**: None.

---

## Architectural Recommendations

*Systemic improvements beyond individual findings.*

---

### AR-01: Adopt "Defense in Depth" at FFI Boundaries

The framework currently treats the FFI boundary as an internal API. All data crossing from plugin to host should be treated as untrusted: validate discriminant values before interpreting them, null-check pointers before dereferencing, bounds-check indices before offsetting, and return errors instead of panicking. This posture is established by R-02, R-03, R-05, and R-06 collectively. Once these are implemented, codify the principle in a contributor guide.

### AR-02: Establish Error Message Design Standards

Error messages should always include: (1) what went wrong in human-readable terms, (2) the relevant file path or resource, and (3) a suggested remediation when possible. The wire format mismatch error is the exemplar: `"wire format mismatch: plugin uses Json (debug build) but host expects Bincode (release build). Ensure both are built with the same profile."` Apply this standard to all `LoadError` and `CallError` variants.

### AR-03: Separate Security Validation from Non-Security Validation

The current `LoadPolicy` conflates security checks (signature verification) with compatibility checks (hash, version, wire format). These should be independent axes: signature enforcement should be binary (on or off), while compatibility can be strict or lenient. This is addressed by R-16 but represents a broader principle for future validation additions.

### AR-04: Plan for Async Runtime Consolidation

Each async plugin dylib gets its own multi-thread tokio runtime (PRF-005). Before the async feature stabilizes, design a mechanism for host-provided runtime injection, or switch to `current_thread` runtime since shims always use `block_on`. This prevents thread proliferation as the plugin ecosystem grows.

### AR-05: Consider Plugin Discovery Without Code Execution

The most fundamental security tension is that `dlopen` executes code before validation. For production use, consider offering a "metadata-only" discovery mode that reads binary sections to find the registry without executing constructors. This is a significant undertaking (platform-specific binary parsing) but would eliminate the code-execution-on-discovery attack surface. In the short term, R-04 (signature before dlopen) provides adequate mitigation.

---

## Summary Roadmap

### Phase 1: Critical Safety Fixes (1-2 weeks)

```
R-01 (free_buffer fix)          [Hours]   No dependencies
R-03 (null-pointer check)       [Hours]   No dependencies
R-06 (arch detection fix)       [Hours]   No dependencies
R-05 (panic -> Result)          [Hours]   No dependencies
R-15 (key file permissions)     [Hours]   No dependencies
R-13 (verify process::exit)     [Hours]   No dependencies
  |
  v
R-02 (method_count + bounds)    [Days]    After R-01
R-04 (sig before dlopen)        [Days]    After R-05
R-16 (Lenient semantics)        [Hours]   After R-04
```

All Critical and most security-Major findings are resolved after Phase 1.

### Phase 2: Usability and Reliability (2-3 weeks)

```
R-07 (error messages)           [Hours]   After R-05
R-08 (scaffolding deps)         [Hours]   No dependencies
R-09 (tracing/observability)    [Days]    After R-04
R-10 (hide raw constructors)    [Hours]   No dependencies
R-11 (build_package error)      [Hours]   No dependencies
R-12 (UnknownStatus variant)    [Hours]   No dependencies
R-14 (panic messages)           [Hours]   After R-03
R-17 (signing consolidation)    [Hours]   No dependencies
R-18 (error path tests)         [Days]    After R-02, R-14
```

### Phase 3: Structural Improvements (4-8 weeks)

```
R-19 (ABI evolution strategy)   [Weeks]   After R-02
R-20 (wire format override)     [Days]    After R-07
R-21 (typed host proxy)         [Weeks]   After R-02
R-22 (decouple facade paths)    [Days]    Coordinate with R-08
R-23 (split commands.rs)        [Hours]   Alongside R-08, R-17
R-24 (test build optimization)  [Days]    No dependencies
R-25 (document contracts)       [Days]    No dependencies
```

### Phase 4: Architectural (Ongoing)

```
AR-01 through AR-05: Ongoing architectural principles to apply as the framework matures.
```

---

## Items Requiring Further Investigation

1. **ABI evolution strategy (R-19)**: The specific mechanism (size-based, version ranges, extensions pointer) needs design work and prototyping before committing to an approach.

2. **Typed host proxy generation (R-21)**: The exact API design for the generated client struct needs consideration -- how to handle optional methods, how to integrate with `PluginHandle` for dynamic dispatch cases, and whether to support multiple interface versions.

3. **Async runtime model (AR-04)**: Whether to switch to `current_thread`, inject host runtimes, or keep per-plugin runtimes needs benchmarking and user research on expected async usage patterns.

4. **Metadata signing (SEC-09)**: Whether the signing model should cover both dylib and manifest, and how to integrate package signatures with the host loading path, needs design work.

5. **Discovery without code execution (AR-05)**: Whether to pursue binary-section parsing for metadata-only discovery depends on the framework's security posture requirements and target deployment environments.
