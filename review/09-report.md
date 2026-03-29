# Architecture Review Report: Fidius Plugin Framework

**Version**: 0.0.0-alpha.1
**Date**: 2026-03-28
**Codebase**: Fidius -- Rust plugin framework for dynamically loaded cdylibs with stable C ABI

---

## Executive Summary

Fidius is a well-architected Rust plugin framework with strong compile-time correctness, clean crate decomposition, and a thoughtful separation between plugin authoring (macros + core types), host loading (runtime validation + FFI calling), and developer tooling (CLI). The macro API provides a natural, idiomatic annotation model, and the full pipeline from trait definition through plugin loading works correctly on the happy path. Documentation quality is above average, with doc comments on nearly every public item, good examples in the facade crate, and a comprehensive specification.

However, the review identified serious weaknesses concentrated in the runtime layer -- specifically at the FFI trust boundary between host and plugin. The three most critical issues are: (1) unchecked vtable index arithmetic enabling undefined behavior and potential code execution, (2) `free_buffer` using incorrect Vec capacity on every method call (unsound under strict allocators), and (3) `discover()` executing `dlopen` on unsigned dylibs, enabling code execution from untrusted files placed in search paths. These issues trace to two root causes: a missing `method_count` field in `PluginDescriptor` and the absence of defensive validation at the FFI boundary. A secondary systemic concern is the complete absence of logging or tracing infrastructure, which makes debugging plugin loading failures unnecessarily difficult. The framework's compile-time rigor is not matched by its runtime defensiveness -- of the 13 Critical findings across all lenses, 12 are runtime issues.

---

## Summary Table

| Lens | Critical | Major | Minor | Observation | Total |
|------|----------|-------|-------|-------------|-------|
| Legibility (LEG) | 0 | 4 | 7 | 5 | 16 |
| Correctness (COR) | 3 (7 raw*) | 6 | 5 | 5 | 19 |
| Evolvability (EVO) | 3 | 5 | 4 | 3 | 15 |
| Performance (PRF) | 0 (1 adj**) | 1 | 3 | 6 | 10 |
| API Design (API) | 3 | 5 | 7 | 4 | 19 |
| Operability (OPS) | 2 | 3 | 5 | 3 | 13 |
| Security (SEC) | 3 | 5 | 5 | 4 | 17 |
| **Total (pre-adjustment)** | **14** | **29** | **36** | **30** | **109** |

*Many findings overlap across lenses. After deduplication via cross-cutting analysis, the unique finding count is approximately 55.*

**After cross-cutting severity adjustments:**

| Finding | Original | Adjusted | Rationale |
|---------|----------|----------|-----------|
| COR-05 | Major | **Critical** | Combined with SEC-02: discover executes dlopen on unsigned dylibs |
| COR-14 | Minor | **Major** | Combined with SEC-07: enables DoS from malformed plugins |
| LEG-06 | Minor | **Major** | Most likely user-facing error with unhelpful messages |
| PRF-001 | Major | **Critical** | Discovery reads each dylib fully 2-3 times; OOM risk |
| OPS-10 | Minor | **Major** | Dead error variant masks segfault on unimplemented methods |
| LEG-08 | Minor | **Major** | Aligned with PRF-001 cross-cutting assessment |
| COR-13 | Minor | **Major** | Part of systemic panic-on-untrusted-data pattern |

---

## Findings by Lens

### Legibility

The codebase is remarkably legible for an FFI-heavy proc-macro framework. Module organization follows the plugin lifecycle cleanly, doc comments are present on nearly every public item, and the code favors clarity over cleverness. The facade crate provides excellent usage examples (LEG-18, positive). The macro crate's IR layer cleanly separates parsing from code generation.

The primary legibility barriers are the repository/crate naming mismatch (`fides` vs `fidius`) and the raw integer vtable indices in the calling API.

