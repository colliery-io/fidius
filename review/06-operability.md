# Operability Review: Fidius Plugin Framework

**Reviewer lens**: Can this be run, observed, debugged, and scaled in production?

**Scope note**: Fidius is a library + CLI, not a long-running service. This review evaluates operability through the lens of "how do consumers debug plugin loading failures" and "how does the CLI surface problems."

**Finding prefix**: OPS-
**Date**: 2026-03-28

---

## Summary

The fidius framework has almost no operability infrastructure. There is no logging framework, no structured error context, no tracing, and no diagnostic tooling beyond `println!`/`eprintln!`. The CLI surfaces errors as bare strings with no debug context (no file paths in most error chains, no suggestion of remediation). The host library silently swallows errors during discovery and emits unstructured warnings to stderr under lenient policy.

For a library at alpha stage, some of this is expected. However, the framework's core value proposition -- dynamically loading untrusted code across an FFI boundary -- is inherently difficult to debug. Users will encounter plugin loading failures from architecture mismatches, wire format mismatches, signature failures, and ABI version drift. The current error reporting makes diagnosing these failures unnecessarily difficult. A host application that loads fidius plugins has no way to observe what the framework is doing, why a plugin was rejected, or what candidates were considered.

The CLI is somewhat better: it validates inputs, provides clear output for happy paths, and exits with appropriate status codes (with one exception). But it lacks `--verbose` or `--debug` flags, provides no structured output format (e.g., JSON), and has no way to diagnose partial failures during scaffolding.

**Finding count**: 2 Critical, 3 Major, 5 Minor, 3 Observations

---

## Observability Assessment

### Logging

**Status: Absent**

The entire codebase uses only `println!` and `eprintln!` for output. There is no logging framework (no `tracing`, no `log`, no `env_logger`). This means:

- **Host library consumers** cannot see what the framework is doing. `PluginHost::load()` iterates search paths, opens dylibs, checks architecture, verifies signatures, validates registries -- and if something goes wrong partway through, the consumer gets either `Ok(plugin)` or `Err(LoadError::PluginNotFound)` with no indication of which candidates were tried or why they were rejected.

- **CLI users** get minimal output. Success paths print helpful summaries. Error paths print `"error: {e}"` with whatever the `Display` impl provides. There is no `--verbose` flag to see what the CLI is doing step by step.

- **Plugin authors** debugging compilation or loading issues have no framework-level diagnostics. The proc macro emits Rust compiler errors for some failure modes (missing version, `&mut self`, unsupported buffer), but others produce cryptic compiler errors (non-unit struct) or runtime failures with no context (wire format mismatch shows raw u8 values).

### Metrics

**Status: Not applicable**

As a library/CLI, metrics are not expected. However, a host application embedding fidius has no way to instrument plugin loading time, call latency, serialization overhead, or error rates. If the library exposed timing hooks or metric callbacks, hosts could integrate with their observability stack.

### Tracing

**Status: Absent**

No distributed tracing, no span context, no correlation IDs. For plugin method calls, a host cannot trace a request through the FFI boundary into the plugin. This is understandable for an alpha library, but worth flagging as a gap for production use.

### Diagnostic Tooling

**Status: Minimal**

The `fidius inspect` CLI command provides useful diagnostic output: it loads a dylib and displays the registry contents (plugin count, names, interface hash, version, wire format, buffer strategy, capabilities). This is the primary debugging tool for "is my plugin built correctly?"

Missing diagnostic capabilities:
- No way to inspect a dylib's architecture without loading it (e.g., `fidius arch-check <dylib>`)
- No way to compare a plugin's interface hash against a host's expected hash
- No way to validate that a plugin in directory X would be loadable by a host configuration Y
- No dry-run mode for `PluginHost::load()` that reports all candidates and why each was accepted/rejected

---

## Failure Mode Analysis

### Plugin Loading Failures

| Failure Mode | Detection | Error Quality | Diagnosability |
|---|---|---|---|
| Dylib not found | `LoadError::PluginNotFound` | Shows plugin name only; does not list searched paths | Poor -- user must guess which directories were searched |
| Architecture mismatch | `LoadError::ArchitectureMismatch` | Shows expected vs got format/arch | Adequate |
| Missing `fidius_get_registry` | `LoadError::SymbolNotFound` | Shows dylib path | Adequate |
| Invalid magic bytes | `LoadError::InvalidMagic` | No context (which file?) | Poor -- no file path in error |
| ABI version mismatch | `LoadError::IncompatibleAbiVersion` | Shows got vs expected | Adequate |
| Wire format mismatch | `LoadError::WireFormatMismatch` | Shows raw u8 values (`0` vs `1`) | Poor -- user must look up what 0 and 1 mean |
| Signature missing | `LoadError::SignatureRequired` | Shows dylib path | Adequate |
| Signature invalid | `LoadError::SignatureInvalid` | Shows dylib path | Adequate |
| Buffer strategy mismatch | `LoadError::BufferStrategyMismatch` | Shows raw u8 values | Poor |

