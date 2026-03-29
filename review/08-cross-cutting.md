# Cross-Cutting Review: Fidius Plugin Framework

**Reviewer lens**: Findings that span multiple lenses, root causes, tensions, and systemic patterns
**Date**: 2026-03-28

---

## Summary

Eight review lenses identified a combined 87 findings across legibility, correctness, evolvability, performance, API design, operability, and security. This cross-cutting analysis reveals that many of these findings trace to a small number of root causes -- primarily the absence of a typed dispatch layer over the vtable, the lack of defensive validation at the FFI trust boundary, and the absence of an observability infrastructure. When viewed together, several findings that appear "Minor" in isolation become "Major" or "Critical" due to their combined effect on the framework's safety and usability.

The framework demonstrates strong architectural separation and a clean compilation-time model. The weaknesses are concentrated in the runtime layer: what happens after a plugin is loaded and methods are called. The most impactful improvements are (1) adding vtable bounds checking and a `method_count` field, (2) replacing panic-on-invalid-data with Result-returning methods, and (3) adding a `tracing`-based observability layer. These three changes would address approximately 30 findings across all lenses.

---

## Cross-Lens Findings

### XC-01: Vtable Index Safety (7 lenses affected)

**Related findings**: COR-01, COR-02, LEG-02, LEG-03, EVO-01, PRF-008, API-01, OPS-10, SEC-04

**Relationship**: This is the single most cross-cutting issue in the codebase. Every lens identified a different facet of the same problem:

- **Correctness**: Out-of-bounds index causes undefined behavior (COR-01). Calling an unimplemented optional method dereferences null (COR-02).
- **Legibility**: Raw integer indices are opaque and error-prone (LEG-02). The flat pointer cast over mixed `fn`/`Option<fn>` types is subtle (LEG-03).
- **Evolvability**: Adding or reordering methods silently breaks all host code (EVO-01).
- **Performance**: Adding a bounds check has negligible cost (PRF-008).
- **API Design**: The calling API (`call_method(0, &input)`) provides zero type safety (API-01). `CallError::NotImplemented` exists but is never returned.
- **Operability**: The `NotImplemented` error variant is dead code -- calling an unimplemented method crashes instead of erroring (OPS-10).
- **Security**: A crafted vtable pointer enables host-side code execution (SEC-04).

**Severity assessment**: **Critical** across the board. This should be the highest-priority fix. The root cause is a single missing field (`method_count` in `PluginDescriptor`) and the absence of a typed dispatch layer.

---

### XC-02: Wire Format Debug/Release Coupling (5 lenses affected)

**Related findings**: LEG-06, LEG-12, EVO-02, API-02, API-15, OPS-03

**Relationship**: The `cfg(debug_assertions)` wire format selection creates problems across multiple dimensions:

- **Legibility**: The coupling is non-obvious (LEG-06). Error messages show raw u8 values instead of format names (LEG-12).
- **Evolvability**: No override mechanism exists; migration to a new wire format is impossible without a flag day (EVO-02).
- **API Design**: Error messages are opaque (API-02). No configuration override exists (API-15).
- **Operability**: This is likely the most common error users will encounter during development, and the error message provides no actionable guidance (OPS-03).

**Severity assessment**: Upgrade LEG-06 from Minor to **Major**. The combined effect of surprising behavior, opaque errors, and no override mechanism makes this a significant usability barrier. The wire format coupling itself is an intentional design choice (correctness over convenience), but the error reporting around it is inadequate.

---

### XC-03: discover() vs load() Behavioral Inconsistency (4 lenses affected)

**Related findings**: COR-05, API-03, SEC-02, OPS-02

**Relationship**: `discover()` and `load()` have fundamentally different security and validation postures:

- **Correctness**: Discovery skips signature verification, showing plugins that load() will reject (COR-05).
- **API Design**: The inconsistency violates the principle of least surprise (API-03).
- **Security**: Discovery executes `dlopen` on unsigned dylibs, enabling code execution from untrusted files (SEC-02).
- **Operability**: Discovery silently swallows all errors, returning empty results with no diagnostics (OPS-02).