| ID | Finding | Severity |
|----|---------|----------|
| LEG-01 | Repository name `fides` vs crate prefix `fidius` -- unexplained mismatch | Major |
| LEG-02 | `call_method` uses raw integer indices with no symbolic names | Major |
| LEG-03 | Unsafe vtable pointer arithmetic assumes flat function pointer layout | Major |
| LEG-04 | Generated companion module naming convention undocumented | Minor |
| LEG-05 | Unit struct requirement is implicit with confusing errors | Major |
| LEG-06 | Wire format debug/release coupling is non-obvious | **Major** (adj. from Minor) |
| LEG-07 | `PluginHost` and `PluginHostBuilder` have duplicate fields | Minor |
| LEG-08 | `detect_architecture` reads entire file into memory | **Major** (adj. from Minor) |
| LEG-09 | Const-eval string comparison loop is hard to follow | Minor |
| LEG-10 | Hardcoded `fidius-core = { version = "0.1" }` in scaffold | Minor |
| LEG-11 | Duplicated `.sig` path construction logic | Minor |
| LEG-12 | Mismatch errors display raw u8 values | Minor |
| LEG-13 | `_has_async` variable computed but unused | Observation |
| LEG-14 | `check_crates_io` User-Agent references wrong GitHub org | Observation |
| LEG-15 | `build_package` returns target directory when cdylib not found | Observation |
| LEG-16 | Wildcard re-exports obscure public API origin | Minor |
| LEG-17 | CLI duplicates `build_package` logic from host crate | Observation |
| LEG-18 | Facade crate doc examples show good usage patterns | Observation (positive) |

---

### Correctness

The core happy path is solid and well-tested end-to-end. ABI layout stability is protected by assertion tests, wire format round-trips are verified, and the full pipeline test provides high confidence in the primary workflow. However, the unsafe FFI layer contains critical issues: no vtable bounds checking, incorrect `free_buffer` capacity, and several untested error paths (panic handling, Result-returning methods, deserialization failure). The test suite has zero tests for any error path through `call_method`.

| ID | Finding | Severity |
|----|---------|----------|
| COR-01 | vtable index out-of-bounds causes undefined behavior | Critical |
| COR-02 | Optional method vtable slots assumed same size as bare fn pointers; calling None causes segfault | Critical |
| COR-03 | `free_buffer` uses incorrect Vec capacity -- UB on every call | Critical |
| COR-04 | Output size truncation from usize to u32 | Major |
| COR-05 | `discover()` silently skips signature verification | **Critical** (adj. from Major) |
| COR-06 | `STATUS_PANIC` path may leave output buffer in inconsistent state | Major |
| COR-07 | `build_registry` truncates plugin count from usize to u32 | Major |
| COR-08 | `detect_architecture` reads entire file into memory | Major |
| COR-09 | `signing::verify_signature` reads entire dylib into memory | Major |
| COR-10 | `init_interface` uses same dep string for both `fidius` and `fidius-core` | Minor |
| COR-11 | `init_plugin` hardcodes `fidius-core = { version = "0.1" }` | Minor |
| COR-12 | `detect_architecture` maps IO error to `LibraryNotFound` | Minor |
| COR-13 | `has_capability` panics instead of returning error | **Major** (adj. from Minor) |
| COR-14 | `buffer_strategy_kind()` and `wire_format_kind()` panic on unknown values | **Major** (adj. from Minor) |
| COR-15 | E2E signing tests share mutable state | Observation |
| COR-16 | No test for Result-returning plugin methods through the host | Observation |
| COR-17 | `package_sign` signs package.toml using dylib signing function | Observation |
| COR-18 | `verify` function calls `process::exit(1)` instead of returning error | Observation |
| COR-19 | Hash known vectors test verifies determinism but not specific values | Observation |

---

### Evolvability

The five-crate decomposition is sound with an acyclic dependency graph and clear responsibilities. The macro crate's IR layer enables safe refactoring (EVO-13, positive), and layout assertion tests provide strong ABI stability guardrails (EVO-14, positive). The `PluginInfo` abstraction decouples host internals from FFI layout (EVO-15, positive).

Key evolvability risks: no ABI versioning or migration strategy, wire format irrevocably coupled to build profile, and generated code tightly coupled to facade crate module paths.

