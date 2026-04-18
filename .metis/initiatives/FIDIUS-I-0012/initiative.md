---
id: typed-host-client-ship-generated
level: initiative
title: "Typed Host Client — Ship Generated Call-by-Name Proxy"
short_code: "FIDIUS-I-0012"
created_at: 2026-04-17T13:23:29.671248+00:00
updated_at: 2026-04-17T18:08:15.867550+00:00
parent: FIDIUS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
initiative_id: typed-host-client-ship-generated
---

# Typed Host Client — Ship Generated Call-by-Name Proxy Initiative

## Context

The `plugin_interface` macro already contains a complete deferred generator for a typed `{Trait}Client` struct at `fidius-macro/src/interface.rs:280-370` (`_generate_client_deferred`). Today, host authors must call plugins through untyped magic indices:

```rust
let out: AddOutput = handle.call_method(__fidius_Calculator::METHOD_ADD, &input)?;
```

The generator is dormant because emitting `PluginHandle` + `CallError` references in the companion module would require the interface crate to depend on `fidius-host`, and interface crates are meant to depend only on the `fidius` facade. FIDIUS-T-0055 shipped the method-index constants but stopped short of the typed client because of the dep-graph concern.

This is the single highest-leverage ergonomic fix available — the code is written, tested in design, and directly addresses the most common host-side friction.

## Goals & Non-Goals

**Goals:**
- Emit a `{Trait}Client` struct alongside each `#[plugin_interface]` invocation
- Typed, named methods: `client.add(&input) -> Result<Output, CallError>`
- Optional methods check capability bit and return `CallError::NotImplemented` before calling
- Plugin crates that do not need the client pay zero cost (no `fidius-host` dep, no `libloading` transitive dep)
- Works in white-label scenarios where `crate = "..."` is set

**Non-Goals:**
- Change the FFI ABI or descriptor layout
- Change the `#[plugin_interface]` or `#[plugin_impl]` input syntax
- Implement async method generation (sync methods only — async is FIDIUS-T-0010 follow-up)

## Detailed Design

### Approach: feature-gated emission in the interface crate

`fidius` grows a `host` feature that re-exports `fidius-host::{PluginHandle, CallError}`. The `plugin_interface` macro emits the client code inside `#[cfg(feature = "host")]` so it's only compiled when the downstream crate enables the feature.

```toml
# host application's Cargo.toml
[dependencies]
my-plugin-api = { version = "1", features = ["host"] }
```

```toml
# interface crate's Cargo.toml
[features]
host = ["fidius/host"]

[dependencies]
fidius = "0.0.5"
```

```toml
# plugin cdylib's Cargo.toml — no host feature
[dependencies]
my-plugin-api = "1"
```

Plugin crates do not enable `host`, so the `Client` struct is never generated in their compilation — no `fidius-host` or `libloading` is pulled in.

### Generated code (unchanged from `_generate_client_deferred`)

```rust
#[cfg(feature = "host")]
pub struct CalculatorClient {
    handle: fidius::PluginHandle,
}

#[cfg(feature = "host")]
impl CalculatorClient {
    pub fn from_handle(handle: fidius::PluginHandle) -> Self { ... }
    pub fn handle(&self) -> &fidius::PluginHandle { ... }

    pub fn add(&self, a: &i64, b: &i64) -> Result<i64, fidius::CallError> {
        self.handle.call_method(0, &(a.clone(), b.clone()))
    }

    // Optional methods check capability first
    pub fn multiply(&self, input: &MulInput) -> Result<MulOutput, fidius::CallError> {
        if !self.handle.has_capability(0) {
            return Err(fidius::CallError::NotImplemented { bit: 0 });
        }
        self.handle.call_method(3, input)
    }
}
```

The existing deferred generator is very close — tuple handling for N args, single-arg passthrough, zero-arg `&()`, optional method capability check. It needs three fixes:

1. Use `#crate_path::CallError` (respect the `crate = "..."` override, currently hardcodes `fidius_host::`)
2. Gate emission with `#[cfg(feature = "host")]`
3. Remove the `.clone()` on multi-arg paths — `call_method` takes `&I: Serialize`, so `&(a, b)` works when both are `&T: Serialize`

### Files to modify

- `fidius-macro/src/interface.rs` — un-defer `_generate_client_deferred`, wire it into `generate_interface`, fix crate-path plumbing and clone issue, remove `#[allow(dead_code)]`
- `fidius/Cargo.toml` — add `host` feature that depends on `fidius-host` as an optional dep
- `fidius/src/lib.rs` — `#[cfg(feature = "host")] pub use fidius_host::{PluginHandle, CallError};`
- `fidius-cli/src/commands.rs` — update `init-interface` scaffold to declare the `host` feature in the generated Cargo.toml
- `fidius-cli/src/commands.rs` — new `init-host` scaffold showing `PluginHost::builder()` + `CalculatorClient::from_handle(handle)` usage
- `fidius-host/tests/integration.rs` — add a test that uses the generated `Client` instead of raw `call_method`
- Docs/tutorial: the three-crate pattern (interface / plugin / host) should center the Client

### Dep-graph verification

A plugin crate built without `host` must not link `libloading`. Add a CI check: build `test-plugin-smoke` and assert `libloading` isn't in `cargo tree`.

## Alternatives Considered

- **Move `PluginHandle` + `CallError` into `fidius-core`.** Rejected: `PluginHandle` fundamentally wraps `Arc<libloading::Library>`. Splitting the type across a loader-less variant in core and a full variant in host adds type-gymnastics for no gain.
- **Host-side macro that re-reads the interface trait.** Rejected: duplicate parsing, drift risk, requires users to write the trait in two macros.
- **Keep raw `call_method(index, input)`.** Rejected: this is the #1 ergonomic complaint from reviewing the codebase. Method-index constants help, but everyone still writes the same boilerplate.

## Implementation Plan

1. Adjust `_generate_client_deferred` to use `#crate_path` consistently, fix multi-arg reference handling, remove `#[cfg]` gates from internal code
2. Add `host` feature to `fidius/Cargo.toml` with optional `fidius-host` dep
3. Re-export `PluginHandle`, `CallError` from `fidius` under `#[cfg(feature = "host")]`
4. Wire `generate_client` into `generate_interface` with `#[cfg(feature = "host")]` attribute on the emitted code
5. Update compile tests to exercise Client generation (interface crate with `host` feature enabled)
6. Update `fidius-host/tests/integration.rs` to use `CalculatorClient`
7. Add dep-graph test: plugin crate built without `host` feature has no `libloading` in its dep tree
8. Update `fidius-cli` scaffolds — `init-interface` adds the `host` feature; new `init-host` command scaffolds a host with Client usage
9. Docs: update tutorial and spec section on Developer Workflow

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- For a trait `Calculator` with methods `add(a,b)`, `version()`, `multiply(input)` (optional), a `CalculatorClient` is generated with typed methods
- `CalculatorClient::from_handle(handle).add(&1, &2)` returns `Ok(3)`
- Plugin crate without `host` feature does not depend on `libloading` or `fidius-host`
- White-label case (`crate = "my_crate::fidius"`) generates a Client that uses `my_crate::fidius::CallError`
- Existing `handle.call_method(index, &input)` path still works (unchanged)