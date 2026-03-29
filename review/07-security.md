# Security Review: Fidius Plugin Framework

**Reviewer lens**: Is this safe against misuse, abuse, and attack?
**Finding prefix**: SEC-
**Date**: 2026-03-28

---

## Summary

Fidius is a plugin framework that loads and executes arbitrary native code (cdylibs). Its security model is trust-based, not isolation-based: once a plugin is loaded, it runs in the host process with full privileges. This is a legitimate design choice for a framework targeting cooperative plugin ecosystems. The review evaluates whether the trust model is correctly implemented, whether the boundaries are clear, and whether the mechanisms for establishing trust (signing, validation) are sound.

The framework's Ed25519 signing model is cryptographically sound but operationally incomplete. Key generation uses `OsRng` (correct), signature verification uses `ed25519-dalek` (well-audited), and the scheme is sign-then-verify over the raw dylib bytes (straightforward). However, several gaps undermine the trust model in practice: secret keys are written to disk with no file permission restrictions, `discover()` bypasses signature verification entirely, `LoadPolicy::Lenient` allows loading plugins that fail signature checks (defeating the purpose), and the `inspect` command executes `dlopen` on arbitrary dylibs without any signature check.

The FFI boundary presents the highest-impact attack surface. A malicious or corrupted plugin can cause undefined behavior in the host through several mechanisms: unchecked vtable indices, null pointer dereference on the STATUS_OK path, incorrect `free_buffer` capacity, and panicking descriptor conversion functions. The host has no sandboxing, no resource limits, and no capability restrictions on loaded plugins.

**Finding count**: 3 Critical, 5 Major, 5 Minor, 4 Observations

---

## Trust Boundary Map

```
                        UNTRUSTED                           TRUSTED
                   ┌─────────────────┐                ┌──────────────────┐
                   │                 │                │                  │
                   │  Plugin dylib   │──dlopen───────▶│  Host process    │
                   │  (cdylib file)  │                │  memory space    │
                   │                 │                │                  │
                   └────────┬────────┘                └──────────────────┘
                            │                                  ▲
                   ┌────────┴────────┐                         │
                   │  .sig file      │──verify────────────────┘
                   │  (detached sig) │
                   └─────────────────┘

  TRUST BOUNDARY 1: File system → dlopen
    Gatekeepers: check_architecture, verify_signature, magic/version checks
    Gap: inspect command bypasses signature check
    Gap: discover() bypasses signature check
    Gap: Lenient policy proceeds after signature failure

  TRUST BOUNDARY 2: FFI call boundary (host ↔ plugin)
    Gatekeepers: catch_unwind (plugin side), status code checking (host side)
    Gap: No vtable bounds checking
    Gap: No null-pointer check on output buffer
    Gap: No input length validation

  TRUST BOUNDARY 3: CLI → file system
    Gatekeepers: clap argument parsing
    Gap: Secret key files written with default permissions
    Gap: package build shells out to cargo with user-controlled paths

  TRUST BOUNDARY 4: CLI → network
    Gatekeepers: None
    Gap: check_crates_io makes unauthenticated HTTP requests
```

---

## Threat Model Observations

### In-scope threats (trust-based model)

1. **Tampered plugin binary** -- An attacker modifies a signed dylib after signing. The Ed25519 signature detects this. **Mitigated** (when signatures are required and policy is Strict).

2. **Unauthorized plugin** -- A plugin from an untrusted source attempts to load. The trusted-keys model requires the host to configure known public keys. **Mitigated** (when signatures are required).

3. **ABI drift** -- Plugin built against a different interface version. Interface hash, wire format, and buffer strategy checks catch this. **Mitigated**.

4. **Plugin panic crossing FFI** -- `catch_unwind` in generated shims prevents unwinding across `extern "C"`. **Mitigated**.

### Out-of-scope but worth noting

5. **Malicious plugin executing arbitrary code** -- By design, loaded plugins execute native code in the host process. No sandboxing exists. This is the accepted risk of a cdylib plugin framework.

6. **Supply chain attack on plugin dependencies** -- Plugins compile arbitrary Rust code. No dependency auditing, SBOM, or reproducible build mechanism exists. This is outside the framework's scope but relevant to the ecosystem.

---

## Findings