| ID | Finding | Severity |
|----|---------|----------|
| EVO-01 | VTable index-based dispatch is fragile and unsafe | Critical |
| EVO-02 | Wire format selection coupled to build profile with no override | Critical |
| EVO-03 | No ABI versioning or migration strategy | Critical |
| EVO-04 | Generated code couples tightly to facade crate module paths | Major |
| EVO-05 | CLI scaffolding embeds hardcoded version strings | Major |
| EVO-06 | Test suite rebuilds test plugin on every invocation | Major |
| EVO-07 | `commands.rs` is a monolithic file | Major |
| EVO-08 | `detect_architecture` reads entire file | Major |
| EVO-09 | Package dependency declarations exist but are inert | Minor |
| EVO-10 | `source_hash` field parsed but never computed/validated | Minor |
| EVO-11 | Signing logic duplicated between CLI and host | Minor |
| EVO-12 | `PluginHost::builder().build()` returns Result but cannot fail | Minor |
| EVO-13 | Proc macro IR layer enables safe refactoring | Observation (positive) |
| EVO-14 | Layout assertion tests provide strong ABI guardrails | Observation (positive) |
| EVO-15 | `PluginInfo` owned copy enables safe evolution | Observation (positive) |

---

### Performance

The framework's performance profile is generally appropriate for its workload. The hot path (method calling) is thin: O(1) vtable lookup, direct function pointer invocation, with serialization as the dominant cost. The framework correctly avoids premature optimization. Per-call `catch_unwind` overhead is negligible compared to serialization (PRF-004, positive observation).

The significant findings are in cold paths: `detect_architecture` reading entire dylibs, discovery loading and discarding every dylib, and per-plugin tokio runtimes for async.

| ID | Finding | Severity |
|----|---------|----------|
| PRF-001 | `detect_architecture` reads entire dylib into memory | **Critical** (adj. from Major) |
| PRF-002 | Signing reads entire dylib twice during load() | Minor |
| PRF-003 | `discover()` loads and validates every dylib, discarding results | Minor |
| PRF-004 | Per-call `catch_unwind` overhead (negligible) | Observation (positive) |
| PRF-005 | Multi-thread tokio runtime per plugin dylib | Minor |
| PRF-006 | `free_buffer` reconstructs Vec with len==capacity assumption | Observation |
| PRF-007 | Registry `build_registry` leaks Vec intentionally (correct) | Observation (positive) |
| PRF-008 | No vtable bounds checking (negligible cost to add) | Observation |
| PRF-009 | Bincode v1 vs v2 (future consideration) | Observation |
| PRF-010 | `interface_hash` allocates (compile-time only, no runtime impact) | Observation |

---

### API Design

The macro API is the strongest surface -- natural, idiomatic Rust annotation model that makes the common case straightforward. The CLI follows standard platform conventions (API-19, positive). The builder pattern is well-implemented (API-17, positive). The package manifest's generic metadata schema is elegant (API-18, positive).

The primary API weakness is `call_method` requiring raw vtable indices with no type safety, creating a large gap between the type-safe trait interface and the untyped host-side dispatch.

| ID | Finding | Severity |
|----|---------|----------|
| API-01 | `call_method` requires raw vtable indices -- unsafe and unergonomic | Critical |
| API-02 | Error types display raw discriminant values | Critical |
| API-03 | `discover()` silently skips signature verification vs `load()` | Critical |
| API-04 | `has_capability` and descriptor accessors panic on invalid input | Major |
| API-05 | `PluginError` stores structured details as JSON string | Major |
| API-06 | `PluginHandle::new()` exposes raw FFI internals publicly | Major |
| API-07 | `build_package` returns directory path instead of error | Major |
| API-08 | Unknown FFI status code reported as `CallError::Serialization` | Major |
| API-09 | `fidius_plugin_registry!()` in `fidius_core` but generated code references `fidius` | Minor |
| API-10 | `PluginHostBuilder` duplicates fields with no validation | Minor |
| API-11 | `init_interface` resolves same dep string for two different crates | Minor |
| API-12 | `verify` CLI calls `process::exit(1)` instead of returning error | Minor |
| API-13 | Signature path construction duplicated three times | Minor |
| API-14 | `LoadedPlugin` exposes raw pointers in public fields | Minor |
| API-15 | Wire format has no override mechanism | Minor |
| API-16 | Facade crate doc examples clearly demonstrate both roles | Observation (positive) |
| API-17 | Builder pattern uses consistent idiomatic conventions | Observation (positive) |
| API-18 | Package manifest API uses generics effectively | Observation (positive) |
| API-19 | CLI command structure follows platform conventions | Observation (positive) |

