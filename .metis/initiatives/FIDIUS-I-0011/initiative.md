---
id: configurable-crate-path-eliminate
level: initiative
title: "Configurable Crate Path — Eliminate Hardcoded fidius:: in Generated Code"
short_code: "FIDIUS-I-0011"
created_at: 2026-04-01T01:34:48.423842+00:00
updated_at: 2026-04-17T13:17:21.918503+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: configurable-crate-path-eliminate
---

# Configurable Crate Path — Eliminate Hardcoded fidius:: in Generated Code Initiative

## Context

Both `#[plugin_interface]` and `#[plugin_impl]` generate code with hardcoded `fidius::` paths — `fidius::wire::serialize`, `fidius::status::STATUS_OK`, `fidius::descriptor::PluginDescriptor`, etc. This means the `fidius` facade crate must be a direct dependency of every plugin crate, resolvable as exactly `fidius`.

In white-label scenarios (e.g., cloacina), the host's interface crate re-exports everything from fidius. Plugin authors want to depend only on the interface crate:

```toml
[dependencies]
cloacina-workflow-plugin = "1.0"
# no direct fidius dependency
```

But the generated code emits `fidius::wire::serialize(...)`, which doesn't resolve. Today this requires workarounds like:

```rust
use cloacina_workflow_plugin::fidius;
use cloacina_workflow_plugin::__fidius_CloacinaPlugin;
```

This is the same problem serde solved with `#[serde(crate = "...")]`.

## Goals & Non-Goals

**Goals:**
- Generated code resolves fidius types through a configurable path
- The interface macro embeds the crate path so the impl macro picks it up automatically
- Plugin crates in white-label ecosystems only need to depend on the interface crate
- Default behavior unchanged — `fidius::` is the default path when no override is set
- Both `plugin_interface` and `plugin_impl` respect the configured path

**Non-Goals:**
- Changing the public API surface of fidius-core
- Supporting multiple fidius versions in the same dylib
- Renaming any existing types or modules

## Detailed Design

### Two-layer approach

**Layer 1: `plugin_interface` embeds the path**

The interface macro accepts an optional `crate = "..."` attribute:

```rust
#[fidius::plugin_interface(version = 1, buffer = PluginAllocated, crate = "crate")]
pub trait CloacinaPlugin: Send + Sync {
    fn execute(&self, input: String) -> String;
}
```

When `crate = "crate"` is set, the generated companion module (`__fidius_CloacinaPlugin`) stores the crate path as a const or uses it in all generated code. The `plugin_interface` re-export pattern becomes:

```rust
// In cloacina-workflow-plugin/src/lib.rs
pub use fidius;  // re-export so plugins can reach it

#[fidius::plugin_interface(version = 1, buffer = PluginAllocated, crate = "crate")]
pub trait CloacinaPlugin: Send + Sync { ... }
```

With `crate = "crate"`, generated code uses `crate::fidius::` paths — resolving through the interface crate's re-export. Plugin crates only need `cloacina-workflow-plugin` as a dependency.

**Layer 2: `plugin_impl` inherits or overrides**

The impl macro reads the path from the companion module. Optionally, it also accepts a direct override:

```rust
#[plugin_impl(CloacinaPlugin)]  // inherits path from interface
impl CloacinaPlugin for MyPlugin { ... }

// Or explicit override:
#[plugin_impl(CloacinaPlugin, crate = "my_crate::fidius")]
impl CloacinaPlugin for MyPlugin { ... }
```

### Generated code changes

Every `fidius::` reference in codegen becomes `#crate_path::`:

```rust
// Before:
fidius::wire::serialize(&val)
fidius::status::STATUS_OK
fidius::descriptor::PluginDescriptor

// After (with crate_path = "cloacina_workflow_plugin::fidius"):
cloacina_workflow_plugin::fidius::wire::serialize(&val)
cloacina_workflow_plugin::fidius::status::STATUS_OK
cloacina_workflow_plugin::fidius::descriptor::PluginDescriptor
```

The `crate_path` is a `syn::Path` parsed from the attribute, defaulting to `fidius` when absent.

### Files to modify

- `fidius-macro/src/ir.rs` — add `crate_path: syn::Path` to `InterfaceAttrs`, parse from attribute
- `fidius-macro/src/interface.rs` — use `crate_path` in all generated code; embed it in companion module as a type alias or re-export
- `fidius-macro/src/impl_macro.rs` — read `crate_path` from companion module or from impl attribute; use it in all shim codegen
- `fidius-macro/tests/` — tests for custom crate path resolution
- `fidius/src/lib.rs` — ensure re-exports are suitable for path resolution

### Ergonomics for white-label interface crates

The interface crate author does:

```rust
// cloacina-workflow-plugin/src/lib.rs
pub use fidius;

#[fidius::plugin_interface(version = 1, buffer = PluginAllocated, crate = "crate")]
pub trait CloacinaPlugin: Send + Sync {
    fn execute(&self, input: String) -> String;
}
```

Plugin authors just:

```rust
// my-plugin/src/lib.rs
use cloacina_workflow_plugin::{plugin_impl, CloacinaPlugin, __fidius_CloacinaPlugin};

#[plugin_impl(CloacinaPlugin)]
impl CloacinaPlugin for MyPlugin {
    fn execute(&self, input: String) -> String { ... }
}

cloacina_workflow_plugin::fidius::fidius_plugin_registry!();
```

No `use cloacina_workflow_plugin::fidius;` shim needed — the generated code resolves everything through the embedded path.

## Alternatives Considered

- **Require `fidius` as a direct dep always**: The current state. Rejected because it leaks implementation details into plugin crates and prevents clean white-labeling.
- **Use `$crate` in proc macros**: Not possible — `$crate` only works in `macro_rules!`, not proc macros.
- **Detect the path automatically via cargo metadata**: Too fragile and slow (proc macros shouldn't shell out to cargo). Explicit is better.
- **Only support `plugin_impl` override**: Would work but forces every plugin author to remember the path. Having the interface macro embed it is zero-effort for plugin authors.

## Implementation Plan

1. Add `crate_path` parsing to `InterfaceAttrs` in `ir.rs`
2. Update `interface.rs` to use `crate_path` in all generated code and embed it in companion module
3. Update `impl_macro.rs` to read `crate_path` from companion module and use it in shims
4. Add `crate` attribute support to `plugin_impl` for explicit override
5. Tests: compile tests with custom crate paths, white-label scenario test
6. Update scaffold templates and docs