### SEC-01: Secret key files written with no permission restrictions (Major)

**File**: `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs`, lines 204-205
**Confidence**: High

#### Description

The `keygen` command writes the Ed25519 secret key to disk using `std::fs::write`, which creates the file with default permissions (typically `0644` on Unix, readable by all users). The 32-byte raw secret key is the complete signing authority for all plugins signed with this key.

```rust
std::fs::write(&secret_path, signing_key.to_bytes())?;
std::fs::write(&public_path, verifying_key.to_bytes())?;
```

#### Impact

Any user or process on the same system can read the secret key file and forge plugin signatures. On shared development machines or CI systems, this is a direct compromise of the signing model.

#### Recommendation

Set file permissions to `0600` (owner-only read/write) on Unix systems immediately after creation. Consider using a platform-appropriate secure storage mechanism. At minimum, emit a warning if the file is world-readable.

```rust
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&secret_path, std::fs::Permissions::from_mode(0o600))?;
}
```

---

### SEC-02: `discover()` bypasses signature verification (Major)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/host.rs`, lines 128-167
**Confidence**: High

#### Description

`PluginHost::discover()` calls `load_library()` on every dylib in the search paths, which executes `dlopen` and calls `fidius_get_registry()` -- running code inside the plugin. It does this regardless of whether `require_signature` is set. Only `load()` checks signatures.

`dlopen` itself executes constructor functions (`__attribute__((constructor))` in C, `ctor` crate in Rust). A malicious dylib can execute arbitrary code the moment it is opened, before any registry validation occurs.

#### Impact

A host that calls `discover()` to list available plugins will execute code in every dylib it finds, even unsigned or tampered ones. An attacker who places a malicious `.dylib` in a search path directory achieves code execution when the host merely scans for plugins.

#### Recommendation

Apply signature verification before `dlopen` in both `discover()` and `load()`. This requires reading and verifying the signature before calling `load_library()`. Consider adding a `scan()` method that lists files without opening them.

---

### SEC-03: `LoadPolicy::Lenient` defeats signature enforcement (Major)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/host.rs`, lines 189-197
**Confidence**: High

#### Description

When `require_signature` is true and `load_policy` is `Lenient`, signature verification failures are reduced to an `eprintln!` warning, and the plugin is loaded anyway.

```rust
Err(e) if self.load_policy == LoadPolicy::Lenient => {
    eprintln!("fidius warning: {e}");
}
```

#### Impact

An attacker who can place a modified dylib in a search path can have it loaded by any host using `Lenient` policy, regardless of signature requirements. The warning goes to stderr where it is easily missed or suppressed. The combination of `require_signature = true` with `Lenient` policy creates a false sense of security.

#### Recommendation

Either: (a) make `Lenient` policy skip signature verification entirely (honestly insecure), or (b) make signature failures always fatal when `require_signature` is true, regardless of load policy. The current behavior -- requiring signatures but ignoring failures -- is the worst option because it suggests security is enforced when it is not. At minimum, document this interaction prominently.

---

### SEC-04: Unchecked vtable index enables host-side code execution from malicious plugin data (Critical)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/handle.rs`, lines 103-106
**Confidence**: High

#### Description

`call_method` reads a function pointer from the vtable using raw pointer arithmetic with no bounds check. While a correct host should only use valid indices, the vtable pointer comes from the loaded plugin. A crafted plugin could provide a vtable pointer that, when indexed, causes the host to call an arbitrary address.

```rust
let fn_ptr = unsafe {
    let fn_ptrs = self.vtable as *const FfiFn;
    *fn_ptrs.add(index)
};
```

The `PluginDescriptor` has no `method_count` field, so there is no way to validate the index at runtime. The vtable is an opaque `*const c_void` -- the host has no knowledge of the vtable's actual size.

#### Impact

Undefined behavior. In the worst case, an attacker-controlled vtable pointer leads the host to execute arbitrary code at an attacker-chosen address (a classic vtable hijack). Even without malicious intent, an incorrect index causes a segfault or data corruption.

#### Recommendation

Add a `method_count: u32` field to `PluginDescriptor` and validate `index < method_count` before the unsafe read. This is the single most impactful safety improvement available.

---

### SEC-05: `inspect` command executes `dlopen` on untrusted dylibs without signature check (Major)