---

### Operability

The framework has almost no operability infrastructure. There is no logging, no tracing, no structured diagnostics, and no verbose mode in the CLI. The most critical gap is that `discover()` silently swallows all errors, returning empty results when plugins exist but fail validation. Error messages for the most common failure (wire format mismatch from debug/release mixing) display raw u8 discriminants.

The `fidius inspect` command is a useful diagnostic tool (OPS-11, positive).

| ID | Finding | Severity |
|----|---------|----------|
| OPS-01 | No logging or tracing framework anywhere in codebase | Critical |
| OPS-02 | `discover()` silently swallows all errors | Critical |
| OPS-03 | Error messages display raw discriminant values | Major |
| OPS-04 | `load()` does not report search paths or rejected candidates | Major |
| OPS-05 | CLI has no `--verbose` or `--debug` flag | Major |
| OPS-06 | `verify` calls `process::exit(1)` | Minor |
| OPS-07 | `InvalidMagic` error provides no file context | Minor |
| OPS-08 | `detect_architecture` misclassifies IO errors | Minor |
| OPS-09 | `CallError::Panic` does not preserve panic message | Minor |
| OPS-10 | `CallError::NotImplemented` variant is dead code | **Major** (adj. from Minor) |
| OPS-11 | `fidius inspect` is a useful diagnostic tool | Observation (positive) |
| OPS-12 | Lenient policy warnings are unstructured | Observation |
| OPS-13 | Async runtime init failure panics with generic message | Observation |

---

### Security

The framework's Ed25519 signing model is cryptographically sound (correct key generation, well-audited verification library, straightforward sign-then-verify scheme). However, several gaps undermine the trust model in practice: `discover()` executes `dlopen` without signature checks, `Lenient` policy defeats signature enforcement, and `inspect` opens arbitrary dylibs without verification. The FFI boundary is the highest-impact attack surface, with unchecked vtable indices and null pointer dereference enabling host-side exploitation from malicious plugins.

| ID | Finding | Severity |
|----|---------|----------|
| SEC-01 | Secret key files written with no permission restrictions | Major |
| SEC-02 | `discover()` bypasses signature verification (dlopen on unsigned) | Major |
| SEC-03 | `LoadPolicy::Lenient` defeats signature enforcement | Major |
| SEC-04 | Unchecked vtable index enables host-side code execution | Critical |
| SEC-05 | `inspect` command executes dlopen without signature check | Major |
| SEC-06 | No null-pointer check on output buffer | Critical |
| SEC-07 | Descriptor field parsing panics on unknown values (DoS) | Major |
| SEC-08 | `free_buffer` uses incorrect capacity (heap corruption) | Critical |
| SEC-09 | Signing covers dylib only, not metadata | Minor |
| SEC-10 | `package build` executes cargo with user-controlled paths | Minor |
| SEC-11 | Key files use raw 32-byte format with no type indicator | Minor |
| SEC-12 | `check_crates_io` unauthenticated HTTP (low risk) | Minor |
| SEC-13 | No key revocation mechanism | Minor |
| SEC-14 | Signature verification and dlopen are not atomic (TOCTOU) | Observation |
| SEC-15 | `dlopen` executes constructor code before validation | Observation |
| SEC-16 | `Send + Sync` on `PluginHandle` relies on undocumented invariants | Observation |
| SEC-17 | Deserialization of untrusted data (acceptable given trust model) | Observation |

---

## Cross-Cutting Concerns

### Root Causes

Five root causes account for the majority of findings:

**RC-01: Missing Vtable Metadata in PluginDescriptor**
`PluginDescriptor` has no `method_count` field. The vtable is an opaque `*const c_void` with no size information. This single omission cascades into 9+ findings across all lenses: no bounds checking (COR-01, SEC-04), no null-fn detection for optional methods (COR-02, OPS-10), no symbolic dispatch (LEG-02, API-01, EVO-01), and undocumented layout assumptions (LEG-03).

**RC-02: No Defensive Validation at FFI Trust Boundary**
The host treats the FFI boundary as an internal API rather than a trust boundary. Multiple code paths panic on untrusted data (COR-13, COR-14, SEC-07), read null pointers (SEC-06, COR-06), or misclassify errors (API-08, COR-12). 8+ findings trace to this root cause.

**RC-03: Absence of Observability Infrastructure**
Zero structured logging or tracing. Every operability finding (OPS-01 through OPS-13) and several legibility and API design findings trace to this gap. Approximately 13 findings would be addressed or mitigated by adding `tracing`.

**RC-04: No Shared Utility Layer for Cross-Crate Concerns**
Signature path construction duplicated 3 times, build logic duplicated between CLI and host, dependency resolution fragmented. 6 findings across legibility, evolvability, and API design.

**RC-05: Wire Format Selection Architecture**
`cfg(debug_assertions)` coupling is defensible for alpha but the error reporting around mismatches was not designed to compensate for user confusion. 6 findings across legibility, evolvability, API design, and operability.

### Systemic Patterns

**SP-01: Strong Compile-Time Model, Weak Runtime Model.** The framework invests heavily in compile-time correctness (macro validation, trait bounds, deterministic hashing) but the runtime layer is permissive (no bounds checking, no null checks, panicking parsers, silent error swallowing). Of 13 Critical findings, 12 are runtime issues.

**SP-02: Correct Happy Path, Fragile Error Paths.** The primary workflow is well-tested end-to-end. Error paths are largely untested and often incorrect. The test suite has zero tests for any error path through `call_method`.

**SP-03: Implicit Contracts Undermine Good Documentation.** Several critical invariants exist only as implicit knowledge: unit struct requirement, vtable index ordering, `Option<fn>` layout assumption, debug/release wire format incompatibility, discover/load behavioral differences.

**SP-04: Alpha-Appropriate Scope with Production-Suggestive Features.** Half-implemented features (package dependencies, source_hash, Lenient policy, multiple buffer strategies) create false expectations about the framework's maturity.

### Key Tensions

| Tension | Assessment |
|---------|------------|
| Safety vs. Performance (vtable bounds checking) | Illusory: bounds check cost (~ns) is negligible vs serialization (~us). Safety wins. |
| Simplicity vs. Debuggability (wire format coupling) | Real but manageable: keep the default, add an override, improve error messages. |
| Security vs. Usability (signature enforcement) | Lenient policy conflates two concerns. Signature checks should always be enforced when enabled; Lenient should control only non-security validation. |
| ABI Stability vs. Framework Evolution | Exact version checking is correct for alpha. Forward-compatibility mechanism needed before beta. |

---

## Appendix: System Overview

For a complete description of the system architecture, crate decomposition, data flow, public interface surface, dependency graph, build process, test infrastructure, and conventions, see [00-system-overview.md](./00-system-overview.md).

Key architectural facts:
- **5 workspace crates** + 1 standalone test fixture, organized by plugin lifecycle role
- **Plugin compilation**: trait annotation -> proc macro -> vtable struct + FFI shims + descriptor + inventory registration
- **Plugin loading**: search paths -> architecture check -> optional signature verification -> dlopen -> registry validation -> descriptor extraction
- **Method calling**: serialize input -> vtable function pointer call -> deserialize output -> free buffer
- **Wire format**: JSON in debug builds, bincode in release builds (determined by `cfg(debug_assertions)`)
- **Signing**: Ed25519 detached signatures over raw dylib bytes
- **Test coverage**: Unit tests, integration tests, compile-fail tests, full-pipeline end-to-end test