### Plugin Call Failures

| Failure Mode | Detection | Error Quality | Diagnosability |
|---|---|---|---|
| Serialization error | `CallError::Serialization` | Includes error message | Adequate |
| Deserialization error | `CallError::Deserialization` | Includes error message | Adequate |
| Plugin panic | `CallError::Panic` | No panic message preserved | Poor -- no backtrace, no panic payload |
| Plugin error | `CallError::Plugin(PluginError)` | Code + message + optional details | Good |
| vtable index OOB | Undefined behavior | N/A | Catastrophic -- silent UB, no detection |
| Unimplemented optional method | `CallError::NotImplemented` exists but is never returned | N/A | The variant exists but no code path produces it |

### Discovery Failures

| Failure Mode | Detection | Error Quality | Diagnosability |
|---|---|---|---|
| Invalid dylib in search path | Silently skipped | No output at all | Poor -- `discover()` returns `Ok(vec![])` with no indication that candidates were found and rejected |
| Search path does not exist | Silently skipped | No output | Poor |
| Permission denied on dylib | `LoadError::LibraryNotFound` | Misclassified as "not found" | Actively misleading |

---

## Findings

### OPS-01: No logging or tracing framework anywhere in the codebase [Critical]

**Location**: Entire codebase
**Confidence**: High

**Description**: The framework has zero structured logging. All output is via `println!`/`eprintln!`. The host library (`fidius-host`) is completely silent -- it provides no diagnostic output whatsoever. The only runtime output from the library is a single `eprintln!("fidius warning: {e}")` in lenient mode signature checking (`host.rs:193`).

For a framework whose primary job is loading untrusted dynamic libraries across an FFI boundary, this is a critical operability gap. When a plugin fails to load, the consumer receives a final error but has no visibility into the loading pipeline: which directories were searched, which files were considered, which validation step failed for each candidate, and why.

**Impact**: Host application developers cannot debug plugin loading issues without reading fidius source code and adding their own instrumentation. In production, plugin loading failures are opaque.

**Recommendation**: Add the `tracing` crate as an optional dependency (behind a feature flag). Instrument key operations with spans and events:
- `PluginHost::load()`: span with plugin name, events for each candidate dylib tried
- `PluginHost::discover()`: span with search paths, events for each dylib found/rejected
- `load_library()`: span with path, events for each validation step
- `verify_signature()`: event for signature check result
- `call_method()`: span with method index, events for serialization size, status code

At minimum, add `log` crate support so consumers can use `RUST_LOG=fidius=debug` to see what the framework is doing.

---

### OPS-02: `discover()` silently swallows all errors, returning empty results with no diagnostics [Critical]

**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/host.rs`, lines 128-167
**Confidence**: High

**Description**: `PluginHost::discover()` catches all `Err` results from `load_library()` and silently continues (`Err(_) => continue`). It also silently skips directories that do not exist (`!search_path.is_dir() => continue`). If every dylib in the search paths fails validation, the method returns `Ok(vec![])` -- an empty success.

A host application calling `discover()` and receiving an empty list has no way to distinguish between "no plugins exist" and "many plugins exist but all failed validation." This is the single most likely source of user confusion: "I compiled my plugin, put it in the right directory, but the host says there are no plugins."

**Evidence**:
```rust
match loader::load_library(&path) {
    Ok(loaded) => { /* ... */ }
    Err(_) => {
        // Skip invalid dylibs during discovery
        continue;
    }
}
```

**Impact**: Users cannot debug plugin discovery failures. The most common troubleshooting scenario (plugin present but not discovered) is completely opaque.

**Recommendation**: At minimum, collect the errors and provide a method to retrieve them (e.g., `discover_with_diagnostics() -> (Vec<PluginInfo>, Vec<(PathBuf, LoadError)>)`). With logging (OPS-01), emit `debug!`-level events for each skipped candidate with the reason.

---

### OPS-03: Error messages display raw discriminant values instead of human-readable names [Major]

**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/error.rs`, lines 40-44
**Confidence**: High

**Description**: `LoadError::WireFormatMismatch` and `LoadError::BufferStrategyMismatch` store and display raw `u8` discriminant values. A user seeing `"wire format mismatch: got 0, expected 1"` must look up the discriminant mapping (`0 = Json, 1 = Bincode`) to understand the problem.

This is particularly confusing for wire format mismatches because the most common cause -- mixing debug-built plugins with release-built hosts -- is not mentioned in the error message.

**Evidence**:
```rust
#[error("wire format mismatch: got {got}, expected {expected}")]
WireFormatMismatch { got: u8, expected: u8 },
```