**File**: `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs`, lines 278-298
**Confidence**: High

#### Description

The `fidius inspect <dylib>` command calls `load_library()` directly, which performs `dlopen` on the provided path with no signature verification. This command is intended for developers to examine plugin metadata, but it executes code in the inspected dylib.

#### Impact

A developer running `fidius inspect malicious.dylib` executes the dylib's constructor code. This is a social engineering vector: "run `fidius inspect` on this plugin to check compatibility" can achieve arbitrary code execution.

#### Recommendation

Add a `--no-verify` flag to `inspect` and require either a signature check or an explicit opt-out. Alternatively, add a warning: "WARNING: inspecting a dylib loads and executes code from it." Consider whether inspection can be done by reading the file without dlopen (e.g., parsing the binary to find the registry without executing it).

---

### SEC-06: No null-pointer check on output buffer allows host crash from malicious plugin (Critical)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/handle.rs`, lines 157
**Confidence**: High

#### Description

On the `STATUS_OK` path, the host unconditionally creates a slice from `out_ptr` without checking for null.

```rust
let output_slice = unsafe { std::slice::from_raw_parts(out_ptr, out_len as usize) };
```

A malicious or buggy plugin that returns `STATUS_OK` without setting `out_ptr` (leaving it as the initial `null_mut()`) causes undefined behavior -- creating a slice from a null pointer. While generated shims always set `out_ptr`, a hand-crafted or corrupted vtable could trigger this.

#### Impact

Undefined behavior in the host process. On most platforms this is a segfault, but the Rust compiler may optimize based on the assumption that slices are never null, potentially causing unexpected behavior.

#### Recommendation

Add a null check before creating the slice:
```rust
if out_ptr.is_null() {
    return Err(CallError::Serialization("plugin returned null output buffer".into()));
}
```

---

### SEC-07: Descriptor field parsing panics on unknown values, enabling DoS from malformed plugins (Major)

**File**: `/Users/dstorey/Desktop/fides/fidius-core/src/descriptor.rs`, lines 174-190
**Confidence**: High

#### Description

`buffer_strategy_kind()` and `wire_format_kind()` call `panic!` on unknown discriminant values. These methods are called in `validate_descriptor()` during plugin loading. A plugin with a corrupted or future-version `wire_format` or `buffer_strategy` byte crashes the host process.

```rust
_ => panic!("invalid buffer_strategy value: {}", self.buffer_strategy),
```

#### Impact

Denial of service. A single malformed dylib in a search path can crash any host that attempts to load or discover plugins. The attacker only needs to place a file with valid magic bytes but an unknown wire format value.

#### Recommendation

Return `Result` from these methods and propagate the error as a `LoadError` variant. Never panic on data read from untrusted sources.

---

### SEC-08: `free_buffer` uses incorrect capacity, enabling heap corruption (Critical)

**File**: `/Users/dstorey/Desktop/fides/fidius-macro/src/impl_macro.rs`, lines 106-110
**Confidence**: High

#### Description

The generated `free_buffer` function reconstructs a `Vec` with `capacity == len`:
```rust
drop(unsafe { Vec::from_raw_parts(ptr, len, len) });
```

The allocation side uses `std::mem::forget(output_bytes)` where `output_bytes` is a `Vec<u8>` that may have `capacity > len` due to the allocator's growth strategy. Reconstructing the `Vec` with the wrong capacity is undefined behavior -- the allocator may attempt to deallocate a memory block of a different size than was originally allocated.

#### Impact

Heap metadata corruption. While many allocators tolerate slight capacity mismatches in practice, this is unsound. Under jemalloc, ASan, or other strict allocators, this can cause crashes, heap corruption, or exploitable conditions. This affects every plugin method call.

#### Recommendation

On the plugin side, call `output_bytes.shrink_to_fit()` before `std::mem::forget` to ensure `capacity == len`. This is a one-line fix in the code generator:
```rust
output_bytes.shrink_to_fit();
let len = output_bytes.len();
let ptr = output_bytes.as_ptr() as *mut u8;
std::mem::forget(output_bytes);
```

---

### SEC-09: Signing model signs dylib content only, not metadata (Minor)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/signing.rs`, lines 60-65
**Confidence**: High

#### Description

