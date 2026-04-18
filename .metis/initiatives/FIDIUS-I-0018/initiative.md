---
id: method-level-metadata-on-plugin
level: initiative
title: "Method-Level Metadata on Plugin Interfaces"
short_code: "FIDIUS-I-0018"
created_at: 2026-04-17T13:43:31.418326+00:00
updated_at: 2026-04-18T01:13:41.868790+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: method-level-metadata-on-plugin
---

# Method-Level Metadata on Plugin Interfaces Initiative

## Context

Host applications built on fidius frequently need to categorize, gate, or instrument plugin methods **before** calling them. Today, the descriptor exposes the method's name (via the trait's vtable layout) and its signature hash (via `interface_hash`), but nothing else. Hosts that need to know:

- Whether a method has observable side effects or is read-only
- Whether it's safe to retry idempotently
- What rate-limit class it belongs to
- What telemetry / redaction rules apply to its arguments
- Any other host-framework-level policy attribute

…have to maintain a parallel hand-written const table mapping method names to metadata, or encode conventions in method naming. Both are fragile and silently drift from the trait. The maintenance burden lands on every downstream consumer (cloacina, internal hosts, future adopters).

This initiative pushes metadata into the trait definition itself and surfaces it through the descriptor so the host can read it without loading or calling the plugin. Fidius defines the mechanism; consumers define the conventions.

## Goals & Non-Goals

**Goals:**
- Interface authors can attach static string key/value metadata to trait methods via `#[fidius::method_meta("key", "value")]`
- Metadata is exposed on `PluginDescriptor` as zero-cost static pointers (no runtime allocation)
- Host reads metadata via `PluginHandle::method_metadata(index)` — returns `&[(&str, &str)]`
- Trait-level metadata via `#[fidius::trait_meta("key", "value")]` (symmetry)
- `fidius inspect` surfaces metadata in its output
- `fidius.*` key namespace reserved for future framework use
- Metadata does **not** participate in `interface_hash` (metadata is a discoverable side-channel, not part of the ABI contract)

**Non-Goals:**
- Fidius defines no semantics for metadata values. `effect=write` means whatever the consumer framework decides. Opaque strings only.
- Not a general KV side-channel for runtime values — metadata is static, compile-time, immutable
- No new wire-level machinery — metadata lives in the descriptor, not transmitted per call
- No mutable or late-bound metadata (e.g., "metadata from a config file") — that's a host-level concern, not an ABI concern
- No support for non-string keys/values (no byte slices, no JSON blobs) — encourages discipline; complex structured metadata belongs elsewhere

## Open Questions from the FR — Recommendations

The feature request raised three open decisions. My recommendations:

### 1. Strings only vs byte slices — **strings only**

Keep `key: *const c_char, value: *const c_char` (null-terminated UTF-8). Byte slices invite users to encode structured data that belongs elsewhere (e.g., JSON blobs → should be a separate file or a different field). Rust string literals are already valid UTF-8. This also makes the `fidius inspect` output trivial to print.

### 2. Reserve `fidius.*` namespace — **yes**

Reject at macro time if key starts with `fidius.` unless we're emitting it ourselves. Cheap to add, frees future moves (e.g., `fidius.since`, `fidius.deprecated`, `fidius.wire_format_override`). Document in the attribute's rustdoc.

### 3. Trait-level metadata too — **yes, include in this initiative**

Symmetry is cheap. Same `MetaKv` struct. Adds two fields to `PluginDescriptor`: `trait_metadata: *const MetaKv, trait_metadata_count: u32`. Motivating use cases: `@kind=integration`, `@stability=experimental`, `@since=1.0`.

## Detailed Design

### Attribute syntax

```rust
#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
#[fidius::trait_meta("kind", "integration")]
#[fidius::trait_meta("stability", "stable")]
pub trait TodoListManagement: Send + Sync {
    #[fidius::method_meta("effect", "write")]
    #[fidius::method_meta("idempotent", "false")]
    #[fidius::method_meta("rate_limit_class", "task_write")]
    fn create_task(&self, input: NewTask) -> Result<Task, PluginError>;

    #[fidius::method_meta("effect", "read")]
    #[fidius::method_meta("idempotent", "true")]
    fn list_tasks(&self, filter: TaskFilter) -> Result<Vec<Task>, PluginError>;

    // A method with no metadata — legal, yields null in the entry array
    fn version(&self) -> String;
}
```

Validation rules (enforced at macro expansion):

- Keys and values must be string literals (no expressions, no consts)
- Keys must be non-empty
- Duplicate keys on the same method (or same trait) are a compile error with spanned diagnostic
- Keys starting with `fidius.` are reserved — compile error if user attempts one
- Values may be empty strings (a value of `""` is legitimate)
- Leading/trailing whitespace in keys is a compile error (prevent invisible divergence bugs)