**Impact**: Users encountering the most common configuration error (debug/release mismatch) receive an unhelpful error message.

**Recommendation**: Store `WireFormat` and `BufferStrategyKind` enum values instead of `u8`, and use `Debug` or a custom `Display` impl. The error message should read: `"wire format mismatch: got Json (debug build), expected Bincode (release build)"`. Consider adding a hint: `"Ensure both plugin and host are compiled with the same build profile."`.

---

### OPS-04: `PluginHost::load()` does not report which search paths were checked or which candidates were rejected [Major]

**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/host.rs`, lines 170-221
**Confidence**: High

**Description**: When `load()` fails to find a plugin, it returns `LoadError::PluginNotFound { name }` with only the plugin name. It does not report:
- Which directories were searched
- Which dylib files were found and examined
- Why each candidate was rejected (wrong name, wrong interface, validation failure)

The method also silently skips non-existent search paths and invalid dylibs with `Err(_) => continue`.

**Evidence**:
```rust
Err(LoadError::PluginNotFound {
    name: name.to_string(),
})
```

**Impact**: When a user's plugin is present but fails to load (e.g., architecture mismatch, ABI version mismatch), the error message says "plugin not found" which is actively misleading. The actual problem is hidden by the silent error swallowing.

**Recommendation**: Collect per-candidate diagnostics during the search. If the plugin name was found but validation failed, return the validation error instead of `PluginNotFound`. If no candidates matched the name, include the search paths and candidate count in the error. Example: `"plugin 'Foo' not found: searched 2 directories, examined 5 dylibs, found 3 plugins with different names"`.

---

### OPS-05: CLI has no `--verbose` or `--debug` flag for diagnostic output [Major]

**Location**: `/Users/dstorey/Desktop/fides/fidius-cli/src/main.rs`
**Confidence**: High

**Description**: The CLI provides no mechanism for users to increase output verbosity. All commands produce fixed output with no option to see intermediate steps. For commands that can fail in complex ways (`inspect`, `package build`, `package verify`), there is no way to get additional context about what the CLI is doing.

The `package build` command is the most affected: it shells out to `cargo build` and on failure returns the stderr, but on success provides only `"Build successful. Output in .../target/{profile}/"` without the actual path to the compiled cdylib.

**Impact**: Users troubleshooting CLI issues must resort to strace/dtruss or reading source code.

**Recommendation**: Add a global `--verbose` flag that enables detailed output for each command. For `package build`, also report the actual cdylib path. Consider a `--format json` flag for machine-readable output to support tooling integration.

---

### OPS-06: `verify` command calls `process::exit(1)` instead of returning an error [Minor]

**Location**: `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs`, line 271
**Confidence**: High

**Description**: The `verify` command hard-exits with `std::process::exit(1)` on verification failure, bypassing the normal error handling in `main()`. Every other command returns `Err(...)` which `main()` handles uniformly by printing the error and exiting with code 1. This inconsistency means `verify` cannot be composed with other operations (e.g., in a future `--batch` mode) and bypasses any cleanup or future post-command hooks.

**Evidence**:
```rust
Err(_) => {
    eprintln!("Signature INVALID: {}", dylib_path.display());
    std::process::exit(1);
}
```

**Impact**: Inconsistent error handling. Destructors are not run. If `verify` is called from `package_verify`, the hard exit also applies, making `package verify` inconsistent with other package subcommands.

**Recommendation**: Return `Err("Signature INVALID: ...")` like all other commands.

---

### OPS-07: `InvalidMagic` error provides no file context [Minor]

**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/error.rs`, line 29; `/Users/dstorey/Desktop/fides/fidius-host/src/loader.rs`, line 94
**Confidence**: High

**Description**: When the magic bytes in a loaded library do not match `FIDIUS_MAGIC`, the error `LoadError::InvalidMagic` is returned with no indication of which file was loaded. During discovery (which loads many files), this error is silently swallowed (OPS-02), but if triggered through a direct `load_library()` call, the user cannot tell which file had bad magic.

**Evidence**:
```rust
#[error("invalid magic bytes (expected FIDIUS\\0\\0)")]
InvalidMagic,
```

**Recommendation**: Add a `path: String` field to `InvalidMagic` to identify the file.

---

### OPS-08: `detect_architecture` misclassifies IO errors as `LibraryNotFound` [Minor]