**Severity assessment**: The security dimension (SEC-02) elevates this from a usability issue to a **Critical** safety issue. `dlopen` executes constructor code before any validation. A malicious dylib in a search path achieves code execution when the host merely calls `discover()`. COR-05 should be upgraded from **Major** to **Critical** given this security context.

---

### XC-04: Signature Path Duplication and Signing Inconsistency (3 lenses affected)

**Related findings**: LEG-11, EVO-11, API-13, SEC-09

**Relationship**: Signing-related logic is scattered across three locations with no shared abstraction:

- **Legibility**: Three identical code blocks for `.sig` path construction (LEG-11).
- **Evolvability**: Changes to the convention require three synchronized updates (EVO-11).
- **API Design**: No shared utility function exists (API-13).
- **Security**: The signing model covers only dylib content, not metadata (SEC-09).

**Severity assessment**: The duplication findings remain **Minor** individually, but the absence of a shared signing abstraction is a **Major** evolvability concern when combined with the metadata signing gap. A unified signing module would address all four findings.

---

### XC-05: Panic-on-Untrusted-Data in Descriptor Parsing (3 lenses affected)

**Related findings**: COR-13, COR-14, API-04, SEC-07

**Relationship**: Three public methods (`has_capability`, `buffer_strategy_kind`, `wire_format_kind`) panic on invalid input. These methods are called on data originating from loaded plugins:

- **Correctness**: Host crashes on programmer error (COR-13) or malformed plugin data (COR-14).
- **API Design**: Library APIs should not panic on invalid input (API-04).
- **Security**: A malformed dylib with an unknown wire format byte crashes the host -- denial of service (SEC-07).

**Severity assessment**: SEC-07 and COR-14 should be upgraded from **Major** to **Critical**. A single crafted dylib with valid magic bytes but an invalid `wire_format` or `buffer_strategy` value crashes any host that scans the directory. Combined with OPS-02 (discover swallows errors), the crash happens silently with no recovery.

---

### XC-06: `detect_architecture` Full-File Read (3 lenses affected)

**Related findings**: LEG-08, COR-08, EVO-08, PRF-001

**Relationship**: `std::fs::read(path)` loads entire dylibs into memory to inspect 20 header bytes:

- **Legibility**: Obviously wasteful to any reader (LEG-08).
- **Correctness**: OOM possible on resource-constrained systems; IO errors misclassified as LibraryNotFound (COR-08, COR-12).
- **Evolvability**: Locks in an inefficient I/O pattern (EVO-08).
- **Performance**: During discovery, every dylib is fully read (PRF-001).

**Severity assessment**: Combined with PRF-002 (signing reads the file again) and PRF-003 (discover opens every dylib), plugin discovery reads each file 2-3 times fully. This compounds to a **Major** performance concern for realistic plugin directories.

---

### XC-07: Scaffolding Version and Dependency Errors (3 lenses affected)

**Related findings**: LEG-10, COR-10, COR-11, EVO-05, API-11

**Relationship**: The CLI scaffolding commands produce incorrect dependency versions:

- **Legibility**: Hardcoded `0.1` version misleads newcomers (LEG-10).
- **Correctness**: Scaffolded projects fail to compile (COR-10, COR-11).
- **Evolvability**: Templates embedded as format strings are fragile to version bumps (EVO-05).
- **API Design**: Same resolved string used for two different crates (API-11).

**Severity assessment**: These findings should be treated as a single **Major** issue. The scaffolding commands are the primary onboarding path, and producing broken projects undermines first impressions.

---

### XC-08: `free_buffer` Capacity Mismatch (3 lenses affected)

**Related findings**: COR-03, PRF-006, SEC-08

**Relationship**: The generated `free_buffer` reconstructs a `Vec` with `capacity == len`, but the original `Vec` may have had excess capacity:

- **Correctness**: Technically undefined behavior on every method call (COR-03).
- **Performance**: Minor memory leakage per call (PRF-006).
- **Security**: Heap corruption under strict allocators (SEC-08).

