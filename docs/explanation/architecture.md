<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# Architecture Overview

How fidius works end-to-end: from a Rust trait to a dynamically loaded plugin
callable across the FFI boundary.

## The Pipeline: Trait to FFI Call

Fidius transforms an ordinary Rust trait into a dynamically loadable plugin
through a five-stage pipeline:

```
  Rust trait               proc-macro expansion          compiled cdylib
 ┌──────────┐    #[plugin_interface]    ┌──────────┐     cargo build      ┌──────────┐
 │ trait     │ ──────────────────────►  │ trait     │  ────────────────►   │ .dylib   │
 │ ImageFil- │                         │ + VTable  │                      │ .so      │
 │ ter       │    #[plugin_impl]       │ + hash    │                      │ .dll     │
 │           │ ──────────────────────► │ + shims   │                      │          │
 └──────────┘                          │ + descr.  │                      └─────┬────┘
                                       └──────────┘                            │
                                                                               │
        host calls method               PluginHandle                    dlopen + dlsym
 ┌──────────┐                    ┌──────────┐                      ┌───────────┘
 │ result = │ ◄──deserialize──── │ vtable   │ ◄────validate─────── │
 │ plugin.  │                    │ [index]  │                      │ fidius_get_registry()
 │ process()│ ───serialize────►  │ (fn_ptr) │ ───FFI call────────► │
 └──────────┘                    └──────────┘                      └──────────┘
```

**Stage 1 -- Define.** The interface author writes a Rust trait annotated with
`#[plugin_interface(version = 1, buffer = PluginAllocated)]`. This trait lives
in a dedicated *interface crate*.

**Stage 2 -- Expand.** The proc macro generates:

- The trait itself (with `#[optional]` attributes stripped)
- A `#[repr(C)]` vtable struct with one function pointer per method
- An interface hash constant (FNV-1a of sorted required signatures)
- Capability bit constants for optional methods
- A descriptor builder function

**Stage 3 -- Implement.** A plugin author writes `#[plugin_impl(TraitName)]`
on their impl block. The macro generates:

- `extern "C"` shim functions that deserialize input, call the method, serialize
  output, and catch panics
- A static vtable populated with those shim pointers
- A `PluginDescriptor` containing the hash, version, wire format, buffer
  strategy, and vtable pointer
- An `inventory::submit!` call to register the descriptor

**Stage 4 -- Export.** The plugin calls `fidius_plugin_registry!()` which emits
a `#[no_mangle] pub extern "C" fn fidius_get_registry()`. On first call, this
collects all `inventory`-submitted descriptors into a `PluginRegistry` struct
and caches it via `OnceLock`.

**Stage 5 -- Load and call.** The host application uses `fidius-host` to
`dlopen` the dylib, `dlsym("fidius_get_registry")`, validate magic bytes, ABI
version, interface hash, wire format, and buffer strategy, then wrap the vtable
in a `PluginHandle` for type-safe method calls.

## Why Five Crates

```
fidius (facade)
├── fidius-core      shared types, wire format, hashing, descriptors
└── fidius-macro     proc macros: #[plugin_interface], #[plugin_impl]

fidius-host          host-side loading, validation, calling
fidius-cli           scaffolding, signing, inspection CLI
```

Each crate exists for a specific reason:

| Crate | Why it exists |
|-------|---------------|
| **fidius-core** | The only crate that both plugin and host link against. Defines the `#[repr(C)]` ABI contract: `PluginRegistry`, `PluginDescriptor`, `BufferStrategyKind`, `WireFormat`, status codes, wire serialization, and `PluginError`. Keeping this minimal ensures the plugin side stays light. |
| **fidius-macro** | A `proc-macro` crate (required to be its own crate by Rust). Depends on `fidius-core` to call `interface_hash()` at compile time and reference descriptor types in generated code. |
| **fidius** | Facade that re-exports `fidius-core` types and `fidius-macro` macros. Interface crates depend on this single crate instead of managing two dependencies. |
| **fidius-host** | Host-side logic that plugins never need: `dlopen`/`dlsym` via `libloading`, architecture detection, Ed25519 signature verification, descriptor validation, and `PluginHandle` for calling methods. Kept separate so plugin cdylibs do not link `libloading` or `ed25519-dalek`. |
| **fidius-cli** | The `fidius` binary for scaffolding (`init-interface`, `init-plugin`), signing (`keygen`, `sign`, `verify`), and inspection (`inspect`). Developer tooling, not a library. |

## Dependency Graph

```
                    ┌────────────┐
                    │ fidius-cli │
                    └─────┬──┬──┘
                          │  │
              ┌───────────┘  └──────────┐
              ▼                         ▼
       ┌────────────┐           ┌────────────┐
       │ fidius-host │           │ fidius-core │
       └──────┬─────┘           └──────▲─────┘
              │                        │
              └────────────────────────┘

       ┌────────────┐
       │   fidius    │  (facade)
       └──┬──────┬──┘
          │      │
          ▼      ▼
  ┌────────────┐  ┌────────────┐
  │ fidius-core│  │ fidius-macro│
  └────────────┘  └──────┬─────┘
                         │
                         ▼
                  ┌────────────┐
                  │ fidius-core │
                  └────────────┘
```

