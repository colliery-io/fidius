# Legibility Review: Fidius Plugin Framework

**Reviewer Lens**: Can a newcomer understand what this does and why?

**Date**: 2026-03-28

---

## Summary

The Fidius codebase is remarkably legible for an FFI-heavy proc-macro framework. Module organization follows the plugin lifecycle cleanly (core types, macro codegen, host loading, CLI tooling, facade). Doc comments are present on nearly every public item and consistently explain *why*, not just *what*. The code favors clarity over cleverness, with the notable exception of the vtable pointer arithmetic in `PluginHandle::call_method` and the const-eval string comparison loop in `impl_macro.rs`.

The primary legibility barriers are: (1) the repository name / crate name mismatch (`fides` vs `fidius`), (2) the generated companion module naming convention (`__fidius_TraitName`) which is unfamiliar until you read the macro source, (3) the implicit requirement that plugin types be unit structs, and (4) the `call_method` API using raw integer indices with no compile-time safety or symbolic names. Overall, the framework is well-structured for its complexity level, but several areas of implicit knowledge would trip up newcomers.

---

## Key Themes

1. **Strong module boundaries with clear responsibilities** -- Each crate has a well-defined role, and cross-crate dependencies flow in one direction. The facade crate is a clean abstraction.

2. **Good doc comments, sparse inline comments** -- Public API documentation is thorough, but the most complex code paths (generated shims, vtable casting) have minimal inline explanation of the *reasoning* behind design choices.

3. **Naming is mostly excellent, with a few surprising conventions** -- Generated symbol names follow a consistent pattern but require knowledge of the double-underscore convention. The `fides`/`fidius` split is the single biggest naming confusion.

4. **Implicit contracts enforced by codegen, not by the type system** -- Several safety-critical assumptions (unit struct requirement, vtable index bounds, method ordering) are enforced only by the macro-generated code failing to compile in non-obvious ways, rather than by types or clear error messages.

5. **Code complexity is concentrated in two files** -- `impl_macro.rs` (320 lines) and `interface.rs` (245 lines) contain the bulk of the framework's logic. Both are well-organized into small functions, but the generated token streams require reading proc-macro output mentally.

---

## Findings

## LEG-01: Repository name `fides` vs crate prefix `fidius`
**Severity**: Major
**Location**: Repository root, all `Cargo.toml` files
**Confidence**: High
### Description
The repository directory is named `fides` but every crate uses the `fidius-` prefix. The GitHub URL in Cargo.toml metadata uses `colliery-io/fidius`. No documentation explains this discrepancy. A newcomer cloning the repository will immediately wonder whether these are two different projects, a rename in progress, or a deliberate choice.
### Evidence
- Repository root: `/Users/dstorey/Desktop/fides`
- All crates: `fidius`, `fidius-core`, `fidius-macro`, `fidius-host`, `fidius-cli`
- System overview (line 6): "The repository name on disk is `fides` but all crate names use the `fidius` prefix."
### Suggested Resolution
Either rename the repository directory to `fidius` to match crate names, or add a note in a top-level README explaining the naming. If `fides` is a deliberate name (e.g., Latin for "trust/faith"), document that connection.

---