**Severity assessment**: Remains **Critical**. The fix is a single line (`output_bytes.shrink_to_fit()` before `forget`), making it both high-impact and low-effort.

---

### XC-09: Builder Pattern and `process::exit` Inconsistencies (3 lenses affected)

**Related findings**: LEG-07, EVO-12, API-10, COR-18, API-12, OPS-06

**Relationship**: Two patterns of API inconsistency:

1. **Builder returns infallible Result**: LEG-07, EVO-12, API-10 all note the `PluginHostBuilder::build()` returns `Result` but cannot fail.
2. **verify uses process::exit**: COR-18, API-12, OPS-06 all note the `verify` command hard-exits instead of returning an error.

**Severity assessment**: Both remain **Minor**. They are polish issues, not safety concerns.

---

## Root Causes

### RC-01: Missing Vtable Metadata in PluginDescriptor

**Related findings**: COR-01, COR-02, LEG-02, LEG-03, EVO-01, PRF-008, API-01, OPS-10, SEC-04

**Root cause**: `PluginDescriptor` has no `method_count` field. The vtable is an opaque `*const c_void` with no size information. This single omission cascades into:

- No bounds checking possible (COR-01, SEC-04, PRF-008)
- No distinction between implemented and unimplemented optional methods at call time (COR-02, OPS-10)
- No symbolic dispatch -- only raw indices (LEG-02, API-01, EVO-01)
- Reliance on undocumented layout assumptions (LEG-03)

**Fix**: Add `method_count: u32` and `required_count: u32` fields to `PluginDescriptor`. Update layout assertions. Generate these values in the macro. Check bounds in `call_method`. This is a breaking ABI change (increment `ABI_VERSION`), but the framework is at alpha and has no deployed plugins.

---

### RC-02: No Defensive Validation at FFI Trust Boundary

**Related findings**: COR-06, COR-13, COR-14, API-04, API-08, SEC-06, SEC-07, OPS-09

**Root cause**: The host-side FFI boundary assumes well-behaved plugins. Multiple code paths panic, read null pointers, or misclassify errors when encountering unexpected data from the plugin side. The framework treats the FFI boundary as an internal API rather than a trust boundary.

**Fix**: Adopt a "defense in depth" posture for all data crossing the FFI boundary:
- Return `Result` instead of panicking on unknown discriminant values
- Null-check output pointers before creating slices
- Add `CallError::UnknownStatus` for unknown status codes
- Validate all u8 discriminants before interpreting them

---

### RC-03: Absence of Observability Infrastructure

**Related findings**: OPS-01, OPS-02, OPS-03, OPS-04, OPS-05, OPS-07, OPS-08, OPS-09, OPS-12, OPS-13, LEG-06, LEG-12, API-02

**Root cause**: The codebase has no logging, tracing, or structured diagnostic framework. Every operability finding traces back to this gap. Error messages use raw values because there is no diagnostic context layer. Discovery swallows errors silently because there is no way to emit diagnostics. The CLI has no verbose mode because there is no output infrastructure to enable.

**Fix**: Add `tracing` as an optional dependency behind a feature flag. Instrument `load()`, `discover()`, `call_method()`, and `verify_signature()` with spans and events. This single change addresses or mitigates approximately 13 findings.

---

### RC-04: No Shared Utility Layer for Cross-Crate Concerns

**Related findings**: LEG-11, LEG-17, EVO-11, API-13, COR-10, COR-11

**Root cause**: Several pieces of logic are duplicated across `fidius-cli` and `fidius-host` because there is no shared utility layer for operations like:
- Signature file path construction (3 duplications)
- Build invocation (2 duplications)
- Dependency resolution (fragmented across CLI commands)

**Fix**: Create a shared module (in `fidius-core` or `fidius-host`) for these cross-cutting utilities. The CLI should delegate to library functions rather than reimplementing.

---

### RC-05: Wire Format Selection Architecture

**Related findings**: LEG-06, LEG-12, EVO-02, API-02, API-15, OPS-03