The signature covers only the raw dylib bytes. It does not cover the `package.toml` manifest, the plugin name, interface name, or any other metadata. A separate `package sign` command signs `package.toml` independently, but this is not integrated into the host loading path.

An attacker could take a legitimately signed dylib and pair it with a different `package.toml`, changing the declared interface name, version, or metadata without invalidating the dylib signature.

#### Impact

Metadata spoofing. A signed dylib claiming to implement interface "SafeFilter" could be paired with a manifest claiming it implements "AdminPlugin". The dylib signature verifies, but the metadata is untrustworthy.

#### Recommendation

Consider signing a bundle that includes both the dylib and its metadata, or include a content hash of the manifest in the signature. Alternatively, document that the dylib signature only attests to binary integrity, not metadata claims.

---

### SEC-10: `package build` executes `cargo` with user-controlled manifest path (Minor)

**File**: `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs`, lines 339-341
**Confidence**: Medium

#### Description

The `package build` command constructs a `cargo build --manifest-path <user-provided-path>` command. The path comes directly from CLI arguments with no sanitization. While `cargo` itself is trusted, a malicious `Cargo.toml` at the provided path could contain build scripts that execute arbitrary code.

```rust
let mut cmd = std::process::Command::new("cargo");
cmd.arg("build").arg("--manifest-path").arg(&cargo_toml);
```

#### Impact

Low -- this is a developer tool, and running `cargo build` on untrusted code is already dangerous. However, the framework does not warn that `package build` executes arbitrary build scripts.

#### Recommendation

Document that `package build` executes build scripts from the target crate. This is inherent to Cargo and not something the framework can prevent, but users should be aware.

---

### SEC-11: Ed25519 key files use raw 32-byte format with no type indicator (Minor)

**File**: `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs`, lines 196-205, 216-220, 240-245
**Confidence**: High

#### Description

Key files are raw 32-byte blobs with no header, magic bytes, or type indicator. The only distinction between a secret key file and a public key file is the file extension (`.secret` vs `.public`). Both are exactly 32 bytes.

If a user accidentally passes their secret key file as the `--key` argument to `verify` (where a public key is expected), the `VerifyingKey::from_bytes` call will either succeed (interpreting the secret key bytes as a public key point, likely failing) or produce a confusing error.

More critically, if a user passes their secret key to a command that publishes or shares it expecting a public key, the secret key is exposed.

#### Impact

User error leading to key confusion or accidental secret key exposure. The framework cannot prevent all such mistakes, but the lack of any type indicator increases the risk.

#### Recommendation

Add a magic byte prefix to key files (e.g., `FDSEC\0` for secret, `FDPUB\0` for public) so misuse produces a clear error message rather than cryptic verification failure.

---

### SEC-12: `check_crates_io` makes unauthenticated HTTP request with no TLS certificate pinning (Minor)

**File**: `/Users/dstorey/Desktop/fides/fidius-cli/src/commands.rs`, lines 55-70
**Confidence**: Medium

#### Description

The `resolve_dep` function calls `check_crates_io` which makes an HTTPS GET request to `crates.io` using `ureq`. The response determines what dependency version is written into a scaffolded `Cargo.toml`. There is no response validation beyond JSON parsing.

A man-in-the-middle attacker on the network could return a crafted response pointing to a specific malicious version. However, `ureq` uses platform TLS, so this requires compromising the TLS connection.

#### Impact

Low. The scaffolded code still needs to be compiled via `cargo`, which performs its own registry verification. This is defense-in-depth.

#### Recommendation

No immediate action required. The risk is low because Cargo's own registry verification is the actual trust boundary.

---

### SEC-13: No revocation mechanism for signing keys (Minor)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/signing.rs`, `/Users/dstorey/Desktop/fides/fidius-host/src/host.rs`
**Confidence**: High

#### Description

The trusted-keys model in `PluginHost` is a simple list of `VerifyingKey` values. There is no mechanism for key revocation, expiration, or rotation. If a signing key is compromised, the only remediation is to rebuild the host with the compromised key removed from the trusted list and re-sign all plugins with a new key.

#### Impact

Operational security gap. Key compromise requires a coordinated flag-day update of all hosts and re-signing of all plugins. There is no way to express "trust this key only for plugins signed before date X."