## LEG-02: `call_method` uses raw integer indices with no symbolic names
**Severity**: Major
**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/handle.rs`, lines 93-167
**Confidence**: High
### Description
The primary method-calling API requires callers to pass a raw `usize` index into the vtable. There are no symbolic constants, no index enum, and no bounds checking. A newcomer must manually count method declarations in the trait to determine the correct index. An off-by-one error silently reads an invalid function pointer.
### Evidence
```rust
pub fn call_method<I: Serialize, O: DeserializeOwned>(
    &self,
    index: usize,
    input: &I,
) -> Result<O, CallError> {
    // ...
    let fn_ptr = unsafe {
        let fn_ptrs = self.vtable as *const FfiFn;
        *fn_ptrs.add(index)
    };
```
Tests confirm the usage pattern: `handle.call_method::<AddInput, AddOutput>(0, &input)` -- the `0` is entirely opaque.
### Suggested Resolution
Generate index constants in the companion module (e.g., `Greeter_METHOD_GREET: usize = 0`) so callers can write `call_method::<_, _>(Greeter_METHOD_GREET, &input)`. Alternatively, generate a typed wrapper that exposes named methods. At minimum, add runtime bounds checking against the vtable size.

---

## LEG-03: Unsafe vtable pointer arithmetic assumes flat function pointer layout
**Severity**: Major
**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/handle.rs`, lines 103-106
**Confidence**: High
### Description
The `call_method` implementation casts the vtable to `*const FfiFn` and offsets by index, treating the vtable as a flat array of bare function pointers. However, optional methods are `Option<FfiFn>` in the generated vtable struct. This works only because Rust guarantees the nullable pointer optimization for `Option<fn>`, making it the same size as a bare function pointer. This guarantee is real but subtle, and no comment explains it.
### Evidence
```rust
let fn_ptr = unsafe {
    let fn_ptrs = self.vtable as *const FfiFn;
    *fn_ptrs.add(index)
};
```
The vtable struct generated by `interface.rs` has mixed types:
```rust
pub struct Greeter_VTable {
    pub greet: unsafe extern "C" fn(...) -> i32,          // bare fn
    pub greet_fancy: Option<unsafe extern "C" fn(...) -> i32>, // Option<fn>
}
```
### Suggested Resolution
Add a comment explaining the nullable pointer optimization assumption. Consider adding a `static_assert` (via `const _: ()`) that `size_of::<Option<FfiFn>>() == size_of::<FfiFn>()` in the generated code so any future Rust compiler change would fail the build rather than silently produce UB.

---

## LEG-04: Generated companion module naming convention (`__fidius_TraitName`) is undocumented in user-facing docs
**Severity**: Minor
**Location**: `/Users/dstorey/Desktop/fides/fidius-macro/src/interface.rs`, line 61
**Confidence**: High
### Description
The `#[plugin_interface]` macro generates a public companion module named `__fidius_{TraitName}` containing the vtable type, hash constants, and capability constants. Plugin authors must import this module (the scaffolded code does `use interface_mod::{..., __fidius_TraitName}`), but the naming convention is only explained in the system overview, not in the macro's doc comment or in the facade crate's documentation.
### Evidence
Scaffolded plugin code in `commands.rs` line 171:
```rust
use {interface_mod}::{{plugin_impl, {trait_name}, PluginError, __fidius_{trait_name}}};
```
The macro doc comment (`fidius-macro/src/lib.rs` lines 26-39) does not mention the companion module at all.
### Suggested Resolution
Add a "Generated Items" section to the `#[plugin_interface]` doc comment listing the companion module and its key contents. Consider whether the double-underscore prefix is necessary -- it signals "internal" but the module is imported directly by plugin authors.

---

## LEG-05: Unit struct requirement is implicit and produces confusing errors
**Severity**: Major
**Location**: `/Users/dstorey/Desktop/fides/fidius-macro/src/impl_macro.rs`, lines 96-98
**Confidence**: High
### Description
The `#[plugin_impl]` macro generates `static INSTANCE: Type = Type;`, which only compiles when `Type` is a unit struct. If someone writes `struct MyPlugin { config: String }` and applies `#[plugin_impl]`, they get a cryptic compiler error about not being able to construct the type, with no mention of the fidius constraint. This is the most likely newcomer mistake.
### Evidence
```rust
let instance = quote! {
    static #instance_name: #impl_type = #impl_type;
};
```
The system overview flags this (open question 6): "Non-unit structs will produce a compile error, but the error message will not explain the fidius constraint."
### Suggested Resolution
Either: (a) Add a compile-time check in the macro that verifies the struct is a unit struct and emits a clear error ("fidius plugins must be unit structs -- methods must take `&self` and the plugin is instantiated as a static singleton"), or (b) document this prominently in the `#[plugin_impl]` doc comment.

---

## LEG-06: Wire format debug/release coupling is non-obvious
**Severity**: Minor
**Location**: `/Users/dstorey/Desktop/fides/fidius-core/src/wire.rs`, lines 28-33
**Confidence**: High
### Description
The wire format (JSON vs bincode) is determined by `cfg(debug_assertions)`, meaning debug-built plugins are incompatible with release-built hosts. While the system detects and rejects mismatches at load time, the coupling is surprising. A newcomer building a plugin in debug mode and a host in release mode will get `WireFormatMismatch` with no explanation of *why* the formats differ.
### Evidence
```rust
#[cfg(debug_assertions)]
pub const WIRE_FORMAT: WireFormat = WireFormat::Json;

#[cfg(not(debug_assertions))]
pub const WIRE_FORMAT: WireFormat = WireFormat::Bincode;
```
The `WireFormatMismatch` error message shows raw `u8` values (`got 0, expected 1`) rather than human-readable format names.
### Suggested Resolution
(1) Change the `WireFormatMismatch` error to display format names (`"got Json, expected Bincode"`) instead of raw u8 values. (2) Add a doc comment to `WIRE_FORMAT` explicitly stating "this means debug-built plugins are incompatible with release-built hosts." (3) Consider adding the build profile to the error message.

---

## LEG-07: `PluginHost` and `PluginHostBuilder` have nearly identical fields
**Severity**: Minor
**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/host.rs`, lines 28-47
**Confidence**: High
### Description
`PluginHost` and `PluginHostBuilder` declare identical field lists. The builder's `build()` method just moves fields from one struct to the other with no validation or transformation. This is unnecessary duplication that a newcomer might read as significant.
### Evidence
Both structs have: `search_paths`, `load_policy`, `require_signature`, `trusted_keys`, `expected_hash`, `expected_wire`, `expected_strategy` -- identical types. `build()` (line 105) simply copies them:
```rust
pub fn build(self) -> Result<PluginHost, LoadError> {
    Ok(PluginHost {
        search_paths: self.search_paths,
        // ... identical fields ...
    })
}
```
### Suggested Resolution
Consider using the builder pattern with `PluginHost` directly (builder methods on `PluginHost` itself), or add a comment explaining that the separate builder exists for future validation logic. Alternatively, a `derive_builder` approach would reduce the duplication.

---

## LEG-08: `detect_architecture` reads entire file into memory
**Severity**: Minor
**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/arch.rs`, line 69
**Confidence**: High
### Description
The architecture detection reads the entire dylib (potentially hundreds of MB) into memory to inspect the first 20 bytes of the header. A newcomer reading this code would immediately question the efficiency.
### Evidence
```rust
let bytes = std::fs::read(path).map_err(|_| LoadError::LibraryNotFound {
    path: path.display().to_string(),
})?;
```
Only `bytes[0..20]` are ever accessed in the function body.
### Suggested Resolution
Use `std::fs::File::open` + `Read::read_exact` with a small fixed buffer (e.g., 20 bytes). This also avoids mapping I/O errors to `LibraryNotFound` for non-missing-file failures (e.g., permission denied).

---

## LEG-09: Const-eval string comparison loop in `generate_descriptor` is hard to follow
**Severity**: Minor
**Location**: `/Users/dstorey/Desktop/fides/fidius-macro/src/impl_macro.rs`, lines 269-297
**Confidence**: High
### Description
The capability bitfield computation uses a nested `while` loop with byte-level string comparison to work within `const` evaluation context. While technically necessary (no `==` on `&str` in const context), the code is dense and lacks a comment explaining why this approach is needed.
### Evidence
```rust
const CAPS: u64 = {
    let optional = #companion::#optional_methods_ident;
    let impl_methods: &[&str] = &[#(#method_strs),*];
    let mut caps: u64 = 0;
    let mut opt_idx = 0;
    while opt_idx < optional.len() {
        let opt_name = optional[opt_idx];
        let mut impl_idx = 0;
        while impl_idx < impl_methods.len() {
            let impl_name = impl_methods[impl_idx];
            if opt_name.len() == impl_name.len() {
                let ob = opt_name.as_bytes();
                let ib = impl_name.as_bytes();
                let mut j = 0;
                let mut eq = true;
                while j < ob.len() {
                    if ob[j] != ib[j] { eq = false; }
                    j += 1;
                }
                if eq { caps |= 1u64 << opt_idx; }
            }
            impl_idx += 1;
        }
        opt_idx += 1;
    }
    caps
};
```
### Suggested Resolution
Add a comment: `// Manual string comparison because str::eq is not const-stable. This computes the capability bitfield at compile time by checking which optional methods are implemented.` Consider extracting a `const fn str_eq(a: &str, b: &str) -> bool` helper to reduce nesting.

---

## LEG-10: Hardcoded `fidius-core = { version = "0.1" }` in scaffolded plugin
**Severity**: Minor
**Location**: `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs`, line 164
**Confidence**: High
### Description
The `init-plugin` command generates a `Cargo.toml` with `fidius-core = { version = "0.1" }` regardless of the actual version (currently `0.0.0-alpha.1`). This will fail to resolve when used with crates.io and confuses newcomers who expect the scaffolded code to work immediately.
### Evidence
```rust
let cargo_toml = format!(
    r#"...
fidius-core = {{ version = "0.1" }}
"#
);
```
### Suggested Resolution
Use `env!("CARGO_PKG_VERSION")` or the resolved interface version to generate a correct dependency version. Alternatively, resolve `fidius-core` through the same `resolve_dep` path used for the interface crate.

---

## LEG-11: Duplicated `.sig` path construction logic
**Severity**: Minor
**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/signing.rs` lines 36-42, `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs` lines 224-230 and 249-255
**Confidence**: High
### Description
The logic to construct a `.sig` file path from a dylib path is duplicated three times across two crates with identical code. A newcomer modifying the signing logic must find and update all three locations.
### Evidence
All three instances:
```rust
let sig_path = dylib_path.with_extension(format!(
    "{}.sig",
    dylib_path.extension().and_then(|e| e.to_str()).unwrap_or("")
));
```
### Suggested Resolution
Extract a `sig_path_for(dylib_path: &Path) -> PathBuf` function into `fidius-core` or `fidius-host` and reuse it from the CLI.

---

## LEG-12: `LoadError::WireFormatMismatch` and `BufferStrategyMismatch` display raw u8 values
**Severity**: Minor
**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/error.rs`, lines 40-44
**Confidence**: High
### Description
When a wire format or buffer strategy mismatch occurs, the error message displays raw `u8` discriminant values rather than human-readable enum names. A newcomer seeing `"wire format mismatch: got 0, expected 1"` must look up what `0` and `1` mean.
### Evidence
```rust
#[error("wire format mismatch: got {got}, expected {expected}")]
WireFormatMismatch { got: u8, expected: u8 },

#[error("buffer strategy mismatch: got {got}, expected {expected}")]
BufferStrategyMismatch { got: u8, expected: u8 },
```
### Suggested Resolution
Store `WireFormat` and `BufferStrategyKind` enum values instead of `u8`, and derive `Display` for both enums (or use the existing `Debug` implementation). The error messages would then read: `"wire format mismatch: got Json, expected Bincode"`.

---

## LEG-13: `_has_async` variable computed but unused
**Severity**: Observation
**Location**: `/Users/dstorey/Desktop/fides/fidius-macro/src/impl_macro.rs`, line 89
**Confidence**: High
### Description
The variable `_has_async` is computed from impl methods but never used. The leading underscore suppresses the warning, but suggests unfinished logic or a removed feature. A newcomer may wonder if async support is incomplete.
### Evidence
```rust
let _has_async = impl_methods.iter().any(|m| m.is_async);
```
### Suggested Resolution
Either use the variable (e.g., for conditional imports of the async runtime) or remove it with a comment explaining that async detection happens per-shim rather than per-impl.

---

## LEG-14: `check_crates_io` User-Agent references wrong GitHub org
**Severity**: Observation
**Location**: `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs`, line 60
**Confidence**: High
### Description
The crates.io API request uses User-Agent `"fidius-cli (https://github.com/fidius-rs/fidius)"` but the actual repository is at `github.com/colliery-io/fidius`. A newcomer following the link would land on a 404.
### Evidence
```rust
.header("User-Agent", "fidius-cli (https://github.com/fidius-rs/fidius)")
```
### Suggested Resolution
Update the User-Agent to reference the correct repository URL.

---

## LEG-15: `build_package` returns target directory when cdylib not found
**Severity**: Observation
**Location**: `/Users/dstorey/Desktop/fides/fidius-host/src/package.rs`, lines 119-121
**Confidence**: Medium
### Description
When `build_package` cannot find the compiled cdylib in the target directory, it silently returns the target directory path instead of an error. A newcomer would expect this function to either return the cdylib path or fail, not return a directory.
### Evidence
```rust
// Return the target dir even if we can't find the specific dylib
Ok(target_dir)
```
### Suggested Resolution
Return an error variant (e.g., `PackageError::CdylibNotFound`) when the expected output file cannot be located. The comment acknowledges the awkwardness.

---

## LEG-16: `fidius_core::lib.rs` uses wildcard re-exports that obscure the public API
**Severity**: Minor
**Location**: `/Users/dstorey/Desktop/fides/fidius-core/src/lib.rs`, lines 26-28
**Confidence**: Medium
### Description
The `fidius-core` lib.rs re-exports `descriptor::*` and `status::*` at the crate root, flattening these modules' contents into the top-level namespace. This makes it harder for a newcomer to discover which module a type originates from when reading code that uses `fidius_core::PluginRegistry` vs `fidius_core::descriptor::PluginRegistry`.
### Evidence
```rust
pub use descriptor::*;
pub use error::PluginError;
pub use status::*;
```
### Suggested Resolution
Consider explicit re-exports (`pub use descriptor::{PluginRegistry, PluginDescriptor, ...}`) or document the design intent. The selective `pub use error::PluginError` shows the pattern is already partially applied.

---

## LEG-17: `package_build` in CLI duplicates `build_package` logic from `fidius-host`
**Severity**: Observation
**Location**: `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs`, lines 327-358 vs `/Users/dstorey/Desktop/fides/fidius-host/src/package.rs`, lines 75-121
**Confidence**: High
### Description
The CLI's `package_build` command reimplements the cargo build invocation rather than calling `fidius_host::package::build_package`. Both construct the same `cargo build --manifest-path` command. A newcomer fixing a build bug must update both locations.
### Evidence
CLI (`commands.rs:339`):
```rust
let mut cmd = std::process::Command::new("cargo");
cmd.arg("build").arg("--manifest-path").arg(&cargo_toml);
```
Host (`package.rs:84`):
```rust
let mut cmd = std::process::Command::new("cargo");
cmd.arg("build").arg("--manifest-path").arg(&cargo_toml);
```
### Suggested Resolution
Have the CLI delegate to `fidius_host::package::build_package` instead of reimplementing the logic.

---

## LEG-18: Facade crate doc examples show good usage patterns
**Severity**: Observation (positive)
**Location**: `/Users/dstorey/Desktop/fides/fidius/src/lib.rs`, lines 15-46
**Confidence**: High
### Description
The facade crate's top-level doc comment provides clear, realistic examples for both interface authors and plugin authors. The examples show the complete workflow including imports, trait definition, implementation, and registry macro call. This is exactly what a newcomer needs.
### Evidence
The doc comment at lines 15-46 includes two complete examples with `ignore` annotations (appropriate since they depend on generated code), covering the two primary user roles.
### Suggested Resolution
None needed -- this is a positive finding. Consider adding a third example for the host side.