**Root cause**: The decision to couple wire format to `cfg(debug_assertions)` is an architectural choice that optimizes for simplicity (no configuration needed) at the cost of flexibility and debuggability. The choice itself is defensible for an alpha, but the error reporting around mismatches was not designed to compensate for the user confusion this coupling creates.

**Fix**: Two-phase approach: (1) Immediately improve error messages to include format names and build profile hints. (2) Later, add a feature flag override mechanism (`force-json`, `force-bincode`).

---

## Tensions

### T-01: Safety vs. Performance (vtable dispatch)

**Lenses in tension**: Correctness/Security vs. Performance

The vtable dispatch uses raw pointer arithmetic for zero-overhead function calls. Adding bounds checking, null checks, and capability validation adds a small cost per call. However, PRF-008 explicitly notes this cost is negligible compared to serialization overhead (~microseconds for serialize vs. ~nanoseconds for a bounds check).

**Assessment**: The tension is illusory. The performance cost of safety checks is unmeasurably small relative to the serialization cost that dominates every call. Safety should win unconditionally here.

---

### T-02: Simplicity vs. Debuggability (wire format coupling)

**Lenses in tension**: Evolvability vs. Legibility/Operability

The `cfg(debug_assertions)` wire format selection provides automatic JSON-for-debugging without any user configuration. This is simpler than a feature flag or runtime option. However, it creates a non-obvious coupling that produces confusing errors when build profiles are mixed.

**Assessment**: The simplicity benefit is real for single-developer workflows. The debuggability cost is real for team workflows, CI/CD pipelines, and plugin ecosystems with mixed builds. For an alpha, the current choice is acceptable. For beta, an override mechanism should be added. The immediate priority is better error messages.

---

### T-03: Security vs. Usability (signature enforcement)

**Lenses in tension**: Security vs. Operability/API Design

Strict signature enforcement (verify before dlopen) would prevent SEC-02 (code execution via discover) and SEC-05 (code execution via inspect). However, it would require all plugins to be signed during development, adding friction. The `Lenient` policy attempts to bridge this gap but creates a false sense of security (SEC-03).

**Assessment**: The current design conflates two concerns: "should we verify signatures?" and "what happens when verification fails?" The `Lenient` policy should not allow bypassing signature verification; instead, it should control what happens for other validation failures (hash mismatch, version mismatch). Signature verification, when enabled, should always be enforced. For development workflows without signing, `require_signature = false` already exists.

---

### T-04: Statelessness vs. Expressiveness (plugin model)

**Lenses in tension**: Evolvability vs. Security

The framework enforces stateless plugins (unit structs, `&self` only) for simplicity and thread safety. This limits expressiveness: plugins cannot maintain configuration, caches, or connections. Supporting stateful plugins (EVO change cost analysis, Change 5) would require a near-complete rewrite.

**Assessment**: This is a conscious, well-reasoned architectural constraint. Stateless plugins eliminate entire classes of bugs (data races, lifecycle management, initialization ordering). The constraint should be documented prominently as a design choice rather than a limitation. If stateful plugins are ever needed, they should be a separate plugin category, not a relaxation of the existing model.

---

### T-05: ABI Stability vs. Framework Evolution

**Lenses in tension**: Evolvability vs. Correctness

The layout assertion tests (EVO-14) provide strong ABI stability guarantees, catching any accidental struct changes. However, the exact-version checking for `ABI_VERSION` and `REGISTRY_VERSION` (EVO-03) means any intentional change requires a flag-day migration. There is no mechanism for backward-compatible evolution.

**Assessment**: For an alpha framework with zero deployed plugins, exact-version checking is correct -- it prevents subtle incompatibilities. Before beta, a forward-compatibility mechanism (e.g., a size field for extensible structs, version ranges) should be designed. The layout tests should be kept regardless.

---

## Systemic Patterns

### SP-01: Strong Compile-Time Model, Weak Runtime Model

The framework invests heavily in compile-time correctness: proc macros validate trait signatures, reject `&mut self`, enforce `Send + Sync`, generate correct shims, and compute deterministic interface hashes. The compile-time experience is polished.