#### Recommendation

This is acceptable for an alpha framework where the trust model is intentionally simple. Document the limitation. For production use, consider adding: key expiration (timestamp in the signature or a signed manifest), a revocation list, or a certificate chain model.

---

### SEC-14: Signature verification and dlopen are not atomic (Observation)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/host.rs`, lines 189-199
**Confidence**: High

#### Description

In `load()`, signature verification and `load_library()` (which calls `dlopen`) are separate operations. There is a TOCTOU (time-of-check-to-time-of-use) window between verifying the signature and opening the library. An attacker with write access to the dylib file could replace it between verification and loading.

#### Impact

Narrow race window. Requires the attacker to have write access to the plugin directory and precise timing. In practice, if an attacker has write access to the plugin directory, they can also replace the `.sig` file, so this is a secondary concern.

#### Recommendation

Document the limitation. A robust fix would be to read the file once, verify the signature on the in-memory bytes, and then write to a temporary file for dlopen (or use `memfd_create` on Linux). This is a hardening measure, not a critical fix.

---

### SEC-15: `dlopen` executes constructor code before any validation (Observation)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/loader.rs`, line 72
**Confidence**: High

#### Description

`Library::new(path)` calls the OS dynamic linker, which executes any constructor functions in the dylib before returning control to the host. All validation (magic bytes, version, hash) happens after `dlopen`. A malicious dylib can execute arbitrary code in its constructor, before the host has any chance to reject it.

This is inherent to the design of dynamic linking on all major platforms. It is not a bug in fidius, but it is an important part of the threat model that should be documented.

#### Impact

Fundamental limitation. Any dylib that passes architecture checks (or is placed in a path where architecture checks succeed) achieves code execution at `dlopen` time, regardless of registry validation or signing (unless signatures are checked before dlopen, per SEC-02).

#### Recommendation

Ensure signature verification happens before `dlopen` (see SEC-02). Document that the architecture check is the only pre-dlopen gate, and that it is based on reading file header bytes rather than executing code. Consider adding to user documentation: "Only load plugins from trusted sources. Signature verification must be enabled to prevent loading of tampered plugins."

---

### SEC-16: `Send + Sync` impl on `PluginHandle` relies on undocumented invariants (Observation)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/handle.rs`, lines 52-54
**Confidence**: Medium

#### Description

`PluginHandle` contains raw pointers (`vtable: *const c_void`, `free_buffer: Option<fn(...)>`) and manually implements `Send` and `Sync`. The safety comment states "vtable and free_buffer point to static code/data in the loaded library" and "All access is read-only."

However, calling through the vtable invokes the plugin method, which calls the user's implementation. If the user's method has internal state (e.g., via thread-local storage, global mutable state in the dylib), concurrent calls from multiple threads could cause data races. The framework enforces `&self` (not `&mut self`), but `&self` does not prevent interior mutability (e.g., `RefCell`, `Mutex`, global statics).

#### Impact

Low in practice because generated plugins are stateless unit structs. But the `Send + Sync` guarantee is stronger than what the framework can actually enforce for arbitrary plugin code.

#### Recommendation

Document that plugins must be thread-safe if the host calls methods from multiple threads. The `Send + Sync` trait bound on the interface trait (`pub trait T: Send + Sync`) provides compile-time checking for the plugin implementation, which is correct.

---

### SEC-17: Deserialization of untrusted data at FFI boundary (Observation)

**File**: `/Users/dstorey/Desktop/fides/fidius-host/src/handle.rs`, line 159
**Confidence**: Medium

#### Description

On the host side, `wire::deserialize(output_slice)` deserializes data provided by the plugin. This data is attacker-controlled if the plugin is malicious. Serde deserialization of untrusted JSON or bincode data can cause:

- Excessive memory allocation (crafted JSON with deeply nested structures)
- CPU exhaustion (bincode with large length prefixes)
- Stack overflow (deeply nested types)

The framework uses standard serde with no limits on depth, size, or allocation.

#### Impact

Denial of service from a malicious plugin. However, since the plugin already has code execution in the host process, DoS via deserialization is a strictly weaker attack than what the plugin can already do.

#### Recommendation

No action needed. The threat model already assumes loaded plugins are trusted. This observation is noted for completeness.