### Descriptor extensions

```rust
#[repr(C)]
pub struct PluginDescriptor {
    // ... existing fields ...

    /// Pointer to an array of `method_count` `MethodMetaEntry` structs.
    /// Null if the interface declared no method metadata on any method.
    pub method_metadata: *const MethodMetaEntry,

    /// Pointer to an array of `MetaKv` pairs for trait-level metadata.
    /// Null if no trait-level metadata was declared.
    pub trait_metadata: *const MetaKv,
    /// Number of trait-level metadata entries.
    pub trait_metadata_count: u32,
}

#[repr(C)]
pub struct MethodMetaEntry {
    /// Pointer to the `MetaKv` array for this method.
    /// Null if this method declared no metadata.
    pub kvs: *const MetaKv,
    /// Number of kv pairs for this method. Zero when kvs is null.
    pub kv_count: u32,
}

#[repr(C)]
pub struct MetaKv {
    pub key: *const c_char,   // static, null-terminated UTF-8
    pub value: *const c_char, // static, null-terminated UTF-8
}
```

### Codegen pattern

For the `TodoListManagement` example, `plugin_interface` emits (inside the `__fidius_TodoListManagement` companion module):

```rust
// Trait-level metadata
static __FIDIUS_TRAIT_META: [MetaKv; 2] = [
    MetaKv { key: b"kind\0".as_ptr() as _, value: b"integration\0".as_ptr() as _ },
    MetaKv { key: b"stability\0".as_ptr() as _, value: b"stable\0".as_ptr() as _ },
];

// Per-method metadata
static __FIDIUS_META_CREATE_TASK: [MetaKv; 3] = [
    MetaKv { key: b"effect\0".as_ptr() as _, value: b"write\0".as_ptr() as _ },
    MetaKv { key: b"idempotent\0".as_ptr() as _, value: b"false\0".as_ptr() as _ },
    MetaKv { key: b"rate_limit_class\0".as_ptr() as _, value: b"task_write\0".as_ptr() as _ },
];
static __FIDIUS_META_LIST_TASKS: [MetaKv; 2] = [ /* ... */ ];
// version() has none

// The per-method entry table — one MethodMetaEntry per trait method in declaration order
static __FIDIUS_METHOD_META_TABLE: [MethodMetaEntry; 3] = [
    MethodMetaEntry { kvs: __FIDIUS_META_CREATE_TASK.as_ptr(), kv_count: 3 },
    MethodMetaEntry { kvs: __FIDIUS_META_LIST_TASKS.as_ptr(), kv_count: 2 },
    MethodMetaEntry { kvs: std::ptr::null(), kv_count: 0 },  // version() has none
];
```

The descriptor builder (`__fidius_build_todolistmanagement_descriptor`) wires these pointers into the emitted `PluginDescriptor`.

**All metadata is zero-cost at runtime.** The arrays live in `.rodata`; pointer reads have no allocation. Host accesses are `CStr::from_ptr` + `str::to_str`.

### Host-side API

```rust
impl PluginHandle {
    /// Returns the metadata declared on the given method index,
    /// or an empty slice if none declared or if `method_id >= method_count`.
    pub fn method_metadata(&self, method_id: u32) -> Vec<(&str, &str)> { ... }

    /// Returns the trait-level metadata declared on the interface,
    /// or an empty slice if none declared.
    pub fn trait_metadata(&self) -> Vec<(&str, &str)> { ... }
}
```

Returned slices are **owned by the loaded library** — their lifetime is the handle's lifetime (via `_library: Arc<Library>`). Returning `Vec<(&str, &str)>` avoids lifetime juggling on the call site; users who want zero-copy can reach via `handle.info()` or the raw descriptor pointer.

### Interaction with `interface_hash`

**Metadata does NOT participate in the interface hash.** Adding or changing a metadata annotation on an existing method does not invalidate deployed plugins. Rationale: the hash is for ABI drift detection (method signatures changed → incompatible call shape). Metadata is a host-readable side channel that doesn't affect how the FFI call is made. If the host wants to enforce "this method must be annotated `effect=write` before I call it," that's a host-level policy check, not an ABI check.

Document this behavior in the method_meta attribute's rustdoc.

### Interaction with optional methods

Metadata is declared at the interface level. It applies regardless of whether a plugin implements the method. If a plugin declares an optional method but doesn't implement it, its `capabilities` bit is unset — but `method_metadata[method_id]` still points at the interface-declared metadata. Callers should consult both.