Key constraints:

- `fidius-core` has **no** dependency on any other fidius crate. It is the
  foundation.
- `fidius-macro` depends on `fidius-core` because it calls
  `fidius_core::hash::interface_hash()` during macro expansion and references
  `fidius_core::descriptor::*` in generated code.
- `fidius-host` depends on `fidius-core` for descriptor types, status codes,
  wire format, and `PluginError`.
- `fidius-host` does **not** depend on `fidius-macro`. The host never expands
  macros; it works with the binary ABI.
- Plugin cdylibs depend on `fidius` (via their interface crate) but never on
  `fidius-host`.

## Data Flow: Trait Definition to FFI Call

### At Compile Time (Interface Crate)

The `#[plugin_interface]` macro extracts a canonical signature string from each
required method, sorts them, and hashes the combined result with FNV-1a to
produce a compile-time constant (`INTERFACE_HASH`). This hash is the ABI
fingerprint for the interface.

### At Compile Time (Plugin Crate)

The `#[plugin_impl]` macro generates:

1. One `extern "C"` shim per method (handles serialization, deserialization, and
   panic-catching).
2. A static vtable populated with those shim pointers.
3. A `PluginDescriptor` referencing the vtable, interface hash, wire format,
   and buffer strategy.
4. A registration call that makes the descriptor discoverable at runtime.

### At Runtime (Host)

1. **Open** the dylib and look up the registry export symbol.
2. **Validate** magic bytes, registry version, ABI version, interface hash,
   wire format, and buffer strategy. Reject on any mismatch.
3. **Copy** descriptor metadata into owned `PluginInfo` structs and wrap the
   vtable in a `PluginHandle`.
4. **Call** a method: serialize the input, invoke the vtable function pointer,
   check the status code, deserialize the output, and free the plugin-allocated
   buffer.

For the exact binary layout, field offsets, and status codes, see the
[ABI specification](../reference/abi-specification.md).

## Design Philosophy

### Safety at the Boundary

Every FFI shim wraps the real method call in `std::panic::catch_unwind`. Panics
become `STATUS_PANIC` (-4) rather than unwinding across the `extern "C"`
boundary, which is undefined behavior. Serialization errors become
`STATUS_SERIALIZATION_ERROR` (-2). Plugin logic errors become
`STATUS_PLUGIN_ERROR` (-3) with a serialized `PluginError` in the output buffer.

### No Code Until First Call

The host's load sequence executes no plugin code. It reads static data
(descriptors, vtable pointers, string constants) from the dylib's memory.
The `fidius_get_registry()` function only collects already-initialized statics.
The first actual plugin code execution happens when the host calls a method
through the vtable.

### Explicit Contracts

Nothing is implicit. The interface hash catches method signature drift. The
wire format field catches debug/release mismatches. The buffer strategy field
catches allocation protocol mismatches. The ABI version catches descriptor
layout changes. Magic bytes catch non-fidius dylibs. Every mismatch produces
a specific, descriptive `LoadError` variant.

### Plugin Side Stays Light

Plugin cdylibs depend only on `fidius-core` (via the facade). They do not link
`libloading`, `ed25519-dalek`, or any host-side logic. The `fidius-core` crate
carries only `serde`, `serde_json`, `bincode`, `thiserror`, and `inventory` --
all lightweight, widely used crates.

## The Interface Crate Pattern

The "schema" for a plugin system lives in a dedicated crate that both the host
and all plugins depend on. This is the *interface crate*.

```
my-app-plugin-api/
├── Cargo.toml           # depends on `fidius`
└── src/lib.rs           # #[plugin_interface] + re-exports
```

The interface crate re-exports `fidius::plugin_impl` and `fidius::PluginError`
so that plugin authors have a single dependency. This pattern exists because:

1. **Single source of truth.** The trait, hash, version, and capability
   constants are generated once and shared by all consumers.
2. **Plugin authors never see fidius internals.** They depend on the interface
   crate, import `plugin_impl` and `PluginError` from it, and implement the
   trait. That is their entire contract surface.
3. **Host and plugins agree automatically.** Both link against the same
   interface crate, so they get the same interface hash, the same vtable
   layout, and the same wire format constant.
4. **Versioning is simple.** Bumping the interface crate version (with a new
   `version = N` in the attribute) is the single action needed to evolve
   the plugin API.

See also: [Interface Evolution](interface-evolution.md) for how changes to the
interface crate affect compatibility.

---

*Related reference documentation:*

- [fidius-core API](../api/rust/fidius-core.md) — descriptor types, wire format, hashing
- [fidius-macro API](../api/rust/fidius-macro.md) — proc macro attributes and generated code
- [fidius-host API](../api/rust/fidius-host.md) — loading, validation, `PluginHandle`