**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/arch.rs`, lines 69-71
**Confidence**: High

**Description**: `std::fs::read(path)` errors are mapped to `LoadError::LibraryNotFound`, discarding the original error. A permission-denied error, a disk I/O error, or an out-of-memory error all appear as "library not found," which is actively misleading for debugging.

**Evidence**:
```rust
let bytes = std::fs::read(path).map_err(|_| LoadError::LibraryNotFound {
    path: path.display().to_string(),
})?;
```

**Impact**: Users with file permission issues receive incorrect error messages, leading them to check the wrong thing (file existence) rather than the actual problem (file permissions).

**Recommendation**: Preserve the IO error kind. Map `NotFound` to `LibraryNotFound`, and use `LoadError::Io(e)` for all other IO errors.

---

### OPS-09: `CallError::Panic` does not preserve the panic message [Minor]

**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/handle.rs`, line 148; `/Users/dstorey/Desktop/fides/fidius-macro/src/impl_macro.rs`, lines 206-209
**Confidence**: High

**Description**: When a plugin method panics, `catch_unwind` catches the panic payload, but the generated shim discards it entirely and returns `STATUS_PANIC`. The host then returns `CallError::Panic` with no message. The panic payload (typically a `&str` or `String`) is available in the `Err` branch of `catch_unwind` but is ignored.

**Evidence** (generated shim code):
```rust
match result {
    Ok(status) => status,
    Err(_) => fidius::status::STATUS_PANIC,
}
```

**Impact**: When a plugin panics, the host application receives `CallError::Panic` with the message `"plugin panicked during method call"` and nothing else. The actual panic message, which would identify the source of the panic, is lost.

**Recommendation**: Extract the panic payload (via `downcast_ref::<&str>()` and `downcast_ref::<String>()`), serialize it into the output buffer with a `STATUS_PANIC` status code, and have the host deserialize it into a `CallError::Panic(String)` variant.

---

### OPS-10: `CallError::NotImplemented` variant exists but is never produced [Minor]

**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/error.rs`, line 84; `/Users/dstorey/Desktop/fides/fidius-host/src/handle.rs`
**Confidence**: High

**Description**: The `CallError::NotImplemented { bit: u32 }` variant exists in the error enum, but no code path in `call_method` or anywhere else produces it. When a host calls an unimplemented optional method, the vtable slot is `None` (null pointer due to nullable pointer optimization), and calling it causes a segfault rather than a clean error.

The `has_capability()` method exists for hosts to check before calling, but there is no enforcement. A host that forgets to check `has_capability()` will get undefined behavior, not a `NotImplemented` error.

**Impact**: The error variant gives the false impression that calling an unimplemented method is handled gracefully. In reality, it causes a crash.

**Recommendation**: Inside `call_method`, read the vtable slot as `Option<FfiFn>` and check for `None` before calling. Return `CallError::NotImplemented` when the slot is null. This requires knowing which methods are optional, which could be tracked via a method count or capability bitfield check.

---

### OPS-11: `fidius inspect` is a useful diagnostic tool (Observation, positive)

**Location**: `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs`, lines 278-298
**Confidence**: High

**Description**: The `fidius inspect` command loads a dylib and displays its registry contents in a clear, human-readable format including plugin count, names, interface hash, version, wire format, buffer strategy, and capabilities. This is the right tool for debugging "is my plugin built correctly?" questions.

**Impact**: Positive. This is a valuable diagnostic tool for plugin authors.

**Recommendation**: Consider adding a `--json` output mode for scripting, and adding the file's detected architecture to the output.

---

### OPS-12: Lenient policy warnings are unstructured and non-configurable (Observation)

**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/host.rs`, line 193
**Confidence**: High

**Description**: When `LoadPolicy::Lenient` encounters a signature failure, it emits `eprintln!("fidius warning: {e}")`. This goes directly to stderr with no structured format, no timestamp, no severity level, and no way for the host application to capture or redirect it. A host embedding fidius in a GUI application or service would have these warnings appear on stderr with no control.

**Evidence**:
```rust
Err(e) if self.load_policy == LoadPolicy::Lenient => {
    eprintln!("fidius warning: {e}");
}
```

**Recommendation**: Replace with a logging call (per OPS-01). If logging is not added, provide a callback mechanism so host applications can handle warnings programmatically.

---

### OPS-13: Async runtime initialization failure panics with a generic message (Observation)

**Location**: `/Users/dstorey/Desktop/fides/fidius-core/src/async_runtime.rs`, line 30
**Confidence**: High

**Description**: If the tokio runtime fails to initialize (e.g., due to resource exhaustion), the `LazyLock` initializer panics with `"failed to create fidius async runtime"`. Since this runs inside a plugin dylib during an FFI call, the panic will be caught by the `catch_unwind` in the generated shim and returned as `CallError::Panic` -- but the descriptive message is lost (per OPS-09). The host will see only "plugin panicked during method call" with no indication that the root cause is runtime initialization failure.

**Recommendation**: If the panic message preservation recommended in OPS-09 is implemented, this becomes diagnosable. Additionally, consider making runtime creation fallible and returning `STATUS_SERIALIZATION_ERROR` or a new status code instead of panicking.