### `fidius inspect` output

```
Plugin: BasicCalculator
  Interface: Calculator (hash: 0x1234...)
  Trait metadata:
    kind = integration
    stability = stable
  Methods:
    [0] add (required)
        effect = write
        idempotent = false
    [1] list (required)
        effect = read
        idempotent = true
    [2] version (required)
        (no metadata)
    [3] multiply (optional, capability bit 0)
        (no metadata)
```

### Backward compatibility and ABI_VERSION coordination

This change bumps `ABI_VERSION` from 2 to the next value. **This directly collides with FIDIUS-I-0014 (Arena) and FIDIUS-I-0016 (drop JSON)**, which both also plan an ABI bump. We have three options:

**Option α: Batch all three into ABI v3.** Ship I-0014, I-0016, and this together as a single "ABI v3" release. Fewest churns for downstream users. Implies sequencing all three to land simultaneously.

**Option β: Sequence them — v3, v4, v5.** Each initiative gets its own version. Simpler coordination, but every downstream consumer re-tests three times and the release notes get busy.

**Option γ: Land non-conflicting subsets.** I-0018 and I-0016 are purely descriptor-additive/descriptor-subtractive. They can land in either order with ABI bumps v3 and v4. I-0014 is the biggest change (vtable sig changes for Arena) — land last as v5.

**Recommendation: Option α (batch v3).** The three changes are all pre-1.0 refinements, none depend on the others shipping independently, and a single batch release minimizes downstream thrash. Coordinate by landing I-0015 first (no ABI), then land I-0013 (no ABI), then prepare I-0014/I-0016/I-0018 on branches and merge as one ABI v3 release.

**Decision needed from the user.** See "Open Decisions" below.

### Files to modify

- `fidius-core/src/descriptor.rs` — add `MetaKv`, `MethodMetaEntry` types; extend `PluginDescriptor` with `method_metadata`, `trait_metadata`, `trait_metadata_count` fields; bump `ABI_VERSION`
- `fidius-macro/src/ir.rs` — parse `#[method_meta(...)]` attributes on trait methods; parse `#[trait_meta(...)]` on the trait itself; validation (duplicates, reserved namespace, non-empty keys)
- `fidius-macro/src/interface.rs` — emit static metadata arrays (`__FIDIUS_META_*`, `__FIDIUS_METHOD_META_TABLE`, `__FIDIUS_TRAIT_META`); wire pointers into descriptor builder
- `fidius-macro/src/lib.rs` — re-export `method_meta` and `trait_meta` as new `#[proc_macro_attribute]` entries (OR define them as inert attributes consumed by `plugin_interface` — see implementation note below)
- `fidius/src/lib.rs` — re-export `MetaKv`, `MethodMetaEntry`, `method_meta`, `trait_meta`
- `fidius-host/src/handle.rs` — add `method_metadata(index)` and `trait_metadata()` methods
- `fidius-host/src/loader.rs` — copy metadata pointers into `LoadedPlugin` (or leave as descriptor-pointer access via Arc<Library>)
- `fidius-host/src/types.rs` — consider whether `PluginInfo` should eagerly copy metadata into owned `Vec<(String, String)>` or leave as lazy access via handle
- `fidius-cli/src/commands.rs` — extend `inspect` output
- `fidius-macro/tests/` — new `method_meta.rs` test covering declaration, lookup via vtable+descriptor, compile-fail cases (duplicate, reserved, non-literal)
- `fidius-host/tests/integration.rs` — new test asserting metadata round-trip through the loader
- `docs/how-to/method-metadata.md` (new) — how to declare, what it's for, what fidius doesn't prescribe
- `docs/reference/abi-specification.md` — update ABI spec section
- Spec `FIDIUS-S-0001` — descriptor layout update

### Implementation note: attribute macros

`#[method_meta(...)]` should be an **inert attribute consumed by `plugin_interface`** (similar to how `#[optional(since = N)]` works today per `fidius-macro/src/ir.rs:parse_optional_attr`). Specifically:

- Don't make `method_meta` a standalone `#[proc_macro_attribute]` that rewrites; that leaves a dangling attribute on the trait method and the user has to deal with it.
- Instead, `plugin_interface` walks trait items, collects `#[method_meta(...)]` attrs, strips them from the emitted trait (like `strip_optional_attrs` does today at `fidius-macro/src/interface.rs:27-37`), and emits the static arrays.
- Same for `#[trait_meta(...)]` on the trait itself.

To make this work, users still write `#[fidius::method_meta("k", "v")]` — for IDE support and namespacing. The macro can be a no-op attribute declared as `#[proc_macro_attribute]` that errors at expansion ("must appear inside a #[plugin_interface] trait") — this gives a better error than an unknown-attribute error, and gives rust-analyzer a target for hover/autocomplete.