The runtime model, by contrast, is permissive: no bounds checking, no null checks, panicking parsers, silent error swallowing, no logging. The gap between compile-time rigor and runtime permissiveness is the codebase's most systemic weakness.

**Pattern**: Of the 7 Critical findings across all lenses, 6 are runtime issues. Only 1 (EVO-02, wire format coupling) is a compile-time/design issue.

---

### SP-02: Correct Happy Path, Fragile Error Paths

The primary workflow (define trait, implement, compile, load, call) is well-tested and works correctly. Every integration test exercises the happy path. Error paths -- malformed plugins, missing signatures, panicking methods, unknown status codes, unimplemented optional methods -- are largely untested (COR-16, COR-05 untested, OPS-10 dead code) and often incorrect (panic instead of error, wrong error variant, silent swallowing).

**Pattern**: The test suite has zero tests for any error path through `call_method`. No test sends an invalid vtable index, calls an unimplemented optional method, triggers a plugin panic, or returns a `Result::Err` from a plugin method.

---

### SP-03: Documentation and Code Quality Are Above Average, But Implicit Contracts Undermine Both

Doc comments are present on nearly every public item. Module organization is clean. The IR layer in the macro crate is well-designed. But several critical invariants exist only as implicit knowledge:

- Plugin types must be unit structs
- Vtable index must match trait method declaration order
- `Option<fn>` and bare `fn` must have the same size (nullable pointer optimization)
- Debug-built plugins are incompatible with release-built hosts
- `discover()` does not check signatures
- `Lenient` policy allows unsigned plugins even when signatures are required

These implicit contracts are the primary source of both safety bugs and user confusion.

---

### SP-04: Alpha-Appropriate Scope with Production-Suggestive Features

The framework is at `0.0.0-alpha.1`, and many design choices (simple signing model, stateless plugins, exact version checking) are appropriate for this stage. However, the presence of features like `LoadPolicy::Lenient`, multiple buffer strategy types in the enum, package dependency declarations, and `source_hash` fields suggest production aspirations. These half-implemented features create false expectations.

**Pattern**: 4 findings (EVO-09, EVO-10, SEC-13, API-05) relate to features that are declared in the type system but not implemented. Each creates a gap between what the API promises and what it delivers.

---

## Severity Adjustments

| Finding | Original Severity | Adjusted Severity | Rationale |
|---------|------------------|-------------------|-----------|
| COR-05 (discover skips signatures) | Major | **Critical** | SEC-02 reveals that discover executes `dlopen` on unsigned dylibs. Combined, this is a code execution vulnerability, not just an inconsistency. |
| COR-14 (descriptor accessors panic on unknown values) | Minor | **Major** | SEC-07 shows this enables DoS from a single malformed dylib in a search path. The panic occurs during validation of untrusted data. |
| LEG-06 (wire format debug/release coupling) | Minor | **Major** | Combined with OPS-03, API-02, and API-15, this is the most likely user-facing error during development, and the error message is unhelpful. |
| PRF-001 (detect_architecture reads entire file) | Major | **Critical** | Combined with PRF-002, PRF-003, and COR-12 (IO error misclassification), discovery of a directory with large dylibs causes excessive memory use and misleading errors. The full-read also affects signing (PRF-002), meaning each plugin load reads the file fully at least twice. |
| OPS-10 (NotImplemented variant is dead code) | Minor | **Major** | Combined with COR-02 and SEC-04, the dead variant is not just unused code -- it represents a designed-but-unimplemented safety mechanism. Calling an unimplemented optional method causes a segfault. |
| LEG-08 (detect_architecture reads entire file) | Minor | **Major** | Duplicates PRF-001; severity aligned to match the cross-cutting assessment. |
| COR-13 (has_capability panics) | Minor | **Major** | Part of the systemic pattern (RC-02) of panicking on invalid input in a library API. Combined with SEC-07. |
| API-08 (unknown status code as Serialization) | Major | **Major** | No change in severity, but flagged as part of the broader RC-02 pattern. This becomes more important if new status codes are added in future versions. |
