<!-- Copyright 2026 Colliery, Inc. Licensed under Apache-2.0. -->

# Method and Trait Metadata

Attach static key/value metadata to plugin interface methods and traits so
host applications can categorize, gate, or instrument them without loading
or calling the plugin. Fidius provides the mechanism — hosts define what
the values mean.

## The Attributes

Two attributes work together:

- `#[fidius::method_meta("key", "value")]` on a trait method
- `#[fidius::trait_meta("key", "value")]` on the trait itself

Both accept two string literals. Multiple attributes may appear on the
same item for multiple key/value pairs.

```rust
#[fidius::plugin_interface(version = 1, buffer = PluginAllocated)]
#[fidius::trait_meta("kind", "integration")]
#[fidius::trait_meta("stability", "stable")]
pub trait TodoListManagement: Send + Sync {
    #[fidius::method_meta("effect", "write")]
    #[fidius::method_meta("idempotent", "false")]
    fn create_task(&self, input: NewTask) -> Result<Task, PluginError>;

    #[fidius::method_meta("effect", "read")]
    #[fidius::method_meta("idempotent", "true")]
    fn list_tasks(&self, filter: TaskFilter) -> Result<Vec<Task>, PluginError>;

    // A method with no metadata — legal, yields an empty entry.
    fn version(&self) -> String;
}
```

## Reading Metadata on the Host Side

Metadata is exposed through `PluginHandle`:

```rust
use fidius_host::{PluginHandle, PluginHost};

let host = PluginHost::builder().search_path("./plugins").build()?;
let loaded = host.load("MyPlugin")?;
let handle = PluginHandle::from_loaded(loaded);

// Trait-level metadata:
for (key, value) in handle.trait_metadata() {
    println!("trait: {key} = {value}");
}

// Per-method metadata (indexed by vtable method index):
for (key, value) in handle.method_metadata(0) {
    println!("method 0: {key} = {value}");
}
```

Both methods return `Vec<(&str, &str)>` borrowing from the loaded
library's static data. They return empty when no metadata was declared.

Inspecting without writing code:

```
$ fidius inspect ./plugins/libmy_plugin.dylib
Plugin Registry: ./plugins/libmy_plugin.dylib
  Plugins: 1

  [0] MyPlugin
      Interface: TodoListManagement
      ...
      Trait metadata:
        kind = integration
        stability = stable
      Method metadata:
        [0]:
          effect = write
          idempotent = false
        [1]:
          effect = read
          idempotent = true
```

## Rules and Constraints

- **Keys and values are string literals.** The macro rejects expressions,
  consts, or computed values. Compile error if you try.
- **Keys must be non-empty.** Empty string → compile error.
- **Keys cannot have leading/trailing whitespace.** `"  foo"` → compile
  error. Prevents invisible divergence bugs.
- **No duplicate keys per method or trait.** Two `effect` entries on the
  same method → compile error.
- **The `fidius.*` namespace is reserved.** Any key starting with
  `fidius.` → compile error. Saved for future framework use.
- **Metadata does NOT participate in `interface_hash`.** Adding or
  removing annotations does not invalidate deployed plugins. Hosts that
  want "this method must be annotated X" semantics should check at load
  time, not rely on the hash.
- **Optional methods' metadata is always surfaced.** Even if the plugin
  doesn't implement an optional method (capability bit unset), the
  interface's metadata for that method is still readable. Metadata
  describes the interface declaration, not the implementation.

## What Fidius Does NOT Define

Fidius treats values as opaque strings. `effect = write` means whatever
the consumer framework decides it means. Suggested conventions that hosts
might define:

- `effect = read | write | pure` — whether the method has observable
  side effects
- `idempotent = true | false` — whether it's safe to retry
- `rate_limit_class = <name>` — which rate limit pool applies
- `redact = <field list>` — arguments to redact from telemetry

These are not fidius conventions — host frameworks pick their own.

## When NOT to Use Metadata

Metadata is **static, compile-time, immutable**. Don't use it for:

- Runtime configuration (use your host's config system)
- Values that vary by plugin instance (use a per-plugin registration step)
- Structured data beyond key/value strings (use a separate side-channel)
- Authentication, authorization, or capability tokens (use signed
  manifests or explicit host-side policy)

## See Also

- [ABI Specification](../reference/abi-specification.md#plugindescriptor-layout)
  — the underlying `method_metadata` and `trait_metadata` descriptor fields
- [Interface Evolution](../explanation/interface-evolution.md) — why
  metadata is outside the interface hash