## Alternatives Considered

- **Runtime method that returns metadata.** E.g., `trait Plugin { fn metadata() -> &'static [&'static [(&str, &str)]] }`. Rejected: pays FFI cost for static data; requires the plugin to be loaded before metadata is discoverable; can't be used for "should I even load this plugin?" decisions.
- **Separate metadata file next to the dylib.** E.g., `plugin.dylib` + `plugin.meta.toml`. Rejected: drift risk (manifest can say one thing, code another), extra IO per plugin discovery, no compile-time validation.
- **Encode in method name.** `read_list_tasks`, `write_create_task`. Rejected: English-specific, fragile, non-extensible, pollutes public API.
- **Consumer-maintained const table keyed by interface_hash + method name.** The status quo. Rejected: silent drift, every consumer re-implements, zero tooling support.
- **Bytes instead of strings for values.** Rejected (see Open Questions above) — invites encoding abuse.
- **Metadata participates in interface_hash.** Rejected — forces rebuilds for non-ABI-affecting annotations like `idempotent=true`. Hosts that want metadata-as-contract can enforce via a separate check.
- **Mutable/late-bound metadata (read from config, environment, etc.).** Rejected — violates the "descriptor is static ABI contract" principle. Hosts that need dynamic policy should compose metadata with runtime config.

## Open Decisions (Human Check-In Required Before Decomposition)

1. **ABI_VERSION coordination strategy (α / β / γ above).** Recommend α — batch I-0014, I-0016, I-0018 into a single ABI v3 release. Decide before any of the three are decomposed.
2. **Should `trait_meta` be in this initiative or deferred?** Recommend include — the codegen is nearly identical to method_meta, same descriptor mechanism. Separating would churn the ABI twice for a one-pointer difference.
3. **API shape of `PluginHandle::method_metadata` — `Vec<(&str, &str)>` vs `&[(&str, &str)]` vs iterator.** Recommend `Vec<(&str, &str)>` — copies the pointer list but not the strings (still borrowed from the library). Users who want zero-alloc can reach through the descriptor directly. Leaning on an iterator makes trivial access harder.
4. **Should optional methods' metadata be surfaced when the capability bit is off?** Recommend yes — the metadata is a property of the interface declaration, not of the plugin implementation. Consumers deciding "should I use this optional method if it's present" need the metadata to answer.

## Implementation Plan

1. Lock decisions from "Open Decisions" section (human check-in)
2. Add `MetaKv`, `MethodMetaEntry` types to `fidius-core/src/descriptor.rs`; bump `ABI_VERSION` (coordinated with I-0014/I-0016 per decision #1)
3. Extend `PluginDescriptor` with `method_metadata`, `trait_metadata`, `trait_metadata_count` fields
4. Add `#[method_meta]` and `#[trait_meta]` inert-attribute parsing in `fidius-macro/src/ir.rs`
5. Emit static metadata arrays in `fidius-macro/src/interface.rs`; wire into descriptor builder
6. Reserve `fidius.*` namespace — compile-time check rejecting user keys in that namespace
7. Add `PluginHandle::method_metadata(index)` and `trait_metadata()` accessors
8. Update `fidius inspect` CLI output
9. Add macro tests: declaration, round-trip, compile-fail (duplicate key, reserved namespace, non-literal, non-empty key required, no whitespace)
10. Add host integration test: load plugin with metadata, read values, verify strings
11. Write `docs/how-to/method-metadata.md`
12. Update ABI spec doc
13. Validate with a cloacina-style use case: declare `effect=write`, read from host, gate on policy

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- `#[fidius::method_meta("effect", "write")]` on a trait method compiles and the value is readable via `PluginHandle::method_metadata(index)` after loading
- `#[fidius::trait_meta("kind", "integration")]` on the trait compiles and is readable via `PluginHandle::trait_metadata()`
- Duplicate keys on the same method produce a compile error with a helpful diagnostic
- Keys starting with `fidius.` produce a compile error ("reserved namespace")
- Non-literal or non-string key/value produces a compile error
- A plugin with no metadata declarations has `method_metadata == null` on descriptor; host accessor returns empty slice without panic
- Adding or removing `method_meta` annotations does not change the `interface_hash` constant
- `fidius inspect` shows trait metadata and per-method metadata in a readable format
- ABI_VERSION bumped (coordinated with I-0014/I-0016)
- `docs/how-to/method-metadata.md` explains use case, syntax, what fidius does and doesn't prescribe
- Integration test confirms metadata round-trips via real dylib load